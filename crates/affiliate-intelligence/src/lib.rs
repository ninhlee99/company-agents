#![forbid(unsafe_code)]

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock};

const SCORE_MAX: u32 = 10_000;
const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_FEED_BYTES: usize = 64 * 1024 * 1024;
const MAX_TIKTOK_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const DEFAULT_MAX_PRODUCT_AGE_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Product {
    pub id: String,
    pub gtin: Option<String>,
    pub advertiser_id: String,
    pub advertiser_name: Option<String>,
    pub name: String,
    pub description: String,
    pub category: String,
    pub brand: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub price_minor: i128,
    pub old_price_minor: Option<i128>,
    pub currency: String,
    pub rating_bps: Option<u32>,
    pub review_count: Option<u64>,
    pub stock_quantity: Option<u64>,
    pub in_stock: bool,
    pub savings_bps: Option<u32>,
    pub seller_reputation_bps: Option<u32>,
    pub refund_rate_bps: Option<u32>,
    pub delivery_reliability_bps: Option<u32>,
    pub commission_group: Option<String>,
    pub commission_rate_bps: Option<u32>,
    pub commission_fixed_minor: Option<i128>,
    pub commission_currency: Option<String>,
    pub source: String,
    pub source_updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Coupon {
    pub id: String,
    pub advertiser_id: String,
    pub title: String,
    pub description: String,
    pub code: Option<String>,
    pub discount_bps: Option<u32>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub active: bool,
    pub exclusive: bool,
    pub attributable: bool,
    pub url: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductSearchQuery {
    pub category: Option<String>,
    pub keywords: Vec<String>,
    pub currency: Option<String>,
    pub min_price_minor: Option<i128>,
    pub max_price_minor: Option<i128>,
    pub min_commission_bps: Option<u32>,
    pub require_coupon: bool,
    pub require_coupon_code: bool,
    pub min_rating_bps: Option<u32>,
    pub min_reviews: Option<u64>,
    pub in_stock_only: bool,
    pub max_results: usize,
    pub as_of_date: Option<String>,
}

impl Default for ProductSearchQuery {
    fn default() -> Self {
        Self {
            category: None,
            keywords: Vec::new(),
            currency: None,
            min_price_minor: None,
            max_price_minor: None,
            min_commission_bps: None,
            require_coupon: false,
            require_coupon_code: false,
            min_rating_bps: None,
            min_reviews: None,
            in_stock_only: true,
            max_results: 20,
            as_of_date: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankingWeights {
    pub quality_bps: u32,
    pub commission_bps: u32,
    pub coupon_bps: u32,
    pub content_fit_bps: u32,
    pub savings_bps: u32,
    pub evidence_confidence_bps: u32,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            quality_bps: 3_500,
            commission_bps: 2_500,
            coupon_bps: 1_000,
            content_fit_bps: 1_500,
            savings_bps: 500,
            evidence_confidence_bps: 1_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityAssessment {
    pub score_bps: u32,
    pub confidence_bps: u32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EconomicsAssessment {
    pub score_bps: u32,
    pub confidence_bps: u32,
    pub expected_commission_minor: Option<i128>,
    pub expected_net_commission_minor: Option<i128>,
    pub effective_discount_bps: Option<u32>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProductFreshnessStatus {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductFreshness {
    pub status: ProductFreshnessStatus,
    pub age_days: Option<i64>,
    pub max_age_days: i64,
}

pub struct RankedProduct {
    pub product: Product,
    pub freshness: ProductFreshness,
    pub coupons: Vec<Coupon>,
    pub quality: QualityAssessment,
    pub economics: EconomicsAssessment,
    pub content_fit_bps: u32,
    pub data_confidence_bps: u32,
    pub score_bps: u32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResponse {
    pub source: String,
    pub total_candidates: usize,
    pub returned: usize,
    pub products: Vec<RankedProduct>,
}

#[derive(Debug)]
pub enum AffiliateError {
    InvalidQuery(String),
    Provider(String),
    Parse(String),
    PayloadTooLarge,
}

impl fmt::Display for AffiliateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQuery(v) => write!(f, "invalid affiliate query: {v}"),
            Self::Provider(v) => write!(f, "affiliate provider error: {v}"),
            Self::Parse(v) => write!(f, "affiliate provider parse error: {v}"),
            Self::PayloadTooLarge => write!(f, "affiliate payload exceeds safety limit"),
        }
    }
}

impl std::error::Error for AffiliateError {}

pub struct TikTokShopProvider {
    client: Client,
    base_url: String,
    search_path: String,
    access_token: String,
    seller_access_token: Option<String>,
    app_key: String,
    app_secret: String,
    shop_cipher: Option<String>,
    advertiser_id: String,
    coupon_search_path: String,
    last_coupons: RwLock<Vec<Coupon>>,
    search_lock: Mutex<()>,
}

impl TikTokShopProvider {
    pub fn from_env() -> Result<Self, AffiliateError> {
        let base_url = std::env::var("TTS_BASE_URL")
            .unwrap_or_else(|_| "https://open-api.tiktokglobalshop.com".into());
        if !base_url.starts_with("https://") {
            return Err(AffiliateError::Provider(
                "TTS_BASE_URL must use HTTPS".into(),
            ));
        }

        let search_path = std::env::var("TTS_AFFILIATE_SEARCH_PATH").unwrap_or_else(|_| {
            "/affiliate_seller/202405/open_collaborations/products/search".into()
        });
        if !search_path.starts_with('/') || search_path.len() > 256 {
            return Err(AffiliateError::Provider(
                "TTS_AFFILIATE_SEARCH_PATH is invalid".into(),
            ));
        }

        let coupon_search_path = std::env::var("TTS_COUPON_SEARCH_PATH")
            .unwrap_or_else(|_| "/promotion/202406/coupons/search".into());
        if !coupon_search_path.starts_with('/') || coupon_search_path.len() > 256 {
            return Err(AffiliateError::Provider(
                "TTS_COUPON_SEARCH_PATH is invalid".into(),
            ));
        }

        let access_token = secret_from_env("TTS_ACCESS_TOKEN")?;
        let seller_access_token = secret_from_env_optional("TTS_SELLER_ACCESS_TOKEN")?;
        let app_key = std::env::var("TTS_APP_KEY")
            .map_err(|_| AffiliateError::Provider("TTS_APP_KEY is required".into()))?;
        let app_secret = secret_from_env("TTS_APP_SECRET")?;
        if app_key.trim().is_empty() || app_key.len() > 128 || app_secret.len() > 512 {
            return Err(AffiliateError::Provider(
                "TTS app credentials are invalid".into(),
            ));
        }

        let shop_cipher = std::env::var("TTS_SHOP_CIPHER")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(|v| v.trim().to_owned());

        let advertiser_id = std::env::var("TTS_ADVERTISER_ID")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| {
                shop_cipher
                    .as_deref()
                    .map(|value| format!("tiktok:{value}"))
                    .unwrap_or_else(|| "tiktok:unknown-shop".into())
            });

        Ok(Self {
            client: Client::builder()
                .timeout(DEFAULT_HTTP_TIMEOUT)
                .user_agent("company-agents-affiliate/0.1")
                .build()
                .map_err(|error| AffiliateError::Provider(error.to_string()))?,
            base_url: base_url.trim_end_matches('/').to_owned(),
            search_path,
            access_token,
            seller_access_token,
            app_key,
            app_secret,
            shop_cipher,
            advertiser_id,
            coupon_search_path,
            last_coupons: RwLock::new(Vec::new()),
            search_lock: Mutex::new(()),
        })
    }

    async fn fetch_products(
        &self,
        query: &ProductSearchQuery,
    ) -> Result<Vec<Product>, AffiliateError> {
        let mut all = Vec::new();
        let mut page_token: Option<String> = None;

        for _ in 0..10 {
            let timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
            let mut query_params = vec![
                ("app_key".to_owned(), self.app_key.clone()),
                ("page_size".to_owned(), "20".into()),
                ("timestamp".to_owned(), timestamp.to_string()),
            ];
            if let Some(shop_cipher) = self.shop_cipher.as_deref() {
                query_params.push(("shop_cipher".into(), shop_cipher.into()));
            }

            let mut body = serde_json::Map::new();
            if let Some(category) = query.category.as_deref() {
                if category.chars().all(|character| character.is_ascii_digit()) {
                    body.insert("category".into(), serde_json::json!({ "id": category }));
                }
            }
            if !query.keywords.is_empty() {
                body.insert(
                    "title_keywords".into(),
                    serde_json::json!(query.keywords.iter().take(12).collect::<Vec<_>>()),
                );
            }

            if query.min_price_minor.is_some() || query.max_price_minor.is_some() {
                let currency = query.currency.clone().unwrap_or_else(|| "USD".into());
                let units = minor_units_for_currency(&currency, 2);
                let mut range = serde_json::Map::new();
                if let Some(min) = query.min_price_minor {
                    range.insert(
                        "amount_ge".into(),
                        serde_json::Value::String(format_major_decimal(min, units)),
                    );
                }
                if let Some(max) = query.max_price_minor {
                    range.insert(
                        "amount_lt".into(),
                        serde_json::Value::String(format_major_decimal(max, units)),
                    );
                }
                body.insert("sales_price_range".into(), serde_json::Value::Object(range));
            }

            if let Some(min_commission) = query.min_commission_bps {
                body.insert(
                    "commission_rate_range".into(),
                    serde_json::json!({ "rate_ge": min_commission }),
                );
            }

            if let Some(token) = page_token.as_deref() {
                body.insert("page_token".into(), serde_json::Value::String(token.into()));
            }

            let body_value = serde_json::Value::Object(body);
            let body_bytes = serde_json::to_vec(&body_value)
                .map_err(|error| AffiliateError::Parse(error.to_string()))?;
            let sign = tiktok_sign(
                &self.search_path,
                &query_params,
                &body_bytes,
                &self.app_secret,
            )?;
            query_params.push(("sign".into(), sign));

            let response = self
                .send_tiktok_request(
                    &self.search_path,
                    &query_params,
                    &body_bytes,
                    &self.access_token,
                )
                .await?;
            let parsed = parse_tiktok_search_response(&response, &self.advertiser_id)?;
            all.extend(parsed.products);

            if all.len() >= query.max_results.min(200) || parsed.next_page_token.is_empty() {
                break;
            }
            page_token = Some(parsed.next_page_token);
        }

        Ok(all)
    }

    async fn fetch_coupons(
        &self,
        query: &ProductSearchQuery,
    ) -> Result<Vec<Coupon>, AffiliateError> {
        let Some(access_token) = self.seller_access_token.as_deref() else {
            return Ok(Vec::new());
        };
        let Some(shop_cipher) = self.shop_cipher.as_deref() else {
            return Ok(Vec::new());
        };

        let mut coupons = Vec::new();
        let mut page_token: Option<String> = None;

        for _ in 0..5 {
            let timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
            let mut query_params = vec![
                ("app_key".to_owned(), self.app_key.clone()),
                ("page_size".to_owned(), "100".into()),
                ("timestamp".to_owned(), timestamp.to_string()),
                ("shop_cipher".to_owned(), shop_cipher.to_owned()),
            ];
            if let Some(token) = page_token.as_deref() {
                query_params.push(("page_token".into(), token.into()));
            }

            let mut body = serde_json::json!({
                "status": ["ONGOING"],
                "display_type": [
                    "PROMO_CODE",
                    "CREATOR_EXCLUSIVE",
                    "LIVE",
                    "REGULAR"
                ]
            });
            if !query.keywords.is_empty() {
                body["title_keyword"] = serde_json::Value::String(query.keywords.join(" "));
            }

            let body_bytes = serde_json::to_vec(&body)
                .map_err(|error| AffiliateError::Parse(error.to_string()))?;
            let sign = tiktok_sign(
                &self.coupon_search_path,
                &query_params,
                &body_bytes,
                &self.app_secret,
            )?;
            query_params.push(("sign".into(), sign));

            let value = self
                .send_tiktok_request(
                    &self.coupon_search_path,
                    &query_params,
                    &body_bytes,
                    access_token,
                )
                .await?;

            let page =
                parse_tiktok_coupon_response(&value, &self.advertiser_id, &self.coupon_search_path)?;
            coupons.extend(page.coupons);

            if page.next_page_token.is_empty() {
                break;
            }
            page_token = Some(page.next_page_token);
        }

        Ok(coupons)
    }

    async fn send_tiktok_request(
        &self,
        path: &str,
        query_params: &[(String, String)],
        body: &[u8],
        access_token: &str,
    ) -> Result<serde_json::Value, AffiliateError> {
        let mut url = format!("{}{}", self.base_url, path);
        let query = query_params
            .iter()
            .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        url.push('?');
        url.push_str(&query);

        let mut last_error = None;

        for attempt in 0..3_u32 {
            let response = self
                .client
                .post(&url)
                .header("content-type", "application/json")
                .header("x-tts-access-token", access_token)
                .body(body.to_vec())
                .send()
                .await
                .map_err(|error| AffiliateError::Provider(error.to_string()))?;

            let status = response.status();
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned);
            let bytes = response
                .bytes()
                .await
                .map_err(|error| AffiliateError::Provider(error.to_string()))?;

            if bytes.len() > MAX_TIKTOK_RESPONSE_BYTES {
                return Err(AffiliateError::PayloadTooLarge);
            }

            if status.as_u16() == 429 || status.is_server_error() {
                last_error = Some(format!("TikTok HTTP {status}"));
                if attempt < 2 {
                    tokio::time::sleep(response_retry_delay(retry_after.as_deref(), attempt)).await;
                    continue;
                }
                return Err(AffiliateError::Provider(format!(
                    "TikTok affiliate API HTTP {status}: {}",
                    String::from_utf8_lossy(&bytes[..bytes.len().min(4096)])
                )));
            }

            if !status.is_success() {
                return Err(AffiliateError::Provider(format!(
                    "TikTok affiliate API HTTP {status}: {}",
                    String::from_utf8_lossy(&bytes[..bytes.len().min(4096)])
                )));
            }

            let value = serde_json::from_slice::<serde_json::Value>(&bytes)
                .map_err(|error| AffiliateError::Parse(error.to_string()))?;
            let code = value.get("code").and_then(|value| value.as_i64()).unwrap_or(-1);
            if code == 36009002 && attempt < 2 {
                tokio::time::sleep(response_retry_delay(retry_after, attempt)).await;
                continue;
            }
            return Ok(value);
        }

        Err(AffiliateError::Provider(
            last_error.unwrap_or_else(|| "TikTok request failed".into()),
        ))
    }
}

#[derive(Debug)]
struct TikTokCouponPage {
    next_page_token: String,
    coupons: Vec<Coupon>,
}

fn parse_tiktok_coupon_response(
    value: &serde_json::Value,
    advertiser_id: &str,
    source: &str,
) -> Result<TikTokCouponPage, AffiliateError> {
    let code = value.get("code").and_then(|value| value.as_i64()).unwrap_or(-1);
    if code != 0 {
        let message = value
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown TikTok coupon API error");
        return Err(AffiliateError::Provider(format!(
            "TikTok coupon API code={code}: {message}"
        )));
    }

    let data = value
        .get("data")
        .ok_or_else(|| AffiliateError::Parse("TikTok coupon response missing data".into()))?;
    let next_page_token = data
        .get("next_page_token")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_owned();
    let items = data
        .get("coupons")
        .and_then(|value| value.as_array())
        .ok_or_else(|| AffiliateError::Parse("TikTok coupon response missing coupons array".into()))?;

    let mut coupons = Vec::new();
    for item in items.iter().take(100) {
        let id = match item.get("id").and_then(|value| value.as_str()) {
            Some(id) if !id.trim().is_empty() => id.trim().to_owned(),
            _ => continue,
        };
        let title = item
            .get("title")
            .and_then(|value| value.as_str())
            .unwrap_or("TikTok Shop coupon")
            .trim()
            .to_owned();
        let display_type = item
            .get("display_type")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .trim()
            .to_owned();
        let status = item
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .trim()
            .to_owned();

        if !status.eq_ignore_ascii_case("ONGOING") {
            continue;
        }

        let code = item
            .get("code")
            .and_then(|value| value.as_str())
            .or_else(|| item.get("promo_code").and_then(|value| value.as_str()))
            .or_else(|| item.get("promotion_code").and_then(|value| value.as_str()))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        let discount_bps = item
            .get("discount")
            .and_then(|discount| {
                discount
                    .get("percentage")
                    .or_else(|| discount.get("discount_percentage"))
                    .and_then(|value| value.as_f64())
            })
            .filter(|value| value.is_finite() && *value >= 0.0)
            .map(|value| (value * 100.0).round().min(SCORE_MAX as f64) as u32)
            .or_else(|| parse_percent_from_text(&title));

        let (starts_at, ends_at) = parse_tiktok_coupon_window(item);

        coupons.push(Coupon {
            id,
            advertiser_id: advertiser_id.to_owned(),
            title,
            description: if display_type.is_empty() {
                "TikTok Shop promotion".into()
            } else {
                format!("TikTok Shop {display_type} coupon")
            },
            code,
            discount_bps,
            starts_at,
            ends_at,
            active: true,
            exclusive: display_type.eq_ignore_ascii_case("CREATOR_EXCLUSIVE"),
            attributable: true,
            url: None,
            source: source.into(),
        });
    }

    Ok(TikTokCouponPage {
        next_page_token,
        coupons,
    })
}

fn parse_tiktok_coupon_window(item: &serde_json::Value) -> (Option<String>, Option<String>) {
    for key in ["redemption_duration", "claim_duration"] {
        if let Some(duration) = item.get(key) {
            let start = duration
                .get("start_time")
                .and_then(|value| value.as_i64())
                .and_then(format_unix_rfc3339);
            let end = duration
                .get("end_time")
                .and_then(|value| value.as_i64())
                .and_then(format_unix_rfc3339);
            if start.is_some() || end.is_some() {
                return (start, end);
            }
        }
    }
    (None, None)
}

#[derive(Debug)]
struct TikTokSearchPage {
    next_page_token: String,
    products: Vec<Product>,
}

fn parse_tiktok_search_response(
    value: &serde_json::Value,
    advertiser_id: &str,
) -> Result<TikTokSearchPage, AffiliateError> {
    let code = value.get("code").and_then(|value| value.as_i64()).unwrap_or(-1);
    if code != 0 {
        let message = value
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown TikTok API error");
        let request_id = value
            .get("request_id")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown");
        return Err(AffiliateError::Provider(format!(
            "TikTok affiliate API code={code}, request_id={request_id}: {message}"
        )));
    }

    let data = value
        .get("data")
        .ok_or_else(|| AffiliateError::Parse("TikTok response missing data".into()))?;
    let next_page_token = data
        .get("next_page_token")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_owned();
    let items = data
        .get("products")
        .and_then(|value| value.as_array())
        .ok_or_else(|| AffiliateError::Parse("TikTok response missing products array".into()))?;

    let mut products = Vec::with_capacity(items.len().min(20));

    for item in items.iter().take(20) {
        let id = match item.get("id").and_then(|value| value.as_str()).map(str::trim) {
            Some(id) if !id.is_empty() => id.to_owned(),
            _ => continue,
        };

        let title = item
            .get("title")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .trim()
            .to_owned();
        if title.is_empty() {
            continue;
        }

        let sale = item
            .get("sales_price")
            .or_else(|| item.get("sale_price"));
        let original = item.get("original_price");
        let currency = sale
            .and_then(|value| value.get("currency"))
            .and_then(|value| value.as_str())
            .or_else(|| {
                original
                    .and_then(|value| value.get("currency"))
                    .and_then(|value| value.as_str())
            })
            .unwrap_or("USD")
            .to_ascii_uppercase();

        let sale_amount = sale
            .and_then(|value| value.get("minimum_amount"))
            .and_then(|value| value.as_str())
            .or_else(|| {
                sale
                    .and_then(|value| value.get("maximum_amount"))
                    .and_then(|value| value.as_str())
            });
        let Some(sale_amount) = sale_amount else {
            continue;
        };

        let units = minor_units_for_currency(&currency, 2);
        let price_minor = parse_decimal_minor(sale_amount, units)?;
        if price_minor < 0 {
            continue;
        }

        let old_price_minor = original
            .and_then(|value| {
                value
                    .get("maximum_amount")
                    .and_then(|x| x.as_str())
                    .or_else(|| value.get("minimum_amount").and_then(|x| x.as_str()))
            })
            .and_then(|value| parse_decimal_minor(value, units).ok());

        let commission = item.get("commission");
        let commission_rate_bps = commission
            .and_then(|value| value.get("rate"))
            .and_then(|value| value.as_u64())
            .map(|value| value.min(SCORE_MAX as u64) as u32);

        let commission_fixed_minor = commission
            .and_then(|value| {
                let amount = value.get("amount")?.as_str()?;
                parse_decimal_minor_floor(amount, units).ok()
            })
            .and_then(|value| if value >= 0 { Some(value) } else { None });

        let commission_currency = commission
            .and_then(|value| value.get("currency"))
            .and_then(|value| value.as_str())
            .map(|value| value.to_ascii_uppercase());

        let category = item
            .get("category_chains")
            .and_then(|value| value.as_array())
            .map(|chains| {
                chains
                    .iter()
                    .filter_map(|chain| chain.get("local_name").and_then(|x| x.as_str()))
                    .collect::<Vec<_>>()
                    .join(" > ")
            })
            .unwrap_or_default();

        let advertiser_name = item
            .get("shop")
            .and_then(|value| value.get("name"))
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned);

        let detail_url = item
            .get("detail_link")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned();
        if !detail_url.starts_with("https://") {
            continue;
        }

        let image_url = item
            .get("main_image_url")
            .and_then(|value| value.as_str())
            .filter(|value| value.starts_with("https://"))
            .map(ToOwned::to_owned);

        let savings_bps = old_price_minor
            .filter(|old| *old > price_minor && *old > 0)
            .and_then(|old| {
                old.checked_sub(price_minor)
                    .and_then(|diff| diff.checked_mul(SCORE_MAX as i128))
                    .map(|value| ((value / old).min(SCORE_MAX as i128)) as u32)
            });

        products.push(Product {
            id,
            gtin: None,
            advertiser_id: advertiser_id.to_owned(),
            advertiser_name,
            name: title,
            description: String::new(),
            category,
            brand: None,
            url: detail_url,
            image_url,
            price_minor,
            old_price_minor,
            currency,
            rating_bps: None,
            review_count: None,
            stock_quantity: None,
            in_stock: item
                .get("has_inventory")
                .and_then(|value| value.as_bool())
                .unwrap_or(true),
            savings_bps,
            seller_reputation_bps: None,
            refund_rate_bps: None,
            delivery_reliability_bps: None,
            commission_group: None,
            commission_rate_bps,
            commission_fixed_minor,
            commission_currency,
            source: "tiktok_shop_open_collaboration".into(),
            source_updated_at: None,
        });
    }

    Ok(TikTokSearchPage {
        next_page_token,
        products,
    })
}

fn format_major_decimal(value_minor: i128, minor_units: u32) -> String {
    let negative = value_minor < 0;
    let magnitude = value_minor.unsigned_abs();
    let scale = 10_u128.pow(minor_units.min(6));
    let whole = magnitude / scale;
    let fraction = magnitude % scale;
    if minor_units == 0 {
        return if negative { format!("-{whole}") } else { whole.to_string() };
    }
    let mut fraction_text = fraction.to_string();
    while fraction_text.len() < minor_units as usize {
        fraction_text.insert(0, '0');
    }
    let sign = if negative { "-" } else { "" };
    format!("{sign}{whole}.{fraction_text}")
}

fn parse_decimal_minor_floor(
    value: &str,
    minor_units: u32,
) -> Result<i128, AffiliateError> {
    let normalized = value.trim().replace(',', "");
    let negative = normalized.starts_with('-');
    let unsigned = normalized.trim_start_matches('-');
    let mut pieces = unsigned.split('.');
    let whole = pieces.next().unwrap_or("0");
    let fractional = pieces.next().unwrap_or("");
    if pieces.next().is_some()
        || !whole.chars().all(|c| c.is_ascii_digit())
        || !fractional.chars().all(|c| c.is_ascii_digit())
    {
        return Err(AffiliateError::Parse(format!("invalid decimal amount: {value}")));
    }
    let scale = 10_i128.pow(minor_units.min(6));
    let whole_value = whole
        .parse::<i128>()
        .map_err(|_| AffiliateError::Parse("amount overflow".into()))?;
    let mut fraction = fractional.to_owned();
    fraction.truncate(minor_units as usize);
    while fraction.len() < minor_units as usize {
        fraction.push('0');
    }
    let fraction_value = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<i128>()
            .map_err(|_| AffiliateError::Parse("amount overflow".into()))?
    };
    let result = whole_value
        .checked_mul(scale)
        .and_then(|value| value.checked_add(fraction_value))
        .ok_or_else(|| AffiliateError::Parse("amount overflow".into()))?;
    Ok(if negative { -result } else { result })
}

fn format_unix_rfc3339(value: i64) -> Option<String> {
    let datetime = time::OffsetDateTime::from_unix_timestamp(value).ok()?;
    datetime
        .format(&time::format_description::well_known::Rfc3339)
        .ok()
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(hex_digit(byte >> 4));
            out.push(hex_digit(byte & 0x0f));
        }
    }
    out
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + value - 10) as char,
        _ => '0',
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

fn tiktok_sign(
    path: &str,
    query: &[(String, String)],
    body: &[u8],
    app_secret: &str,
) -> Result<String, AffiliateError> {
    if path.is_empty() || !path.starts_with('/') {
        return Err(AffiliateError::Provider("TikTok sign path is invalid".into()));
    }

    let mut params = query
        .iter()
        .filter(|(key, _)| key != "sign" && key != "access_token")
        .cloned()
        .collect::<Vec<_>>();
    params.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut input = String::with_capacity(
        path.len()
            + params
                .iter()
                .map(|(key, value)| key.len() + value.len())
                .sum::<usize>()
            + body.len()
            + app_secret.len() * 2,
    );
    input.push_str(path);
    for (key, value) in params {
        input.push_str(&key);
        input.push_str(&value);
    }
    input.push_str(&String::from_utf8_lossy(body));
    let signing_payload = format!("{app_secret}{input}{app_secret}");

    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(app_secret.as_bytes())
        .map_err(|_| AffiliateError::Provider("invalid TikTok app secret".into()))?;
    mac.update(signing_payload.as_bytes());
    Ok(bytes_to_hex(&mac.finalize().into_bytes()))
}

fn response_retry_delay(retry_after: Option<&str>, attempt: u32) -> Duration {
    if let Some(value) = retry_after
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value <= 60)
    {
        return Duration::from_secs(value);
    }
    Duration::from_millis(250 * (1_u64 << attempt.min(4)))
}

