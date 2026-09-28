#![forbid(unsafe_code)]

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    env,
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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
    command_path: &str,
    job: &WorkerJob,
    timeout_duration: Duration,
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

    let mut child = Command::new(command_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("WEB_RELAY_BACKEND", &job.backend)
        .env("WEB_RELAY_MODEL", &job.model)
        .spawn()
        .map_err(|error| format!("adapter spawn failed: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&input)
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

        let output = String::from_utf8(output.stdout)
            .map_err(|_| "adapter output is not UTF-8".to_owned())?
            .trim()
            .to_owned();

        if output.is_empty() {
            return Err("adapter returned empty output".into());
        }

        let value: serde_json::Value = serde_json::from_str(&output)
            .map_err(|_| "adapter output is not one JSON object".to_owned())?;
        if !value.is_object() {
            return Err("adapter output must be a JSON object".into());
        }

        Ok(output)
    })
    .await
    .map_err(|_| "adapter execution timed out".to_owned())??;

    Ok(result)
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
    let bounded = &error[..error.len().min(MAX_ERROR_BYTES)];
    client
        .post(format!("{relay_url}/v1/jobs/{}/fail", job.job_id))
        .bearer_auth(token)
        .json(&FailRequest {
            lease_token: job.lease_token,
            error: bounded,
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
    let adapter_command = required("WEB_SESSION_ADAPTER_COMMAND")?;
    let backend_filter = env::var("WEB_SESSION_BACKEND").ok().filter(|v| !v.trim().is_empty());
    let idle_sleep = env_duration("WEB_SESSION_IDLE_SLEEP_SECONDS", 2, 1, 15);
    let adapter_timeout = env_duration("WEB_SESSION_ADAPTER_TIMEOUT_SECONDS", 45, 5, 120);

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

        match run_adapter(&adapter_command, &job, adapter_timeout).await {
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
