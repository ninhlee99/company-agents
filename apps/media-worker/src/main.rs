use company_store::CompanyStore;
use media_pipeline::{FfmpegExecutor, MediaExecutor};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        let result = tokio::task::spawn_blocking(move || worker.execute(&job)).await;
        match result {
            Ok(Ok(())) => {
                store.finish_media_job(&job.id, true, None).await?;
            }
            Ok(Err(error)) => {
                store
                    .finish_media_job(&job.id, false, Some(&error.to_string()))
                    .await?;
            }
            Err(join_error) => {
                store
                    .finish_media_job(
                        &job.id,
                        false,
                        Some(&format!("worker task failed: {join_error}")),
                    )
                    .await?;
            }
        }
    }
}