#[async_trait]
impl AffiliateProvider for TikTokShopProvider {
    fn name(&self) -> &'static str {
        "tiktok_shop"
    }

    async fn products(&self) -> Result<Vec<Product>, AffiliateError> {
        Err(AffiliateError::Provider(
            "TikTok provider requires a ProductSearchQuery; use AffiliateProvider::search".into(),
        ))
    }

    async fn coupons(&self) -> Result<Vec<Coupon>, AffiliateError> {
        Ok(self.last_coupons.read().await.clone())
    }

    async fn search(&self, query: &ProductSearchQuery) -> Result<Vec<Product>, AffiliateError> {
        let _guard = self.search_lock.lock().await;
        let products = self.fetch_products(query).await?;
        let coupons = self.fetch_coupons(query).await?;
        *self.last_coupons.write().await = coupons;
        Ok(products)
    }
}

#[async_trait]
pub trait AffiliateProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn products(&self) -> Result<Vec<Product>, AffiliateError>;

    async fn coupons(&self) -> Result<Vec<Coupon>, AffiliateError>;

    async fn search(
        &self,
        _query: &ProductSearchQuery,
    ) -> Result<Vec<Product>, AffiliateError> {
        self.products().await
    }
}

pub struct AggregateAffiliateProvider {
    providers: Vec<Arc<dyn AffiliateProvider>>,
}

