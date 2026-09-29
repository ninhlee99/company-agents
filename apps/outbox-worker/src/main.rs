#![forbid(unsafe_code)]

use company_store::CompanyStore;
use commercial_sales::messaging::{OutboundEmail, OutboundMessageProvider, ProviderReceipt, ResendProvider};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Serialize;
use sha2::Sha256;
use std::{env, time::Duration};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize)]
struct OutboxDelivery<'a> {
    id: i64,
    company_id: &'a str,
    event_type: &'a str,
    aggregate_id: Option<&'a str>,
    schema_version: i32,
    payload: serde_json::Value,
    attempts: i32,
}

fn env_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (min..=max).contains(value))
        .unwrap_or(default)
}

fn sign(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

async fn deliver(
    client: &Client,
    url: &str,
    secret: &str,
    event: &OutboxDelivery<'_>,
) -> Result<(), String> {
    let body = serde_json::to_vec(event).map_err(|error| error.to_string())?;
    let mut request = client
        .post(url)
        .header("content-type", "application/json")
        .header("x-company-event-id", event.id.to_string())
        .header("x-company-event-type", event.event_type)
        .header("x-company-event-attempt", event.attempts.to_string())
        .body(body.clone());

    if !secret.is_empty() {
        request = request.header("x-company-event-signature", sign(secret, &body));
    }

    let response = request.send().await.map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("webhook returned {}", response.status()));
    }
    Ok(())
}

async fn deliver_email(provider: &ResendProvider, event: &OutboxDelivery<'_>) -> Result<ProviderReceipt, String> {
    let message_id = event.payload.get("message_id").and_then(|v| v.as_str()).ok_or("message_id missing")?;
    let email = OutboundEmail {
        message_id: message_id.to_owned(),
        from: std::env::var("RESEND_FROM").map_err(|_| "RESEND_FROM is required".to_owned())?,
        to: event.payload.get("recipient").and_then(|v| v.as_str()).ok_or("recipient missing")?.to_owned(),
        subject: event.payload.get("subject").and_then(|v| v.as_str()).ok_or("subject missing")?.to_owned(),
        html: event.payload.get("html_body").and_then(|v| v.as_str()).ok_or("html_body missing")?.to_owned(),
        idempotency_key: event.payload.get("idempotency_key").and_then(|v| v.as_str()).ok_or("idempotency_key missing")?.to_owned(),
        unsubscribe_url: event.payload.get("unsubscribe_url").and_then(|v| v.as_str()).map(str::to_owned),
    };
    provider.send_email(&email).await.map_err(|e| e.to_string())
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = env::var("DATABASE_URL")?;
    let company_id = env::var("COMPANY_ID")?;
    let webhook_url = env::var("OUTBOX_WEBHOOK_URL").unwrap_or_default();
    let webhook_secret = env::var("OUTBOX_WEBHOOK_SECRET").unwrap_or_default();
    let resend = ResendProvider::from_env().ok();
    let poll_seconds = env_u64("OUTBOX_POLL_SECONDS", 2, 1, 60);
    let lease_seconds = env_u64("OUTBOX_LEASE_SECONDS", 30, 5, 300);
    let batch_size = env_u64("OUTBOX_BATCH_SIZE", 20, 1, 100) as i64;
    let max_attempts = env_u64("OUTBOX_MAX_ATTEMPTS", 10, 1, 100) as i32;

    if webhook_url.trim().is_empty() {
        return Err("OUTBOX_WEBHOOK_URL is required; refusing to consume events without a sink".into());
    }
    if !webhook_url.starts_with("https://")
        && !webhook_url.starts_with("http://127.0.0.1")
        && !webhook_url.starts_with("http://localhost")
    {
        return Err("OUTBOX_WEBHOOK_URL must use HTTPS, localhost, or 127.0.0.1".into());
    }

    let store = CompanyStore::connect(&database_url).await?;
    store.migrate().await?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .build()?;
    let owner = format!("outbox-worker-{}", Uuid::new_v4());

    loop {
        let events = store
            .claim_outbox_events(&company_id, &owner, lease_seconds as i64, max_attempts, batch_size)
            .await?;

        if events.is_empty() {
            tokio::time::sleep(Duration::from_secs(poll_seconds)).await;
            continue;
        }

        for event in events {
            let delivery = OutboxDelivery {
                id: event.id,
                company_id: &event.company_id,
                event_type: &event.event_type,
                aggregate_id: event.aggregate_id.as_deref(),
                schema_version: event.schema_version,
                payload: event.payload.clone(),
                attempts: event.attempts,
            };

            let result = if event.event_type == "EXTERNAL_EMAIL_SEND" {
                if let Some(message_id) = delivery.payload.get("message_id").and_then(|v| v.as_str()) {
                    store.mark_outbound_email_processing(&event.company_id, message_id).await?;
                }
                match resend.as_ref() {
                    Some(provider) => match deliver_email(provider, &delivery).await {
                        Ok(receipt) => {
                            if let Some(message_id) = delivery.payload.get("message_id").and_then(|v| v.as_str()) {
                                store.record_outbound_email_result(&event.company_id, message_id, Some(&receipt.provider), Some(&receipt.provider_reference), None).await?;
                            }
                            Ok(())
                        }
                        Err(error) => Err(error),
                    },
                    None => Err("Resend provider is not configured".to_owned()),
                }
            } else {
                deliver_webhook(&client, &webhook_url, &webhook_secret, &delivery).await
            };

            match result {
                Ok(()) => store.mark_outbox_published(event.id, &owner).await?,
                Err(error) => {
                    if event.event_type == "EXTERNAL_EMAIL_SEND" {
                        if let Some(message_id) = delivery.payload.get("message_id").and_then(|v| v.as_str()) {
                            store.record_outbound_email_result(&event.company_id, message_id, None, None, Some(&error)).await?;
                        }
                    }
                    eprintln!("outbox event {} delivery failed: {error}", event.id);
                    store.release_outbox_event(event.id, &owner, &error).await?;
                }
            }
        }
    }
}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run().await
}
