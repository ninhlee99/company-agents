#![forbid(unsafe_code)]

use company_store::CompanyStore;
use media_pipeline::{FfmpegExecutor, MediaExecutor};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let company_id = std::env::var("COMPANY_ID")?;
    let poll_seconds = std::env::var("MEDIA_WORKER_POLL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (1..=300).contains(value))
        .unwrap_or(5);

    let store = Arc::new(CompanyStore::connect(&database_url).await?);
    store.migrate().await?;
    let executor = Arc::new(FfmpegExecutor::from_env()?);

    let mut ticker = tokio::time::interval(Duration::from_secs(poll_seconds));
    loop {
        ticker.tick().await;

        let Some(job) = store.claim_media_job(&company_id).await? else {
            continue;
        };

        let executor = Arc::clone(&executor);
        let store = Arc::clone(&store);
        let job_id = job.id.clone();
        let result = tokio::task::spawn_blocking(move || executor.execute(&job)).await;

        match result {
            Ok(Ok(())) => {
                let probe = executor.probe(&job.output_path);
                match probe {
                    Ok(probe) => {
                        let policy = media_policy(&job);
                        let qa = media_pipeline::qa(&probe, &policy);
                        if qa.passed {
                            store.finish_media_job(&job_id, "SUCCEEDED", None).await?;
                        } else {
                            store
                                .finish_media_job(
                                    &job_id,
                                    "QA_FAILED",
                                    Some(&qa.failures.join("; ")),
                                )
                                .await?;
                        }
                    }
                    Err(error) => {
                        store
                            .finish_media_job(
                                &job_id,
                                "QA_FAILED",
                                Some(&error.to_string()),
                            )
                            .await?;
                    }
                }
            }
            Ok(Err(error)) => {
                store
                    .finish_media_job(&job_id, "FAILED", Some(&error.to_string()))
                    .await?;
            }
            Err(error) => {
                store
                    .finish_media_job(
                        &job_id,
                        "FAILED",
                        Some(&format!("worker task join failed: {error}")),
                    )
                    .await?;
            }
        }
    }
}

fn media_policy(job: &media_pipeline::MediaJob) -> media_pipeline::MediaQaPolicy {
    let max_file_size_bytes = std::env::var("MEDIA_MAX_FILE_SIZE_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(50 * 1024 * 1024);

    let min_duration_millis = std::env::var("MEDIA_MIN_DURATION_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1_000);

    let configured_max = std::env::var("MEDIA_MAX_DURATION_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(job.max_duration_seconds as u64 * 1_000);

    let max_duration_millis =
        configured_max.min(job.max_duration_seconds as u64 * 1_000).max(min_duration_millis);

    let allowed_codecs = std::env::var("MEDIA_ALLOWED_CODECS")
        .unwrap_or_else(|_| "h264,vp9".into())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(16)
        .map(ToOwned::to_owned)
        .collect();

    let require_audio = std::env::var("MEDIA_REQUIRE_AUDIO")
        .ok()
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);

    media_pipeline::MediaQaPolicy {
        max_file_size_bytes,
        min_duration_millis,
        max_duration_millis,
        allowed_codecs,
        require_audio,
    }
}