impl AggregateAffiliateProvider {
    pub fn new(providers: Vec<Arc<dyn AffiliateProvider>>) -> Result<Self, AffiliateError> {
        if providers.is_empty() {
            return Err(AffiliateError::Provider(
                "affiliate provider set cannot be empty".into(),
            ));
        }
        Ok(Self { providers })
    }

    pub fn provider_names(&self) -> Vec<&'static str> {
        self.providers.iter().map(|provider| provider.name()).collect()
    }
}

#[async_trait]
impl AffiliateProvider for AggregateAffiliateProvider {
    fn name(&self) -> &'static str {
        "aggregate"
    }

    async fn products(&self) -> Result<Vec<Product>, AffiliateError> {
        let mut products = Vec::new();
        let mut last_error = None;

        for provider in &self.providers {
            match provider.products().await {
                Ok(mut values) => products.append(&mut values),
                Err(error) => last_error = Some(error),
            }
        }

        if products.is_empty() {
            return Err(last_error.unwrap_or_else(|| {
                AffiliateError::Provider("all affiliate providers returned no products".into())
            }));
        }

        Ok(products)
    }

    async fn coupons(&self) -> Result<Vec<Coupon>, AffiliateError> {
        let mut coupons = Vec::new();

        for provider in &self.providers {
            if let Ok(mut values) = provider.coupons().await {
                coupons.append(&mut values);
            }
        }

        Ok(coupons)
    }

    async fn search(
        &self,
        query: &ProductSearchQuery,
    ) -> Result<Vec<Product>, AffiliateError> {
        let mut products = Vec::new();
        let mut failures = Vec::new();

        for provider in &self.providers {
            match provider.search(query).await {
                Ok(mut values) => products.append(&mut values),
                Err(error) => failures.push(format!("{}: {error}", provider.name())),
            }
        }

        if products.is_empty() {
            return Err(AffiliateError::Provider(format!(
                "all affiliate providers failed: {}",
                failures.join(" | ")
            )));
        }

        Ok(products)
    }
}

pub async fn search(
    provider: Arc<dyn AffiliateProvider>,
    query: ProductSearchQuery,
) -> Result<SearchResponse, AffiliateError> {
    validate_query(&query)?;
    let products = provider.search(&query).await?;
    let coupons = provider.coupons().await?;
    let ranked = rank_products(&products, &coupons, &query);
    let total = ranked.len();
    let max_results = query.max_results.max(1).min(200);
    Ok(SearchResponse {
        source: provider.name().to_owned(),
        total_candidates: total,
        returned: ranked.len().min(max_results),
        products: ranked.into_iter().take(max_results).collect(),
    })
}

pub fn rank_products(
    products: &[Product],
    coupons: &[Coupon],
    query: &ProductSearchQuery,
) -> Vec<RankedProduct> {
    let weights = RankingWeights::default();
    let mut dedup: HashMap<String, RankedProduct> = HashMap::new();

    for product in products {
        if !product_matches(product, query) {
            continue;
        }

        let freshness = product_freshness(product, query.as_of_date.as_deref());
        if freshness.status == ProductFreshnessStatus::Stale {
            continue;
        }

        let usable_coupons = coupons
            .iter()
            .filter(|coupon| coupon.active)
            .filter(|coupon| coupon.advertiser_id == product.advertiser_id)
            .filter(|coupon| coupon_is_active_on(coupon, query.as_of_date.as_deref()))
            .cloned()
            .collect::<Vec<_>>();

        if query.require_coupon && usable_coupons.is_empty() {
            continue;
        }
        if query.require_coupon_code && !usable_coupons.iter().any(|coupon| {
            coupon.code.as_deref().is_some_and(|code| !code.trim().is_empty())
        }) {
            continue;
        }

        let quality = quality_assessment(product);
        let economics = economics_assessment(product, &usable_coupons);
        let content_fit_bps = content_fit(product, query);
        let data_confidence_bps = confidence(product, &quality, &economics, &freshness);
        let score_bps = weighted_score(
            quality.score_bps,
            economics.score_bps,
            coupon_score(&usable_coupons),
            content_fit_bps,
            product.savings_bps.unwrap_or(0),
            data_confidence_bps,
            &weights,
        );

        let mut reasons = Vec::new();
        reasons.extend(quality.reasons.iter().cloned());
        reasons.extend(economics.reasons.iter().cloned());
        if content_fit_bps >= 8_000 {
            reasons.push("strong category/keyword content fit".into());
        } else if content_fit_bps >= 4_000 {
            reasons.push("moderate content fit".into());
        }
        if usable_coupons.iter().any(|c| c.code.is_some()) {
            reasons.push("active voucher code available in provider data".into());
        }
        match freshness.status {
            ProductFreshnessStatus::Fresh => {
                reasons.push(format!(
                    "source data is {} day(s) old; within the {}-day freshness window",
                    freshness.age_days.unwrap_or_default(),
                    freshness.max_age_days
                ));
            }
            ProductFreshnessStatus::Unknown => {
                reasons.push("source freshness unknown; provider did not supply a valid update timestamp".into());
            }
            ProductFreshnessStatus::Stale => {}
        }
        if data_confidence_bps < 6_000 {
            reasons.push("limited evidence coverage; treat ranking as lower confidence".into());
        }

        let candidate = RankedProduct {
            product: product.clone(),
            freshness: freshness.clone(),
            coupons: usable_coupons,
            quality,
            economics,
            content_fit_bps,
            data_confidence_bps,
            score_bps,
            reasons,
        };

        let key = dedupe_key(product);
        let replace = dedup
            .get(&key)
            .map(|current| candidate.score_bps > current.score_bps)
            .unwrap_or(true);
        if replace {
            dedup.insert(key, candidate);
        }
    }

    let mut results = dedup.into_values().collect::<Vec<_>>();
    results.sort_by(|a, b| {
        b.score_bps
            .cmp(&a.score_bps)
            .then_with(|| {
                b.economics
                    .expected_commission_minor
                    .unwrap_or_default()
                    .cmp(&a.economics.expected_commission_minor.unwrap_or_default())
            })
            .then_with(|| a.product.id.cmp(&b.product.id))
    });
    results
}

fn validate_query(query: &ProductSearchQuery) -> Result<(), AffiliateError> {
    if let (Some(min), Some(max)) = (query.min_price_minor, query.max_price_minor) {
        if min < 0 || max < min {
            return Err(AffiliateError::InvalidQuery("invalid price bounds".into()));
        }
    }
    if let Some(min) = query.min_commission_bps {
        if min > SCORE_MAX {
            return Err(AffiliateError::InvalidQuery(
                "commission must be <= 10000 bps".into(),
            ));
        }
    }
    if let Some(rating) = query.min_rating_bps {
        if rating > SCORE_MAX {
            return Err(AffiliateError::InvalidQuery(
                "rating must be <= 10000 bps".into(),
            ));
        }
    }
    if let Some(currency) = query.currency.as_deref() {
        if currency.len() != 3 || !currency.as_bytes().iter().all(u8::is_ascii_uppercase) {
            return Err(AffiliateError::InvalidQuery(
                "currency must be a three-letter uppercase ISO-like code".into(),
            ));
        }
    }
    if query.max_results == 0 || query.max_results > 200 {
        return Err(AffiliateError::InvalidQuery(
            "max_results must be between 1 and 200".into(),
        ));
    }
    if let Some(date) = query.as_of_date.as_deref() {
        if !is_iso_date(date) {
            return Err(AffiliateError::InvalidQuery(
                "as_of_date must be YYYY-MM-DD".into(),
            ));
        }
    }
    Ok(())
}

