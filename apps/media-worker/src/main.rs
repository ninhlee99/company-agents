#![forbid(unsafe_code)]

use company_store::CompanyStore;
use media_pipeline::{FfmpegRunner, MediaExecutor};
use std::{env, sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = env::var("DATABASE_URL")?;
    let company_id = env::var("COMPANY_ID")?;
    let poll_seconds = env::var("MEDIA_WORKER_POLL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (1..=300).contains(value))
        .unwrap_or(5);

    let ffmpeg_bin = env::var("FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".into());
    let timeout_seconds = env::var("FFMPEG_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (1..=86_400).contains(value))
        .unwrap_or(900);

    let store = Arc::new(CompanyStore::connect(&database_url).await?);
    store.migrate().await?;
    let runner = Arc::new(FfmpegRunner::new(
        ffmpeg_bin,
        Duration::from_secs(timeout_seconds),
    ));

    let mut ticker = tokio::time::interval(Duration::from_secs(poll_seconds));
    loop {
        ticker.tick().await;

        let Some(job) = store.claim_media_job(&company_id).await? else {
            continue;
        };

        let worker = Arc::clone(&runner);
        let job_for_execution = job.clone();
        let job_id = job.id.clone();

        let result = tokio::task::spawn_blocking(move || {
            worker.execute(&job_for_execution)
        })
        .await;

        match result {
            Ok(Ok(())) => {
                store.finish_media_job(&job_id, "SUCCEEDED", None).await?;
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
                        Some(&format!("media worker task failed: {error}")),
                    )
                    .await?;
            }
        }
    }
}
