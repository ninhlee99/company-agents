#![forbid(unsafe_code)]

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs,
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    time::timeout,
};
use uuid::Uuid;

const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_ERROR_BYTES: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkerJob {
    job_id: Uuid,
    protocol_version: u16,
    backend: String,
    model: String,
    system: String,
    user: String,
    response_format: String,
    allow_tools: bool,
    attempt: i32,
    lease_token: Uuid,
    lease_expires_in_ms: u64,
    expires_in_ms: u64,
}

#[derive(Debug, Serialize)]
struct ClaimRequest {
    backend: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompleteRequest<'a> {
    lease_token: Uuid,
    output: &'a str,
}

#[derive(Debug, Serialize)]
struct FailRequest<'a> {
    lease_token: Uuid,
    error: &'a str,
}

#[derive(Debug, Serialize)]
struct AdapterInput<'a> {
    protocol_version: u16,
    backend: &'a str,
    model: &'a str,
    system: &'a str,
    user: &'a str,
    response_format: &'a str,
    allow_tools: bool,
    attempt: i32,
}

fn env_duration(name: &str, default_seconds: u64, min: u64, max: u64) -> Duration {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(min, max))
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(default_seconds))
}

fn required(name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let value = env::var(name)?.trim().to_owned();
    if value.len() < 8 {
        return Err(format!("{name} is too short").into());
    }
    Ok(value)
}

fn validate_job(job: &WorkerJob) -> Result<(), String> {
    if job.protocol_version != 1
        || !matches!(job.backend.as_str(), "gemini-web" | "chatgpt-web" | "claude-web")
        || job.response_format != "json_object"
        || job.allow_tools
    {
        return Err("unsupported or unsafe web relay job".into());
    }

    if job.system.len() > 128 * 1024 || job.user.len() > 384 * 1024 {
        return Err("job prompt exceeds worker limits".into());
    }

    if job.expires_in_ms == 0 || job.lease_expires_in_ms == 0 {
        return Err("job lease or expiry already elapsed".into());
    }

    Ok(())
}

async fn run_adapter(
    adapter_kind: &str,
    command_path: Option<&str>,
    job: &WorkerJob,
    timeout_duration: Duration,
    workdir: &str,
) -> Result<String, String> {
    let input = serde_json::to_vec(&AdapterInput {
        protocol_version: job.protocol_version,
        backend: &job.backend,
        model: &job.model,
        system: &job.system,
        user: &job.user,
        response_format: &job.response_format,
        allow_tools: job.allow_tools,
        attempt: job.attempt,
    })
    .map_err(|error| error.to_string())?;

    let (program, args, stdin_payload) = match adapter_kind {
        "command" | "browser-command" => {
            let command_path = command_path.ok_or("WEB_SESSION_ADAPTER_COMMAND is required")?;
            (
                command_path.to_owned(),
                Vec::new(),
                Some(input.clone()),
            )
        }
        "codex-chatgpt" => {
            if job.backend != "chatgpt-web" {
                return Err("codex-chatgpt adapter only accepts chatgpt-web jobs".into());
            }
            let prompt = format!(
                "{}\n\nReturn exactly one JSON object and no prose. Do not use tools.\n\nUSER REQUEST:\n{}",
                job.system, job.user
            );
            (
                env::var("CODEX_BIN").unwrap_or_else(|_| "codex".into()),
                vec![
                    "exec".into(),
                    "--json".into(),
                    "--sandbox".into(),
                    "read-only".into(),
                    "--model".into(),
                    job.model.clone(),
                    prompt,
                ],
                None,
            )
        }
        "claude-code" => {
            if job.backend != "claude-web" {
                return Err("claude-code adapter only accepts claude-web jobs".into());
            }
            (
                env::var("CLAUDE_BIN").unwrap_or_else(|_| "claude".into()),
                vec![
                    "-p".into(),
                    job.user.clone(),
                    "--output-format".into(),
                    "json".into(),
                    "--max-turns".into(),
                    "1".into(),
                    "--permission-mode".into(),
                    "plan".into(),
                    "--system-prompt".into(),
                    job.system.clone(),
                    "--model".into(),
                    job.model.clone(),
                ],
                None,
            )
        }
        other => return Err(format!("unknown WEB_SESSION_ADAPTER_KIND={other}")),
    };

    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(if stdin_payload.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("WEB_RELAY_BACKEND", &job.backend)
        .env("WEB_RELAY_MODEL", &job.model)
        .current_dir(&workdir);

    let mut child = command
        .spawn()
        .map_err(|error| format!("adapter spawn failed: {error}"))?;

    if let (Some(payload), Some(mut stdin)) = (stdin_payload, child.stdin.take()) {
        stdin
            .write_all(&payload)
            .await
            .map_err(|error| format!("adapter stdin failed: {error}"))?;
    }

    let result = timeout(timeout_duration, async {
        let output = child
            .wait_with_output()
            .await
            .map_err(|error| format!("adapter wait failed: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(
                &output.stderr[..output.stderr.len().min(MAX_ERROR_BYTES)],
            );
            return Err(format!("adapter exited unsuccessfully: {stderr}"));
        }

        if output.stdout.len() > MAX_OUTPUT_BYTES {
            return Err("adapter output exceeds 1 MiB".into());
        }

        let raw = String::from_utf8(output.stdout)
            .map_err(|_| "adapter output is not UTF-8".to_owned())?;
        extract_json_object(adapter_kind, &raw)
    })
    .await
    .map_err(|_| "adapter execution timed out".to_owned())??;

    Ok(result)
}

fn extract_json_object(adapter_kind: &str, raw: &str) -> Result<String, String> {
    match adapter_kind {
        "command" | "browser-command" => validate_json_object(raw),
        "claude-code" => {
            let envelope: serde_json::Value = serde_json::from_str(raw.trim())
                .map_err(|_| "Claude Code output is not JSON".to_owned())?;
            let candidate = envelope
                .get("result")
                .or_else(|| envelope.get("output_text"))
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "Claude Code JSON output has no result text".to_owned())?;
            validate_json_object(candidate)
        }
        "codex-chatgpt" => {
            let mut candidate = None;
            for line in raw.lines().rev() {
                let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                if let Some(text) = find_agent_text(&value) {
                    candidate = Some(text.to_owned());
                    break;
                }
            }
            let candidate = candidate.ok_or_else(|| "Codex JSONL output has no final agent text".to_owned())?;
            validate_json_object(&candidate)
        }
        other => Err(format!("unknown adapter kind {other}")),
    }
}