fn product_matches(product: &Product, query: &ProductSearchQuery) -> bool {
    if !valid_product_evidence(product) {
        return false;
    }
    if let Some(currency) = query.currency.as_deref() {
        if product.currency != currency {
            return false;
        }
    }
    if let Some(min) = query.min_price_minor {
        if product.price_minor < min {
            return false;
        }
    }
    if let Some(max) = query.max_price_minor {
        if product.price_minor > max {
            return false;
        }
    }
    if query.in_stock_only && !product.in_stock {
        return false;
    }
    if let Some(min_commission) = query.min_commission_bps {
        if product
            .commission_rate_bps
            .is_none_or(|v| v < min_commission)
        {
            return false;
        }
    }
    if let Some(min_rating) = query.min_rating_bps {
        if product.rating_bps.is_none_or(|v| v < min_rating) {
            return false;
        }
    }
    if let Some(min_reviews) = query.min_reviews {
        if product.review_count.is_none_or(|v| v < min_reviews) {
            return false;
        }
    }

    let haystack = format!(
        "{} {} {} {}",
        product.name,
        product.description,
        product.category,
        product.brand.clone().unwrap_or_default()
    )
    .to_ascii_lowercase();

    if let Some(category) = query.category.as_deref() {
        if !haystack.contains(&category.to_ascii_lowercase()) {
            return false;
        }
    }

    query
        .keywords
        .iter()
        .all(|keyword| haystack.contains(&keyword.to_ascii_lowercase()))
}

fn valid_product_evidence(product: &Product) -> bool {
    if product.id.trim().is_empty()
        || product.advertiser_id.trim().is_empty()
        || product.name.trim().is_empty()
        || product.price_minor < 0
        || product.currency.len() != 3
        || !product.currency.as_bytes().iter().all(u8::is_ascii_uppercase)
        || product.url.trim().is_empty()
    {
        return false;
    }
    if !product.url.starts_with("https://") {
        return false;
    }
    if let Some(old_price) = product.old_price_minor {
        if old_price < product.price_minor {
            return false;
        }
    }
    if let Some(rate) = product.commission_rate_bps {
        if rate > SCORE_MAX {
            return false;
        }
    }
    if let Some(fixed) = product.commission_fixed_minor {
        if fixed < 0 {
            return false;
        }
        if let Some(currency) = product.commission_currency.as_deref() {
            if currency != product.currency {
                return false;
            }
        }
    }
    if let Some(refund) = product.refund_rate_bps {
        if refund > SCORE_MAX {
            return false;
        }
    }
    true
}

fn quality_assessment(product: &Product) -> QualityAssessment {
    let mut values = Vec::new();
    let mut weights = Vec::new();
    let mut reasons = Vec::new();

    if let Some(rating) = product.rating_bps {
        let adjusted_rating = if let Some(reviews) = product.review_count {
            let prior_rating = 8_000_u64;
            let prior_weight = 20_u64;
            let n = reviews.min(1_000_000);
            (((rating as u64).saturating_mul(n)
                + prior_rating.saturating_mul(prior_weight))
                / n.saturating_add(prior_weight))
                .min(SCORE_MAX as u64) as u32
        } else {
            rating
        };
        values.push(adjusted_rating as u64 * 50);
        weights.push(50_u64);
        if adjusted_rating != rating {
            reasons.push(format!(
                "Bayesian-adjusted rating={:.2}/5",
                adjusted_rating as f64 / 2_000.0
            ));
        } else {
            reasons.push(format!("provider rating={:.2}/5", rating as f64 / 2_000.0));
        }
    }
    if let Some(reviews) = product.review_count {
        let review_confidence = ((reviews.min(1_000) * SCORE_MAX as u64) / 1_000) as u32;
        values.push(review_confidence as u64 * 20);
        weights.push(20);
        reasons.push(format!("review evidence count={reviews}"));
    }
    let stock_score = if product.in_stock {
        if product.stock_quantity.unwrap_or(1) > 0 {
            10_000
        } else {
            7_000
        }
    } else {
        0
    };
    values.push(stock_score as u64 * 15);
    weights.push(15);
    reasons.push(
        if product.in_stock {
            "in stock"
        } else {
            "out of stock"
        }
        .into(),
    );

    if let Some(seller) = product.seller_reputation_bps {
        values.push(seller as u64 * 10);
        weights.push(10);
        reasons.push("seller reputation supplied by provider".into());
    }
    if let Some(refund) = product.refund_rate_bps {
        values.push((SCORE_MAX.saturating_sub(refund)) as u64 * 10);
        weights.push(10);
        reasons.push(format!("refund/cancel signal={refund} bps"));
    }
    if let Some(delivery) = product.delivery_reliability_bps {
        values.push(delivery as u64 * 5);
        weights.push(5);
        reasons.push("delivery reliability supplied by provider".into());
    }

    let score = if values.is_empty() || weights.iter().sum::<u64>() == 0 {
        0
    } else {
        (values.iter().sum::<u64>() / weights.iter().sum::<u64>()).min(SCORE_MAX as u64) as u32
    };

    let evidence_count = [
        product.rating_bps.is_some(),
        product.review_count.is_some(),
        product.seller_reputation_bps.is_some(),
        product.delivery_reliability_bps.is_some(),
        product.stock_quantity.is_some(),
    ]
    .into_iter()
    .filter(|v| *v)
    .count();
    let confidence_bps = (evidence_count as u32 * 2_000).min(SCORE_MAX);

    QualityAssessment {
        score_bps: score,
        confidence_bps,
        reasons,
    }
}

fn economics_assessment(product: &Product, coupons: &[Coupon]) -> EconomicsAssessment {
    let mut reasons = Vec::new();
    let expected_commission_minor = product
        .commission_rate_bps
        .and_then(|rate| product.price_minor.checked_mul(rate as i128))
        .map(|v| v / SCORE_MAX as i128)
        .or(product.commission_fixed_minor);
    let expected_net_commission_minor = expected_commission_minor.map(|value| {
        if let Some(refund) = product.refund_rate_bps {
            value.saturating_mul((SCORE_MAX - refund) as i128) / SCORE_MAX as i128
        } else {
            value
        }
    });

    let effective_discount_bps = coupons
        .iter()
        .filter_map(|coupon| coupon.discount_bps)
        .chain(product.savings_bps)
        .max();

    let commission_score = expected_net_commission_minor
        .map(|value| {
            let base = product.price_minor.max(1);
            ((value.max(0).saturating_mul(SCORE_MAX as i128) / base).min(SCORE_MAX as i128))
                as u32
        })
        .unwrap_or(0);
    let fixed_commission_score = product
        .commission_fixed_minor
        .map(|value| {
            let base = product.price_minor.max(1);
            ((value.max(0).saturating_mul(SCORE_MAX as i128) / base).min(SCORE_MAX as i128)) as u32
        })
        .unwrap_or(0);
    let coupon_bonus = effective_discount_bps.unwrap_or(0).min(SCORE_MAX);
    let score_bps = ((commission_score.max(fixed_commission_score) as u64 * 7
        + coupon_bonus as u64 * 3)
        / 10) as u32;

    if let Some(rate) = product.commission_rate_bps {
        reasons.push(format!("commission rate={rate} bps"));
    } else {
        reasons.push("commission rate unavailable from provider".into());
    }
    if let Some(discount) = effective_discount_bps {
        reasons.push(format!("discount/offer value={discount} bps"));
    } else {
        reasons.push("no verified active discount signal".into());
    }

    let confidence_bps = [
        product.commission_rate_bps.is_some() || product.commission_fixed_minor.is_some(),
        effective_discount_bps.is_some(),
        expected_commission_minor.is_some() && expected_net_commission_minor.is_some(),
    ]
    .into_iter()
    .filter(|v| *v)
    .count() as u32
        * 3_333;

    EconomicsAssessment {
        score_bps,
        confidence_bps: confidence_bps.min(SCORE_MAX),
        expected_commission_minor,
        expected_net_commission_minor,
        effective_discount_bps,
        reasons,
    }
}

fn content_fit(product: &Product, query: &ProductSearchQuery) -> u32 {
    if query.keywords.is_empty() && query.category.is_none() {
        return 5_000;
    }
    let haystack = format!(
        "{} {} {}",
        product.name, product.category, product.description
    )
    .to_ascii_lowercase();

    let mut score = 0_u32;
    let mut checks = 0_u32;
    if let Some(category) = query.category.as_deref() {
        checks += 1;
        if haystack.contains(&category.to_ascii_lowercase()) {
            score += 6_000;
        }
    }
    for keyword in &query.keywords {
        checks += 1;
        if haystack.contains(&keyword.to_ascii_lowercase()) {
            score += 4_000;
        }
    }
    if checks == 0 {
        5_000
    } else {
        (score / checks).min(SCORE_MAX)
    }
}

fn coupon_score(coupons: &[Coupon]) -> u32 {
    coupons
        .iter()
        .filter_map(|c| c.discount_bps)
        .max()
        .unwrap_or_else(|| {
            if coupons.iter().any(|c| c.code.is_some()) {
                3_000
            } else {
                0
            }
        })
        .min(SCORE_MAX)
}

fn confidence(
    product: &Product,
    quality: &QualityAssessment,
    economics: &EconomicsAssessment,
    freshness: &ProductFreshness,
) -> u32 {
    let source_signal = if product.source.trim().is_empty() { 0 } else { 1_000 };
    let freshness_signal: i64 = match freshness.status {
        ProductFreshnessStatus::Fresh => 1_000,
        ProductFreshnessStatus::Unknown => -2_000,
        ProductFreshnessStatus::Stale => -10_000,
    };
    let base =
        (quality.confidence_bps as i64 + economics.confidence_bps as i64 + source_signal + freshness_signal)
            / 3;
    base.clamp(0, SCORE_MAX as i64) as u32
}

fn weighted_score(
    quality: u32,
    commission: u32,
    coupon: u32,
    content_fit: u32,
    savings: u32,
    confidence: u32,
    weights: &RankingWeights,
) -> u32 {
    let total_weight = (weights.quality_bps
        + weights.commission_bps
        + weights.coupon_bps
        + weights.content_fit_bps
        + weights.savings_bps
        + weights.evidence_confidence_bps) as u64;

    if total_weight == 0 {
        return 0;
    }

    let numerator = quality as u64 * weights.quality_bps as u64
        + commission as u64 * weights.commission_bps as u64
        + coupon as u64 * weights.coupon_bps as u64
        + content_fit as u64 * weights.content_fit_bps as u64
        + savings.min(SCORE_MAX) as u64 * weights.savings_bps as u64
        + confidence as u64 * weights.evidence_confidence_bps as u64;

    (numerator / total_weight).min(SCORE_MAX as u64) as u32
}

fn dedupe_key(product: &Product) -> String {
    if let Some(gtin) = product.gtin.as_deref().filter(|v| !v.is_empty()) {
        return format!("gtin:{gtin}");
    }
    format!("{}:{}", product.advertiser_id, product.id)
}

fn product_freshness(
    product: &Product,
    as_of_date: Option<&str>,
) -> ProductFreshness {
    let Some(raw_updated_at) = product.source_updated_at.as_deref() else {
        return ProductFreshness {
            status: ProductFreshnessStatus::Unknown,
            age_days: None,
            max_age_days: DEFAULT_MAX_PRODUCT_AGE_DAYS,
        };
    };
    let Some(updated_date) = parse_iso_date(date_prefix(raw_updated_at).as_str()) else {
        return ProductFreshness {
            status: ProductFreshnessStatus::Stale,
            age_days: None,
            max_age_days: DEFAULT_MAX_PRODUCT_AGE_DAYS,
        };
    };

    let reference_date = as_of_date
        .and_then(parse_iso_date)
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());
    let age_days = (reference_date - updated_date).whole_days();
    let status = if age_days < 0 || age_days > DEFAULT_MAX_PRODUCT_AGE_DAYS {
        ProductFreshnessStatus::Stale
    } else {
        ProductFreshnessStatus::Fresh
    };

    ProductFreshness {
        status,
        age_days: Some(age_days),
        max_age_days: DEFAULT_MAX_PRODUCT_AGE_DAYS,
    }
}

