use company_store::CompanyStore;
use media_pipeline::{FfmpegExecutor, MediaExecutor, MediaJob, MediaQaPolicy};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let company_id = std::env::var("COMPANY_ID")?;
    let poll_seconds = std::env::var("MEDIA_WORKER_POLL_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.max(1))
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

        let worker = executor.clone();
        let job_for_render = job.clone();
        let result =
            tokio::task::spawn_blocking(move || worker.execute(&job_for_render)).await;
        match result {
            Ok(Ok(())) => {
                let probe = executor.probe(&job.output_path);
                match probe {
                    Ok(probe) => {
                        let policy = media_policy(&job);
                        let qa = media_pipeline::qa(&probe, &policy);
                        if qa.passed {
                            store.finish_media_job(&job.id, "SUCCEEDED", None).await?;
                        } else {
                            let reason = qa.failures.join("; ");
                            store
                                .finish_media_job(&job.id, "QA_FAILED", Some(&reason))
                                .await?;
                        }
                    }
                    Err(error) => {
                        store
                            .finish_media_job(&job.id, "QA_FAILED", Some(&error.to_string()))
                            .await?;
                    }
                }
            }
            Ok(Err(error)) => {
                store
                    .finish_media_job(&job.id, "FAILED", Some(&error.to_string()))
                    .await?;
            }
            Err(join_error) => {
                store
                    .finish_media_job(
                        &job.id,
                        "FAILED",
                        Some(&format!("worker task failed: {join_error}")),
                    )
                    .await?;
            }
        }
    }
}

fn media_policy(job: &MediaJob) -> MediaQaPolicy {
    let max_file_size_bytes = std::env::var("MEDIA_MAX_FILE_SIZE_BYTES")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(50 * 1024 * 1024);
    let min_duration_millis = std::env::var("MEDIA_MIN_DURATION_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1_000);
    let configured_max = std::env::var("MEDIA_MAX_DURATION_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(job.max_duration_seconds as u64 * 1_000);
    let max_duration_millis = configured_max.min(job.max_duration_seconds as u64 * 1_000);
    let allowed_codecs = std::env::var("MEDIA_ALLOWED_CODECS")
        .unwrap_or_else(|_| "h264,vp9".into())
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .take(16)
        .map(ToOwned::to_owned)
        .collect();
    let require_audio = std::env::var("MEDIA_REQUIRE_AUDIO")
        .ok()
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);

    MediaQaPolicy {
        max_file_size_bytes,
        min_duration_millis,
        max_duration_millis,
        allowed_codecs,
        require_audio,
    }
}