fn validate_json_object(raw: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw.trim()).map_err(|_| "adapter output is not valid JSON".to_owned())?;
    if !value.is_object() {
        return Err("adapter output must be exactly one JSON object".into());
    }
    Ok(value.to_string())
}

fn find_agent_text(value: &serde_json::Value) -> Option<&str> {
    if value.get("type").and_then(serde_json::Value::as_str)
        .is_some_and(|kind| kind == "agent_message" || kind == "output_text")
    {
        if let Some(text) = value.get("text").and_then(serde_json::Value::as_str) {
            return Some(text);
        }
    }

    if let Some(item) = value.get("item") {
        if let Some(text) = find_agent_text(item) {
            return Some(text);
        }
    }

    if let Some(content) = value.get("content").and_then(serde_json::Value::as_array) {
        for block in content.iter().rev() {
            if let Some(text) = find_agent_text(block) {
                return Some(text);
            }
            if let Some(text) = block.get("text").and_then(serde_json::Value::as_str) {
                return Some(text);
            }
        }
    }

    None
}

async fn claim(client: &Client, relay_url: &str, token: &str, backend: Option<&str>) -> Result<Option<WorkerJob>, String> {
    let response = client
        .post(format!("{relay_url}/v1/jobs/claim"))
        .bearer_auth(token)
        .json(&ClaimRequest {
            backend: backend.map(str::to_owned),
        })
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("claim HTTP {status}"));
    }

    response
        .json::<Option<WorkerJob>>()
        .await
        .map_err(|error| error.to_string())
}

async fn complete(client: &Client, relay_url: &str, token: &str, job: &WorkerJob, output: &str) -> Result<(), String> {
    client
        .post(format!("{relay_url}/v1/jobs/{}/complete", job.job_id))
        .bearer_auth(token)
        .json(&CompleteRequest {
            lease_token: job.lease_token,
            output,
        })
        .send()
        .await
        .map_err(|error| error.to_string())
        .and_then(|response| {
            if response.status().is_success() {
                Ok(())
            } else {
                Err(format!("complete HTTP {}", response.status()))
            }
        })
}