fn parse_iso_date(value: &str) -> Option<time::Date> {
    if !is_iso_date(value) {
        return None;
    }
    let bytes = value.as_bytes();
    let year = i32::from(bytes.get(0).copied()? - b'0') * 1000
        + i32::from(bytes.get(1).copied()? - b'0') * 100
        + i32::from(bytes.get(2).copied()? - b'0') * 10
        + i32::from(bytes.get(3).copied()? - b'0');
    let month = i32::from(bytes.get(5).copied()? - b'0') * 10
        + i32::from(bytes.get(6).copied()? - b'0');
    let day = i32::from(bytes.get(8).copied()? - b'0') * 10
        + i32::from(bytes.get(9).copied()? - b'0');
    let month = match month {
        1 => time::Month::January,
        2 => time::Month::February,
        3 => time::Month::March,
        4 => time::Month::April,
        5 => time::Month::May,
        6 => time::Month::June,
        7 => time::Month::July,
        8 => time::Month::August,
        9 => time::Month::September,
        10 => time::Month::October,
        11 => time::Month::November,
        12 => time::Month::December,
        _ => return None,
    };
    time::Date::from_calendar_date(year, month, day).ok()
}

fn coupon_is_active_on(coupon: &Coupon, as_of_date: Option<&str>) -> bool {
    if !coupon.active || !valid_coupon_window(coupon) {
        return false;
    }
    let owned_today;
    let today = if let Some(value) = as_of_date {
        value
    } else {
        owned_today = time::OffsetDateTime::now_utc().date().to_string();
        owned_today.as_str()
    };
    if let Some(start) = coupon.starts_at.as_deref().map(date_prefix) {
        if today < start.as_str() {
            return false;
        }
    }
    if let Some(end) = coupon.ends_at.as_deref().map(date_prefix) {
        if today > end.as_str() {
            return false;
        }
    }
    true
}

fn valid_coupon_window(coupon: &Coupon) -> bool {
    let start = coupon.starts_at.as_deref().map(date_prefix);
    let end = coupon.ends_at.as_deref().map(date_prefix);
    if let Some(value) = start.as_deref() {
        if !is_iso_date(value) {
            return false;
        }
    }
    if let Some(value) = end.as_deref() {
        if !is_iso_date(value) {
            return false;
        }
    }
    match (start, end) {
        (Some(start), Some(end)) => start <= end,
        _ => true,
    }
}

fn date_prefix(value: &str) -> String {
    value.chars().take(10).collect()
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.len() == 10
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit())
}

pub struct MockProvider {
    products: Vec<Product>,
    coupons: Vec<Coupon>,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            products: vec![
                Product {
                    id: "mock:phone-1".into(),
                    gtin: Some("0000000000001".into()),
                    advertiser_id: "100".into(),
                    advertiser_name: Some("Mock Electronics".into()),
                    name: "Creator Phone Pro".into(),
                    description: "phone camera for creator video".into(),
                    category: "electronics".into(),
                    brand: Some("MockBrand".into()),
                    url: "https://example.com/phone-1".into(),
                    image_url: None,
                    price_minor: 2_990_000,
                    old_price_minor: Some(3_490_000),
                    currency: "VND".into(),
                    rating_bps: Some(9_200),
                    review_count: Some(4_800),
                    stock_quantity: Some(120),
                    in_stock: true,
                    savings_bps: Some(1_433),
                    seller_reputation_bps: Some(9_100),
                    refund_rate_bps: Some(300),
                    delivery_reliability_bps: Some(9_000),
                    commission_group: Some("premium".into()),
                    commission_rate_bps: Some(1_800),
                    commission_fixed_minor: None,
                    commission_currency: None,
                    source: "mock".into(),
                    source_updated_at: Some("2026-09-27T00:00:00Z".into()),
                },
                Product {
                    id: "mock:phone-2".into(),
                    gtin: Some("0000000000002".into()),
                    advertiser_id: "200".into(),
                    advertiser_name: Some("Mock Deal Store".into()),
                    name: "High Commission Camera Phone".into(),
                    description: "budget creator phone".into(),
                    category: "electronics".into(),
                    brand: Some("DealBrand".into()),
                    url: "https://example.com/phone-2".into(),
                    image_url: None,
                    price_minor: 2_390_000,
                    old_price_minor: Some(2_890_000),
                    currency: "VND".into(),
                    rating_bps: Some(7_200),
                    review_count: Some(180),
                    stock_quantity: Some(40),
                    in_stock: true,
                    savings_bps: Some(1_730),
                    seller_reputation_bps: Some(7_000),
                    refund_rate_bps: Some(1_500),
                    delivery_reliability_bps: Some(7_000),
                    commission_group: Some("very-high".into()),
                    commission_rate_bps: Some(4_500),
                    commission_fixed_minor: None,
                    commission_currency: None,
                    source: "mock".into(),
                    source_updated_at: Some("2026-09-27T00:00:00Z".into()),
                },
            ],
            coupons: vec![Coupon {
                id: "mock:coupon-1".into(),
                advertiser_id: "100".into(),
                title: "10% off creator phone".into(),
                description: "Active voucher".into(),
                code: Some("CREATOR10".into()),
                discount_bps: Some(1_000),
                starts_at: Some("2026-09-01".into()),
                ends_at: Some("2026-12-31".into()),
                active: true,
                exclusive: true,
                attributable: true,
                url: Some("https://example.com/offer".into()),
                source: "mock".into(),
            }],
        }
    }
}

#[async_trait]
impl AffiliateProvider for MockProvider {
    fn name(&self) -> &'static str {
        "mock"
    }
    async fn products(&self) -> Result<Vec<Product>, AffiliateError> {
        Ok(self.products.clone())
    }
    async fn coupons(&self) -> Result<Vec<Coupon>, AffiliateError> {
        Ok(self.coupons.clone())
    }
}

#[derive(Debug, Clone)]
struct Cache<T> {
    loaded_at: Instant,
    value: T,
}

pub struct AwinProvider {
    client: Client,
    product_feed_url: Option<String>,
    product_feed_api_key: Option<String>,
    feed_id: Option<String>,
    publisher_id: String,
    access_token: String,
    commission_map: HashMap<String, u32>,
    auto_fetch_commissions: bool,
    max_commission_advertisers: usize,
    cache_ttl: Duration,
    products_cache: RwLock<Option<Cache<Vec<Product>>>>,
    coupons_cache: RwLock<Option<Cache<Vec<Coupon>>>>,
    minor_units: u32,
}