async fn fail(client: &Client, relay_url: &str, token: &str, job: &WorkerJob, error: &str) -> Result<(), String> {
    let bounded = error.chars().take(MAX_ERROR_BYTES).collect::<String>();
    client
        .post(format!("{relay_url}/v1/jobs/{}/fail", job.job_id))
        .bearer_auth(token)
        .json(&FailRequest {
            lease_token: job.lease_token,
            error: &bounded,
        })
        .send()
        .await
        .map_err(|error| error.to_string())
        .and_then(|response| {
            if response.status().is_success() {
                Ok(())
            } else {
                Err(format!("fail HTTP {}", response.status()))
            }
        })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let relay_url = required("LLM_RELAY_URL")?.trim_end_matches('/').to_owned();
    let worker_token = required("LLM_RELAY_WORKER_TOKEN")?;
    let adapter_kind = env::var("WEB_SESSION_ADAPTER_KIND")
        .unwrap_or_else(|_| "browser-command".into())
        .to_ascii_lowercase();
    if !matches!(
        adapter_kind.as_str(),
        "browser-command" | "command" | "codex-chatgpt" | "claude-code"
    ) {
        return Err("unsupported WEB_SESSION_ADAPTER_KIND".into());
    }
    let adapter_command = env::var("WEB_SESSION_ADAPTER_COMMAND")
        .ok()
        .filter(|value| !value.trim().is_empty());
    if matches!(adapter_kind.as_str(), "browser-command" | "command")
        && adapter_command.is_none()
    {
        return Err("WEB_SESSION_ADAPTER_COMMAND is required for command adapters".into());
    }

    let backend_filter = env::var("WEB_SESSION_BACKEND").ok().filter(|v| !v.trim().is_empty());
    let idle_sleep = env_duration("WEB_SESSION_IDLE_SLEEP_SECONDS", 2, 1, 15);
    let adapter_timeout = env_duration("WEB_SESSION_ADAPTER_TIMEOUT_SECONDS", 45, 5, 120);
    let workdir = env::var("WEB_SESSION_WORKDIR")
        .unwrap_or_else(|_| "/tmp/company-agents-llm".into());
    fs::create_dir_all(&workdir)
        .map_err(|error| format!("cannot create isolated web session workdir: {error}"))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("company-agents-web-session-worker/0.1")
        .build()?;

    loop {
        let job = match claim(&client, &relay_url, &worker_token, backend_filter.as_deref()).await {
            Ok(Some(job)) => job,
            Ok(None) => {
                tokio::time::sleep(idle_sleep).await;
                continue;
            }
            Err(error) => {
                eprintln!("claim error: {error}");
                tokio::time::sleep(idle_sleep).await;
                continue;
            }
        };

        if let Err(error) = validate_job(&job) {
            let _ = fail(&client, &relay_url, &worker_token, &job, &error).await;
            continue;
        }

        match run_adapter(
            &adapter_kind,
            adapter_command.as_deref(),
            &job,
            adapter_timeout,
            &workdir,
        )
        .await {
            Ok(output) => {
                if let Err(error) = complete(&client, &relay_url, &worker_token, &job, &output).await {
                    eprintln!("complete error for {}: {error}", job.job_id);
                }
            }
            Err(error) => {
                eprintln!("adapter error for {}: {error}", job.job_id);
                if let Err(report_error) = fail(&client, &relay_url, &worker_token, &job, &error).await {
                    eprintln!("fail report error for {}: {report_error}", job.job_id);
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_restrictions_are_enforced() {
        let base = WorkerJob {
            job_id: Uuid::nil(),
            protocol_version: 1,
            backend: "chatgpt-web".into(),
            model: "gpt-5.6-luna".into(),
            system: "system".into(),
            user: "user".into(),
            response_format: "json_object".into(),
            allow_tools: false,
            attempt: 1,
            lease_token: Uuid::nil(),
            lease_expires_in_ms: 10_000,
            expires_in_ms: 10_000,
        };
        assert!(validate_job(&base).is_ok());

        let mut unsafe_job = base.clone();
        unsafe_job.allow_tools = true;
        assert!(validate_job(&unsafe_job).is_err());

        let mut expired = base;
        expired.expires_in_ms = 0;
        assert!(validate_job(&expired).is_err());
    }

    #[test]
    fn claude_json_result_is_unwrapped_and_validated() {
        let raw = r#"{"type":"result","result":"{\"action\":\"ProduceReport\"}"}"#;
        assert_eq!(
            extract_json_object("claude-code", raw).unwrap(),
            r#"{"action":"ProduceReport"}"#
        );
    }

    #[test]
    fn codex_jsonl_extracts_last_agent_message() {
        let raw = concat!(
            r#"{"type":"item.completed","item":{"type":"reasoning","summary":[]}}"#,
            "\n",
            r#"{"type":"item.completed","item":{"type":"agent_message","content":[{"type":"output_text","text":"{\"action\":\"ProduceReport\"}"}]}}"#,
            "\n"
        );
        assert_eq!(
            extract_json_object("codex-chatgpt", raw).unwrap(),
            r#"{"action":"ProduceReport"}"#
        );
    }

    #[test]
    fn browser_command_requires_one_json_object() {
        assert_eq!(
            validate_json_object(r#"{"action":"ProduceReport","cost_minor":0}"#).unwrap(),
            r#"{"action":"ProduceReport","cost_minor":0}"#
        );
        assert!(validate_json_object(r#"["not-object"]"#).is_err());
        assert!(validate_json_object("not-json").is_err());
    }
}