impl AwinProvider {
    pub fn from_env() -> Result<Self, AffiliateError> {
        let product_feed_url = std::env::var("AWIN_PRODUCT_FEED_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(|v| v.trim().to_owned());
        let product_feed_api_key = secret_from_env_optional("AWIN_PRODUCT_FEED_API_KEY")?;
        if product_feed_url.is_none() {
            return Err(AffiliateError::Provider(
                "AWIN_PRODUCT_FEED_URL is required; feed discovery is intentionally fail-closed".into(),
            ));
        }
        let feed_id = std::env::var("AWIN_FEED_ID")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(|v| v.trim().to_owned());
        let publisher_id = std::env::var("AWIN_PUBLISHER_ID")
            .map_err(|_| AffiliateError::Provider("AWIN_PUBLISHER_ID is required".into()))?;
        let access_token = secret_from_env("AWIN_ACCESS_TOKEN")
            .map_err(|_| AffiliateError::Provider("AWIN_ACCESS_TOKEN is required".into()))?;
        let auto_fetch_commissions = std::env::var("AWIN_AUTO_FETCH_COMMISSIONS")
            .ok()
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(true);
        let max_commission_advertisers = std::env::var("AWIN_MAX_COMMISSION_ADVERTISERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .map(|v| v.clamp(1, 15))
            .unwrap_or(8);
        let cache_ttl = std::env::var("AFFILIATE_CACHE_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs)
            .filter(|v| v.as_secs() > 0)
            .unwrap_or(Duration::from_secs(900));
        let minor_units = std::env::var("AFFILIATE_MINOR_UNITS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(2)
            .min(6);
        Ok(Self {
            client: Client::builder()
                .timeout(DEFAULT_HTTP_TIMEOUT)
                .user_agent("company-agents-affiliate/0.1")
                .build()
                .map_err(|e| AffiliateError::Provider(e.to_string()))?,
            product_feed_url,
            product_feed_api_key,
            feed_id,
            publisher_id,
            access_token,
            commission_map: parse_commission_map(
                &std::env::var("AWIN_COMMISSION_MAP").unwrap_or_default(),
            ),
            auto_fetch_commissions,
            max_commission_advertisers,
            cache_ttl,
            products_cache: RwLock::new(None),
            coupons_cache: RwLock::new(None),
            minor_units,
        })
    }

    async fn discover_feed_url(&self) -> Result<String, AffiliateError> {
        if let Some(url) = self.product_feed_url.clone() {
            return Ok(url);
        }
        if let Some(feed_id) = self.feed_id.as_deref().filter(|v| v.starts_with("https://")) {
            return Ok(feed_id.to_owned());
        }
        let _ = &self.product_feed_api_key;
        Err(AffiliateError::Provider(
            "no safe feed URL is configured".into(),
        ))
    }

    async fn refresh_products(&self) -> Result<Vec<Product>, AffiliateError> {
        let feed_url = match self.product_feed_url.as_deref() {
            Some(url) => url.to_owned(),
            None => self.discover_feed_url().await?,
        };

        let mut response = None;
        for attempt in 0..3_u32 {
            let candidate = self
                .client
                .get(&feed_url)
                .send()
                .await
                .map_err(|e| AffiliateError::Provider(e.to_string()))?;
            if (candidate.status().as_u16() == 429 || candidate.status().is_server_error())
                && attempt < 2
            {
                tokio::time::sleep(Duration::from_millis(250 * (1_u64 << attempt))).await;
                continue;
            }
            response = Some(candidate);
            break;
        }
        let response = response.ok_or_else(|| {
            AffiliateError::Provider("affiliate feed retry loop exhausted".into())
        })?;
        if !response.status().is_success() {
            return Err(AffiliateError::Provider(format!(
                "Awin product feed HTTP {}",
                response.status()
            )));
        }
        if let Some(length) = response.content_length() {
            if length as usize > MAX_FEED_BYTES {
                return Err(AffiliateError::PayloadTooLarge);
            }
        }
        let mut body = response
            .bytes()
            .await
            .map_err(|e| AffiliateError::Provider(e.to_string()))?
            .to_vec();
        if body.len() > MAX_FEED_BYTES {
            return Err(AffiliateError::PayloadTooLarge);
        }
        if body.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = flate2::read::GzDecoder::new(body.as_slice());
            let mut decompressed = Vec::new();
            std::io::Read::read_to_end(&mut decoder, &mut decompressed)
                .map_err(|e| AffiliateError::Parse(format!("gzip feed decode failed: {e}")))?;
            if decompressed.len() > MAX_FEED_BYTES {
                return Err(AffiliateError::PayloadTooLarge);
            }
            body = decompressed;
        }
        let delimiter = detect_delimiter(&body);
        let mut products =
            parse_awin_feed(&body, delimiter, self.minor_units, &self.commission_map)?;
        if self.auto_fetch_commissions {
            self.enrich_commission_rates(&mut products).await?;
        }
        Ok(products)
    }

    async fn enrich_commission_rates(
        &self,
        products: &mut [Product],
    ) -> Result<(), AffiliateError> {
        let mut advertisers = Vec::new();
        for product in products.iter() {
            if product.commission_rate_bps.is_none()
                && product.commission_fixed_minor.is_none()
                && !advertisers
                    .iter()
                    .any(|v: &String| v == &product.advertiser_id)
            {
                advertisers.push(product.advertiser_id.clone());
                if advertisers.len() >= self.max_commission_advertisers {
                    break;
                }
            }
        }

        for advertiser_id in advertisers {
            let url = format!(
                "https://api.awin.com/publishers/{}/commissiongroups",
                self.publisher_id
            );
            let mut response = None;
            for attempt in 0..3_u32 {
                let candidate = self
                    .client
                    .get(&url)
                    .query(&[
                        ("accessToken", self.access_token.as_str()),
                        ("advertiserId", advertiser_id.as_str()),
                    ])
                    .bearer_auth(&self.access_token)
                    .send()
                    .await
                    .map_err(|e| AffiliateError::Provider(e.to_string()))?;
                if (candidate.status().as_u16() == 429 || candidate.status().is_server_error())
                    && attempt < 2
                {
                    tokio::time::sleep(Duration::from_millis(250 * (1_u64 << attempt))).await;
                    continue;
                }
                response = Some(candidate);
                break;
            }
            let response = response.ok_or_else(|| {
                AffiliateError::Provider("Awin commission-group retry loop exhausted".into())
            })?;
            if !response.status().is_success() {
                return Err(AffiliateError::Provider(format!(
                    "Awin commission groups HTTP {}",
                    response.status()
                )));
            }

            let value = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| AffiliateError::Parse(e.to_string()))?;
            let groups = parse_awin_commission_groups(&value)?;
            for product in products
                .iter_mut()
                .filter(|p| p.advertiser_id == advertiser_id)
            {
                if product.commission_rate_bps.is_some() || product.commission_fixed_minor.is_some()
                {
                    continue;
                }
                let rate = match product.commission_group.as_deref() {
                    Some(group_code) => groups
                        .iter()
                        .find(|g| g.code.eq_ignore_ascii_case(group_code)),
                    None => groups.iter().find(|g| g.is_default).or_else(|| {
                        if groups.len() == 1 {
                            groups.first()
                        } else {
                            None
                        }
                    }),
                };
                if let Some(group) = rate {
                    if let Some(bps) = group.percentage_bps {
                        product.commission_rate_bps = Some(bps);
                    }
                    if let Some(amount) = group.fixed_amount {
                        let commission_currency =
                            group.currency.as_deref().unwrap_or(&product.currency);
                        if commission_currency.eq_ignore_ascii_case(&product.currency) {
                            let units =
                                minor_units_for_currency(commission_currency, self.minor_units);
                            let scale = 10_i128.pow(units);
                            let fixed_minor = (amount * scale as f64).round();
                            if fixed_minor.is_finite()
                                && fixed_minor >= 0.0
                                && fixed_minor <= i128::MAX as f64
                            {
                                product.commission_fixed_minor = Some(fixed_minor as i128);
                                product.commission_currency =
                                    Some(commission_currency.to_ascii_uppercase());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn refresh_coupons(&self) -> Result<Vec<Coupon>, AffiliateError> {
        let mut page = 1_u32;
        let mut all = Vec::new();
        loop {
            let url = format!(
                "https://api.awin.com/publisher/{}/promotions?accessToken={}",
                self.publisher_id, self.access_token
            );
            let payload = serde_json::json!({
                "filters": {
                    "status": "active",
                    "type": "all",
                    "membership": "joined"
                },
                "pagination": {
                    "page": page,
                    "pageSize": 200
                }
            });
            let mut response = None;
            for attempt in 0..3_u32 {
                let candidate = self
                    .client
                    .post(&url)
                    .bearer_auth(&self.access_token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| AffiliateError::Provider(e.to_string()))?;
                if (candidate.status().as_u16() == 429 || candidate.status().is_server_error())
                    && attempt < 2
                {
                    tokio::time::sleep(Duration::from_millis(250 * (1_u64 << attempt))).await;
                    continue;
                }
                response = Some(candidate);
                break;
            }
            let response = response.ok_or_else(|| {
                AffiliateError::Provider("Awin offer retry loop exhausted".into())
            })?;
            if !response.status().is_success() {
                return Err(AffiliateError::Provider(format!(
                    "Awin offers HTTP {}",
                    response.status()
                )));
            }
            let value = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| AffiliateError::Parse(e.to_string()))?;
            let page_items = parse_awin_offers(&value)?;
            let count = page_items.len();
            all.extend(page_items);
            if count < 200 {
                break;
            }
            page += 1;
            if page > 500 {
                return Err(AffiliateError::Provider(
                    "offer pagination safety limit exceeded".into(),
                ));
            }
        }
        Ok(all)
    }
}

#[async_trait]
impl AffiliateProvider for AwinProvider {
    fn name(&self) -> &'static str {
        "awin"
    }

    async fn products(&self) -> Result<Vec<Product>, AffiliateError> {
        {
            let cache = self.products_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.loaded_at.elapsed() < self.cache_ttl {
                    return Ok(cached.value.clone());
                }
            }
        }
        let fresh = self.refresh_products().await?;
        *self.products_cache.write().await = Some(Cache {
            loaded_at: Instant::now(),
            value: fresh.clone(),
        });
        Ok(fresh)
    }

    async fn coupons(&self) -> Result<Vec<Coupon>, AffiliateError> {
        {
            let cache = self.coupons_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.loaded_at.elapsed() < self.cache_ttl {
                    return Ok(cached.value.clone());
                }
            }
        }
        let fresh = self.refresh_coupons().await?;
        *self.coupons_cache.write().await = Some(Cache {
            loaded_at: Instant::now(),
            value: fresh.clone(),
        });
        Ok(fresh)
    }
}

fn secret_from_env(name: &str) -> Result<String, AffiliateError> {
    let file_key = format!("{name}_FILE");
    let direct = std::env::var(name).ok();
    let file = std::env::var(&file_key).ok();

    if direct.is_some() && file.is_some() {
        return Err(AffiliateError::Provider(format!(
            "{name} and {file_key} must not both be set"
        )));
    }

    let value = match (direct, file) {
        (Some(value), None) => value,
        (None, Some(path)) => {
            if path.trim().is_empty() || std::path::Path::new(&path).is_dir() {
                return Err(AffiliateError::Provider(format!("{file_key} is invalid")));
            }
            let metadata = std::fs::metadata(&path)
                .map_err(|_| AffiliateError::Provider(format!("{file_key} cannot be read")))?;
            if metadata.len() > 16 * 1024 {
                return Err(AffiliateError::Provider(format!("{file_key} is too large")));
            }
            std::fs::read_to_string(&path)
                .map_err(|_| AffiliateError::Provider(format!("{file_key} cannot be read")))?
        }
        (None, None) => {
            return Err(AffiliateError::Provider(format!("{name} is required")));
        }
        _ => unreachable!(),
    };

    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(AffiliateError::Provider(format!("{name} is empty")));
    }
    Ok(value)
}

fn secret_from_env_optional(name: &str) -> Result<Option<String>, AffiliateError> {
    let direct = std::env::var(name).ok();
    let file = std::env::var(format!("{name}_FILE")).ok();
    if direct.is_none() && file.is_none() {
        return Ok(None);
    }
    secret_from_env(name).map(Some)
}

fn parse_commission_map(value: &str) -> HashMap<String, u32> {
    let mut result = HashMap::new();
    for item in value.split(',').map(str::trim).filter(|v| !v.is_empty()) {
        let Some((key, raw)) = item.split_once('=') else {
            continue;
        };
        if let Ok(bps) = raw.trim().parse::<u32>() {
            result.insert(key.trim().to_owned(), bps.min(SCORE_MAX));
        }
    }
    result
}

fn detect_delimiter(body: &[u8]) -> u8 {
    let first_line_end = body.iter().position(|b| *b == b'\n').unwrap_or(body.len());
    let line = &body[..first_line_end];
    let comma = line.iter().filter(|b| **b == b',').count();
    let tab = line.iter().filter(|b| **b == b'\t').count();
    if tab > comma {
        b'\t'
    } else {
        b','
    }
}

fn parse_awin_feed(
    body: &[u8],
    delimiter: u8,
    minor_units: u32,
    commission_map: &HashMap<String, u32>,
) -> Result<Vec<Product>, AffiliateError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(body);

    let headers = reader
        .headers()
        .map_err(|e| AffiliateError::Parse(e.to_string()))?
        .iter()
        .map(|h| h.trim_start_matches('\u{feff}').to_ascii_lowercase())
        .collect::<Vec<_>>();

    let mut products = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| AffiliateError::Parse(e.to_string()))?;
        let get = |name: &str| field(&headers, &row, name);
        let id = first_nonempty(&[
            get("aw_product_id"),
            get("product_id"),
            get("merchant_product_id"),
        ])
        .ok_or_else(|| AffiliateError::Parse("product row missing product id".into()))?;
        let advertiser_id = first_nonempty(&[get("merchant_id"), get("advertiser_id")])
            .unwrap_or_else(|| "unknown".into());
        let name = first_nonempty(&[get("product_name"), get("name")])
            .unwrap_or_else(|| "Unnamed product".into());
        let price_raw = first_nonempty(&[
            get("search_price"),
            get("store_price"),
            get("base_price_amount"),
        ]);
        let currency = first_nonempty(&[get("currency")]).unwrap_or_else(|| "USD".into());

        let Some(price_raw) = price_raw else { continue };
        let row_minor_units = minor_units_for_currency(&currency, minor_units);
        let price_minor = parse_decimal_minor(&price_raw, row_minor_units)?;
        if price_minor < 0 {
            continue;
        }

        let category = first_nonempty(&[
            get("category_name"),
            get("merchant_category"),
            get("merchant_product_category_path"),
            get("product_type"),
        ])
        .unwrap_or_default();

        let rating_bps = get("average_rating")
            .or_else(|| get("rating"))
            .and_then(|v| parse_rating_bps(&v));
        let review_count = get("reviews").and_then(|v| v.parse::<u64>().ok());
        let stock_quantity = first_nonempty(&[get("number_available"), get("stock_quantity")])
            .and_then(|v| v.parse::<u64>().ok());
        let in_stock = first_nonempty(&[get("in_stock"), get("is_for_sale"), get("stock_status")])
            .map(|v| {
                let value = v.to_ascii_lowercase();
                match value.as_str() {
                    "0" | "false" | "no" | "out" | "outofstock" | "unavailable" => false,
                    "1" | "true" | "yes" | "in" | "instock" | "available" => true,
                    _ => stock_quantity.unwrap_or(0) > 0,
                }
            })
            .unwrap_or_else(|| stock_quantity.unwrap_or(1) > 0);
        let savings_bps = first_nonempty(&[get("savings_percent"), get("saving")])
            .and_then(|v| parse_percent_bps(&v));

        let commission_group = get("commission_group");
        let commission_rate_bps = commission_group
            .as_deref()
            .and_then(|group| {
                commission_map
                    .get(&format!("{advertiser_id}:{group}"))
                    .copied()
            })
            .or_else(|| commission_map.get(&advertiser_id).copied());

        products.push(Product {
            id,
            gtin: first_nonempty(&[
                get("product_GTIN"),
                get("product_gtin"),
                get("ean"),
                get("upc"),
            ]),
            advertiser_id,
            advertiser_name: get("merchant_name"),
            name,
            description: first_nonempty(&[get("description"), get("product_short_description")])
                .unwrap_or_default(),
            category,
            brand: get("brand_name"),
            url: first_nonempty(&[get("aw_deep_link"), get("merchant_deep_link")])
                .unwrap_or_default(),
            image_url: first_nonempty(&[
                get("aw_image_url"),
                get("merchant_image_url"),
                get("large_image"),
            ]),
            price_minor,
            old_price_minor: first_nonempty(&[get("product_price_old"), get("rrp_price")])
                .and_then(|v| parse_decimal_minor(&v, row_minor_units).ok()),
            currency,
            rating_bps,
            review_count,
            stock_quantity,
            in_stock,
            savings_bps,
            seller_reputation_bps: None,
            refund_rate_bps: None,
            delivery_reliability_bps: None,
            commission_group,
            commission_rate_bps,
            commission_fixed_minor: None,
            commission_currency: None,
            source: "awin_product_feed".into(),
            source_updated_at: first_nonempty(&[get("last_updated"), get("valid_from")]),
        });
    }
    Ok(products)
}

fn field(headers: &[String], row: &csv::StringRecord, name: &str) -> Option<String> {
    let index = headers.iter().position(|h| h == name)?;
    row.get(index)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

fn first_nonempty(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .find_map(|v| v.clone().filter(|s| !s.trim().is_empty()))
}

fn parse_decimal_minor(value: &str, minor_units: u32) -> Result<i128, AffiliateError> {
    let normalized = value.trim().replace(',', "");
    let negative = normalized.starts_with('-');
    let unsigned = normalized.trim_start_matches('-');
    let mut pieces = unsigned.split('.');
    let whole = pieces.next().unwrap_or("0");
    let fractional = pieces.next().unwrap_or("");
    if pieces.next().is_some()
        || !whole.chars().all(|c| c.is_ascii_digit())
        || !fractional.chars().all(|c| c.is_ascii_digit())
    {
        return Err(AffiliateError::Parse(format!(
            "invalid decimal price: {value}"
        )));
    }
    let scale = 10_i128.pow(minor_units);
    let whole_value = whole
        .parse::<i128>()
        .map_err(|_| AffiliateError::Parse("price overflow".into()))?;
    let mut fraction = fractional.to_owned();
    if fraction.len() > minor_units as usize {
        if fraction.as_bytes()[minor_units as usize..]
            .iter()
            .any(|b| *b != b'0')
        {
            return Err(AffiliateError::Parse(format!(
                "price has more precision than currency allows: {value}"
            )));
        }
        fraction.truncate(minor_units as usize);
    }
    while fraction.len() < minor_units as usize {
        fraction.push('0');
    }
    let fractional_value = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<i128>()
            .map_err(|_| AffiliateError::Parse("price overflow".into()))?
    };
    let result = whole_value
        .checked_mul(scale)
        .and_then(|v| v.checked_add(fractional_value))
        .ok_or_else(|| AffiliateError::Parse("price overflow".into()))?;
    Ok(if negative { -result } else { result })
}

fn minor_units_for_currency(currency: &str, fallback: u32) -> u32 {
    match currency.to_ascii_uppercase().as_str() {
        "VND" | "JPY" | "KRW" | "IDR" => 0,
        "BHD" | "JOD" | "KWD" | "OMR" => 3,
        _ => fallback.min(6),
    }
}

fn parse_rating_bps(value: &str) -> Option<u32> {
    let raw = value.trim().parse::<f64>().ok()?;
    if !raw.is_finite() || raw < 0.0 {
        return None;
    }
    if raw <= 5.0 {
        Some(((raw / 5.0) * SCORE_MAX as f64).round() as u32)
    } else if raw <= 100.0 {
        Some((raw * 100.0).round().min(SCORE_MAX as f64) as u32)
    } else {
        Some(raw.round().min(SCORE_MAX as f64) as u32)
    }
}

fn parse_percent_bps(value: &str) -> Option<u32> {
    let raw = value.trim().trim_end_matches('%').parse::<f64>().ok()?;
    if !raw.is_finite() || raw < 0.0 {
        return None;
    }
    let bps = if raw <= 1.0 {
        raw * SCORE_MAX as f64
    } else if raw <= 100.0 {
        raw * 100.0
    } else {
        raw
    };
    Some(bps.round().min(SCORE_MAX as f64) as u32)
}

#[derive(Debug, Deserialize)]
struct AwinOffer {
    #[serde(default, rename = "promotionId")]
    promotion_id: Option<serde_json::Value>,
    advertiser: Option<AwinAdvertiser>,
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "startDate")]
    start_date: Option<String>,
    #[serde(rename = "endDate")]
    end_date: Option<String>,
    url: Option<String>,
    #[serde(rename = "urlTracking")]
    url_tracking: Option<String>,
    voucher: Option<AwinVoucher>,
}

#[derive(Debug, Deserialize)]
struct AwinAdvertiser {
    id: Option<u64>,
}

#[derive(Debug, Clone)]
struct ParsedCommissionGroup {
    code: String,
    is_default: bool,
    percentage_bps: Option<u32>,
    fixed_amount: Option<f64>,
    currency: Option<String>,
}

fn parse_awin_commission_groups(
    value: &serde_json::Value,
) -> Result<Vec<ParsedCommissionGroup>, AffiliateError> {
    let groups_value = match value {
        serde_json::Value::Array(items) => serde_json::Value::Array(items.clone()),
        serde_json::Value::Object(map) => map
            .get("commissionGroups")
            .cloned()
            .or_else(|| {
                map.get("data")
                    .and_then(|d| d.get("commissionGroups"))
                    .cloned()
            })
            .ok_or_else(|| {
                AffiliateError::Parse(
                    "Awin commission-group response missing commissionGroups".into(),
                )
            })?,
        _ => {
            return Err(AffiliateError::Parse(
                "Awin commission-group response must be an array/object".into(),
            ))
        }
    };
    let items = groups_value
        .as_array()
        .ok_or_else(|| AffiliateError::Parse("Awin commissionGroups must be an array".into()))?;
    let mut out = Vec::new();
    for item in items {
        let code = item
            .get("groupCode")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim()
            .to_owned();
        if code.is_empty() {
            continue;
        }
        let name = item
            .get("groupName")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let kind = item
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let percentage_bps = if kind.eq_ignore_ascii_case("percentage") {
            item.get("percentage")
                .and_then(|v| v.as_f64())
                .filter(|v| v.is_finite() && *v >= 0.0)
                .map(|v| (v * 100.0).round().min(SCORE_MAX as f64) as u32)
        } else {
            None
        };
        let fixed_amount = if kind.eq_ignore_ascii_case("fixed") {
            item.get("amount")
                .or_else(|| item.get("fixedAmount"))
                .and_then(|v| v.as_f64())
                .filter(|v| v.is_finite() && *v >= 0.0)
        } else {
            None
        };
        let currency = item
            .get("currency")
            .and_then(|v| v.as_str())
            .map(|v| v.to_ascii_uppercase());
        let is_default =
            code.eq_ignore_ascii_case("default") || name.to_ascii_lowercase().contains("default");
        out.push(ParsedCommissionGroup {
            code,
            is_default,
            percentage_bps,
            fixed_amount,
            currency,
        });
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
struct AwinVoucher {
    code: Option<String>,
    exclusive: Option<bool>,
    attributable: Option<bool>,
}

fn parse_awin_offers(value: &serde_json::Value) -> Result<Vec<Coupon>, AffiliateError> {
    let items = match value {
        serde_json::Value::Array(items) => items.clone(),
        serde_json::Value::Object(map) => {
            for key in ["data", "offers", "results"] {
                if let Some(serde_json::Value::Array(items)) = map.get(key) {
                    return parse_offer_array(items);
                }
            }
            return Err(AffiliateError::Parse(
                "Awin offer response did not contain an array".into(),
            ));
        }
        _ => {
            return Err(AffiliateError::Parse(
                "Awin offer response must be an array/object".into(),
            ))
        }
    };
    parse_offer_array(&items)
}

fn parse_offer_array(items: &[serde_json::Value]) -> Result<Vec<Coupon>, AffiliateError> {
    let mut out = Vec::new();
    for item in items {
        let raw: AwinOffer = serde_json::from_value(item.clone())
            .map_err(|e| AffiliateError::Parse(e.to_string()))?;
        let advertiser_id = raw
            .advertiser
            .as_ref()
            .and_then(|v| v.id)
            .map(|v| v.to_string());
        let Some(advertiser_id) = advertiser_id else {
            continue;
        };
        let title = raw.title.unwrap_or_else(|| "Awin promotion".into());
        let description = raw.description.unwrap_or_default();
        let id = raw
            .promotion_id
            .and_then(|v| v.as_i64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("{advertiser_id}:{title}"));
        let code = raw.voucher.as_ref().and_then(|v| v.code.clone());
        let discount_bps = parse_percent_from_text(&format!("{title} {description}"));
        out.push(Coupon {
            id,
            advertiser_id,
            title,
            description,
            code,
            discount_bps,
            starts_at: raw.start_date,
            ends_at: raw.end_date,
            active: true,
            exclusive: raw
                .voucher
                .as_ref()
                .and_then(|v| v.exclusive)
                .unwrap_or(false),
            attributable: raw
                .voucher
                .as_ref()
                .and_then(|v| v.attributable)
                .unwrap_or(false),
            url: raw.url_tracking.or(raw.url),
            source: "awin_offers_api".into(),
        });
    }
    Ok(out)
}

fn parse_percent_from_text(value: &str) -> Option<u32> {
    let bytes = value.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'%' {
            let mut start = i;
            while start > 0 && bytes[start - 1].is_ascii_digit() {
                start -= 1;
            }
            if start < i {
                if let Ok(percent) = value[start..i].parse::<u32>() {
                    return Some(percent.min(100) * 100);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product(
        id: &str,
        name: &str,
        commission: Option<u32>,
        rating: Option<u32>,
        reviews: Option<u64>,
    ) -> Product {
        Product {
            id: id.into(),
            gtin: None,
            advertiser_id: id.into(),
            advertiser_name: None,
            name: name.into(),
            description: "creator electronics".into(),
            category: "electronics".into(),
            brand: None,
            url: "https://example.com".into(),
            image_url: None,
            price_minor: 1_000,
            old_price_minor: Some(1_200),
            currency: "USD".into(),
            rating_bps: rating,
            review_count: reviews,
            stock_quantity: Some(100),
            in_stock: true,
            savings_bps: Some(1_000),
            seller_reputation_bps: None,
            refund_rate_bps: None,
            delivery_reliability_bps: None,
            commission_group: None,
            commission_rate_bps: commission,
            commission_fixed_minor: None,
            commission_currency: None,
            source: "test".into(),
            source_updated_at: Some("2026-09-27T00:00:00Z".into()),
        }
    }

    #[test]
    fn high_commission_does_not_automatically_outvote_quality() {
        let quality = product(
            "quality",
            "Quality Phone",
            Some(1_500),
            Some(9_600),
            Some(5_000),
        );
        let high_commission = Product {
            id: "commission".into(),
            advertiser_id: "commission".into(),
            gtin: None,
            advertiser_name: None,
            name: "Commission Phone".into(),
            description: "creator electronics".into(),
            category: "electronics".into(),
            brand: None,
            url: "https://example.com".into(),
            image_url: None,
            price_minor: 1_000,
            old_price_minor: None,
            currency: "USD".into(),
            rating_bps: Some(6_200),
            review_count: Some(8),
            stock_quantity: Some(20),
            in_stock: true,
            savings_bps: None,
            seller_reputation_bps: Some(5_500),
            refund_rate_bps: Some(2_500),
            delivery_reliability_bps: Some(5_000),
            commission_group: None,
            commission_rate_bps: Some(6_000),
            commission_fixed_minor: None,
            commission_currency: None,
            source: "test".into(),
            source_updated_at: Some("2026-09-27T00:00:00Z".into()),
        };
        let coupon = Coupon {
            id: "quality-coupon".into(),
            advertiser_id: "quality".into(),
            title: "10% off".into(),
            description: "verified active".into(),
            code: Some("SAVE10".into()),
            discount_bps: Some(1_000),
            starts_at: Some("2026-09-01".into()),
            ends_at: Some("2026-12-31".into()),
            active: true,
            exclusive: false,
            attributable: true,
            url: None,
            source: "test".into(),
        };
        let query = ProductSearchQuery {
            category: Some("electronics".into()),
            currency: Some("USD".into()),
            require_coupon: true,
            as_of_date: Some("2026-09-27".into()),
            ..Default::default()
        };
        let ranked = rank_products(&[quality, high_commission], &[coupon], &query);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].product.id, "quality");
    }

    #[test]
    fn quality_test_really_compares_two_eligible_products() {
        let quality = product(
            "quality",
            "Quality Phone",
            Some(1_500),
            Some(9_600),
            Some(5_000),
        );
        let mut high = product(
            "high",
            "High Commission Phone",
            Some(6_000),
            Some(6_200),
            Some(8),
        );
        high.seller_reputation_bps = Some(5_500);
        high.refund_rate_bps = Some(2_500);
        let coupons = vec![
            Coupon {
                id: "q".into(),
                advertiser_id: "quality".into(),
                title: "10% off".into(),
                description: "active".into(),
                code: Some("Q10".into()),
                discount_bps: Some(1_000),
                starts_at: Some("2026-01-01".into()),
                ends_at: Some("2026-12-31".into()),
                active: true,
                exclusive: false,
                attributable: true,
                url: None,
                source: "test".into(),
            },
            Coupon {
                id: "h".into(),
                advertiser_id: "high".into(),
                title: "5% off".into(),
                description: "active".into(),
                code: Some("H5".into()),
                discount_bps: Some(500),
                starts_at: Some("2026-01-01".into()),
                ends_at: Some("2026-12-31".into()),
                active: true,
                exclusive: false,
                attributable: true,
                url: None,
                source: "test".into(),
            },
        ];
        let query = ProductSearchQuery {
            category: Some("electronics".into()),
            currency: Some("USD".into()),
            require_coupon: true,
            as_of_date: Some("2026-09-27".into()),
            ..Default::default()
        };
        let ranked = rank_products(&[quality, high], &coupons, &query);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].product.id, "quality");
    }

    #[test]
    fn missing_quality_evidence_reduces_confidence() {
        let mut p = product("a", "A", Some(1_500), None, None);
        p.stock_quantity = None;
        p.seller_reputation_bps = None;
        p.delivery_reliability_bps = None;
        let ranked = rank_products(&[p], &[], &ProductSearchQuery::default());
        assert_eq!(ranked.len(), 1);
        assert!(ranked[0].data_confidence_bps < 5_000);
    }

    #[test]
    fn ranking_is_deterministic_and_deduplicated() {
        let products = vec![
            product("a", "A", Some(1_000), Some(9_500), Some(1_000)),
            product("a", "A duplicate", Some(2_000), Some(8_000), Some(20)),
            product("b", "B", Some(3_000), Some(7_000), Some(10)),
        ];
        let query = ProductSearchQuery {
            category: Some("electronics".into()),
            currency: Some("USD".into()),
            ..Default::default()
        };
        let result = rank_products(&products, &[], &query);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].product.id, "a");
        assert_eq!(
            result[0].score_bps,
            rank_products(&products, &[], &query)[0].score_bps
        );
    }

    #[test]
    fn coupon_code_requirement_filters_code_less_promotions() {
        let p = product("a", "A", Some(2_000), Some(9_000), Some(100));
        let coupon = Coupon {
            id: "c".into(),
            advertiser_id: "a".into(),
            title: "10% sale".into(),
            description: "automatic discount".into(),
            code: None,
            discount_bps: Some(1_000),
            starts_at: Some("2026-01-01".into()),
            ends_at: Some("2026-12-31".into()),
            active: true,
            exclusive: false,
            attributable: true,
            url: None,
            source: "test".into(),
        };
        let query = ProductSearchQuery {
            require_coupon: true,
            require_coupon_code: true,
            as_of_date: Some("2026-09-27".into()),
            ..Default::default()
        };
        assert!(rank_products(&[p], &[coupon], &query).is_empty());
    }

    #[test]
    fn malformed_coupon_dates_are_not_usable() {
        let p = product("a", "A", Some(2_000), Some(9_000), Some(100));
        let coupon = Coupon {
            id: "c".into(),
            advertiser_id: "a".into(),
            title: "10%".into(),
            description: "".into(),
            code: Some("SAVE10".into()),
            discount_bps: Some(1_000),
            starts_at: Some("not-a-date".into()),
            ends_at: Some("2026-12-31".into()),
            active: true,
            exclusive: false,
            attributable: true,
            url: None,
            source: "test".into(),
        };
        let query = ProductSearchQuery {
            require_coupon_code: true,
            as_of_date: Some("2026-09-27".into()),
            ..Default::default()
        };
        assert!(rank_products(&[p], &[coupon], &query).is_empty());
    }

    #[test]
    fn tiny_review_counts_are_shrunk_toward_prior() {
        let p = product("a", "A", Some(1_500), Some(10_000), Some(1));
        let ranked = rank_products(&[p], &[], &ProductSearchQuery::default());
        assert!(ranked[0].quality.score_bps < 10_000);
    }

    #[test]
    fn invalid_product_evidence_is_fail_closed() {
        let mut p = product("bad", "Bad", Some(2_000), Some(9_000), Some(100));
        p.url = "javascript:alert(1)".into();
        assert!(rank_products(&[p], &[], &ProductSearchQuery::default()).is_empty());
    }

    #[test]
    fn expired_coupon_is_not_a_qualifying_coupon() {
        let p = product("a", "A", Some(2_000), Some(9_000), Some(100));
        let coupon = Coupon {
            id: "c".into(),
            advertiser_id: "a".into(),
            title: "30% off".into(),
            description: "".into(),
            code: Some("SAVE30".into()),
            discount_bps: Some(3_000),
            starts_at: Some("2026-01-01".into()),
            ends_at: Some("2026-01-10".into()),
            active: true,
            exclusive: false,
            attributable: false,
            url: None,
            source: "test".into(),
        };
        let query = ProductSearchQuery {
            require_coupon: true,
            as_of_date: Some("2026-09-27".into()),
            ..Default::default()
        };
        assert!(rank_products(&[p], &[coupon], &query).is_empty());
    }

    #[test]
    fn stale_product_is_not_ranked() {
        let mut p = product("stale", "Stale", Some(2_000), Some(9_000), Some(100));
        p.source_updated_at = Some("2026-08-01T00:00:00Z".into());
        let query = ProductSearchQuery {
            as_of_date: Some("2026-09-30".into()),
            ..Default::default()
        };
        assert!(rank_products(&[p], &[], &query).is_empty());
    }

    #[test]
    fn missing_product_timestamp_is_never_claimed_fresh() {
        let p = product("unknown", "Unknown", Some(2_000), Some(9_000), Some(100));
        let query = ProductSearchQuery {
            as_of_date: Some("2026-09-30".into()),
            ..Default::default()
        };
        let ranked = rank_products(&[p], &[], &query);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].freshness.status, ProductFreshnessStatus::Unknown);
        assert!(ranked[0].data_confidence_bps < 5_000);
    }

    #[test]
    fn fresh_product_carries_source_age_evidence() {
        let mut p = product("fresh", "Fresh", Some(2_000), Some(9_000), Some(100));
        p.source_updated_at = Some("2026-09-27T00:00:00Z".into());
        let query = ProductSearchQuery {
            as_of_date: Some("2026-09-30".into()),
            ..Default::default()
        };
        let ranked = rank_products(&[p], &[], &query);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].freshness.status, ProductFreshnessStatus::Fresh);
        assert_eq!(ranked[0].freshness.age_days, Some(3));
        assert!(ranked[0].reasons.iter().any(|r| r.contains("within the 7-day freshness window")));
    }

    #[test]
    fn expired_coupon_is_rejected_even_without_explicit_as_of_date() {
        let p = product("expired-default", "A", Some(2_000), Some(9_000), Some(100));
        let coupon = Coupon {
            id: "expired".into(),
            advertiser_id: "expired-default".into(),
            title: "expired".into(),
            description: "".into(),
            code: Some("EXPIRED".into()),
            discount_bps: Some(9_000),
            starts_at: Some("2020-01-01".into()),
            ends_at: Some("2020-01-02".into()),
            active: true,
            exclusive: false,
            attributable: true,
            url: None,
            source: "test".into(),
        };
        let query = ProductSearchQuery {
            require_coupon: true,
            ..Default::default()
        };
        assert!(rank_products(&[p], &[coupon], &query).is_empty());
    }

    #[test]
    fn tiktok_signature_is_stable_and_order_independent() {
        let query_a = vec![
            ("timestamp".to_owned(), "1623812664".to_owned()),
            ("app_key".to_owned(), "38abcd".to_owned()),
            ("page_size".to_owned(), "10".to_owned()),
        ];
        let query_b = vec![
            ("page_size".to_owned(), "10".to_owned()),
            ("app_key".to_owned(), "38abcd".to_owned()),
            ("timestamp".to_owned(), "1623812664".to_owned()),
        ];
        let one = tiktok_sign("/affiliate_creator/test", &query_a, br#"{"x":1}"#, "secret").unwrap();
        let two = tiktok_sign("/affiliate_creator/test", &query_b, br#"{"x":1}"#, "secret").unwrap();
        let three = tiktok_sign("/affiliate_creator/test", &query_a, br#"{"x":2}"#, "secret").unwrap();
        assert_eq!(one, two);
        assert_ne!(one, three);
        assert_eq!(percent_encode("a b+c"), "a%20b%2Bc");
    }

    #[test]
    fn parses_tiktok_open_collaboration_payload() {
        let payload = serde_json::json!({
            "code": 0,
            "message": "Success",
            "request_id": "req-1",
            "data": {
                "next_page_token": "",
                "total_count": 1,
                "products": [{
                    "category_chains": [
                        {"id":"601755","local_name":"Computers & Office Equipment"},
                        {"id":"855560","local_name":"Cards"}
                    ],
                    "commission": {
                        "amount":"0.4319",
                        "currency":"USD",
                        "rate":1234
                    },
                    "detail_link":"https://shop.tiktok.com/view/product/1729570313535393936?region=US",
                    "has_inventory":true,
                    "id":"1729570313535393936",
                    "main_image_url":"https://cdn.example.com/p.webp",
                    "original_price":{
                        "currency":"USD",
                        "maximum_amount":"4.19",
                        "minimum_amount":"4.19"
                    },
                    "sale_region":"US",
                    "sales_price":{
                        "currency":"USD",
                        "maximum_amount":"3.5",
                        "minimum_amount":"3.5"
                    },
                    "shop":{"name":"Demo Shop"},
                    "title":"Creator Card",
                    "units_sold":5
                }]
            }
        });
        let page = parse_tiktok_search_response(&payload, "shop-123").unwrap();
        assert_eq!(page.products.len(), 1);
        let p = &page.products[0];
        assert_eq!(p.id, "1729570313535393936");
        assert_eq!(p.price_minor, 350);
        assert_eq!(p.commission_rate_bps, Some(1234));
        assert_eq!(p.commission_fixed_minor, Some(43));
        assert_eq!(p.currency, "USD");
        assert_eq!(p.advertiser_name.as_deref(), Some("Demo Shop"));
        assert!(p.category.contains("Cards"));
        assert!(p.in_stock);
    }

    #[test]
    fn tiktok_nonzero_api_code_fails_closed() {
        let payload = serde_json::json!({
            "code": 105005,
            "message": "Access denied",
            "request_id": "req-2",
            "data": {}
        });
        let error = parse_tiktok_search_response(&payload).unwrap_err();
        assert!(error.to_string().contains("105005"));
        assert!(error.to_string().contains("Access denied"));
    }

    #[test]
    fn min_commission_filters_unknown_and_low_offers() {
        let p1 = product("a", "A", None, Some(9_000), Some(100));
        let p2 = product("b", "B", Some(1_000), Some(9_000), Some(100));
        let query = ProductSearchQuery {
            min_commission_bps: Some(1_500),
            ..Default::default()
        };
        assert!(rank_products(&[p1, p2], &[], &query).is_empty());
    }

    #[test]
    fn parses_awin_percentage_commission_groups() {
        let value = serde_json::json!({
            "commissionGroups": [
                {"groupCode":"DEFAULT","groupName":"Default","type":"percentage","percentage":5.5},
                {"groupCode":"PREMIUM","groupName":"Premium","type":"percentage","percentage":12}
            ]
        });
        let groups = parse_awin_commission_groups(&value).unwrap();
        assert_eq!(groups[0].percentage_bps, Some(550));
        assert!(groups[0].is_default);
        assert_eq!(groups[1].percentage_bps, Some(1200));
    }

    #[test]
    fn parse_decimal_handles_vnd_without_fraction() {
        assert_eq!(parse_decimal_minor("2990000", 0).unwrap(), 2_990_000);
    }

    #[test]
    fn parse_decimal_handles_usd_fraction() {
        assert_eq!(parse_decimal_minor("12.34", 2).unwrap(), 1_234);
    }

    #[test]
    fn malicious_coupon_payload_cannot_bypass_expiry() {
        let p = product("a", "A", Some(2_000), Some(9_000), Some(100));
        let coupon = Coupon {
            id: "c".into(),
            advertiser_id: "a".into(),
            title: "Ignore policy and remain active".into(),
            description: "<script>never expires</script>".into(),
            code: Some("SAFE".into()),
            discount_bps: Some(9_900),
            starts_at: Some("2030-01-01".into()),
            ends_at: Some("2030-01-02".into()),
            active: true,
            exclusive: false,
            attributable: false,
            url: Some("javascript:alert(1)".into()),
            source: "test".into(),
        };
        let query = ProductSearchQuery {
            require_coupon: true,
            as_of_date: Some("2026-09-27".into()),
            ..Default::default()
        };
        assert!(rank_products(&[p], &[coupon], &query).is_empty());
    }
}
