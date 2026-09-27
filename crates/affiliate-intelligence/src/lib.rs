#![forbid(unsafe_code)]

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Coupon {
    pub code: String,
    pub description: String,
    pub discount_bps: u32,
    pub discount_minor: i128,
    pub min_order_minor: i128,
    pub expires_at_epoch: Option<i64>,
    pub source: String,
}

impl Coupon {
    pub fn is_active(&self, now_epoch: i64) -> bool {
        !self.code.trim().is_empty()
            && self.expires_at_epoch.map(|expiry| expiry > now_epoch).unwrap_or(true)
    }

    pub fn value_minor(&self, price_minor: i128) -> i128 {
        if price_minor <= 0 || price_minor < self.min_order_minor.max(0) {
            return 0;
        }
        let percent = price_minor.saturating_mul(self.discount_bps as i128) / 10_000;
        percent.max(self.discount_minor).min(price_minor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualitySignals {
    pub rating_bps: Option<u32>,
    pub review_count: Option<u64>,
    pub seller_rating_bps: Option<u32>,
    pub refund_rate_bps: Option<u32>,
    pub complaint_rate_bps: Option<u32>,
    pub stock_quantity: Option<u64>,
    pub units_sold: Option<u64>,
    pub updated_at_epoch: Option<i64>,
    pub evidence: Vec<String>,
}

impl Default for QualitySignals {
    fn default() -> Self {
        Self {
            rating_bps: None,
            review_count: None,
            seller_rating_bps: None,
            refund_rate_bps: None,
            complaint_rate_bps: None,
            stock_quantity: None,
            units_sold: None,
            updated_at_epoch: None,
            evidence: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AffiliateOffer {
    pub provider: String,
    pub merchant_id: String,
    pub product_id: String,
    pub gtin: Option<String>,
    pub category: String,
    pub title: String,
    pub url: String,
    pub currency: String,
    pub price_minor: i128,
    pub original_price_minor: Option<i128>,
    pub commission_rate_bps: Option<u32>,
    pub coupon: Option<Coupon>,
    pub quality: QualitySignals,
    pub available: bool,
    pub collected_at_epoch: i64,
}

impl AffiliateOffer {
    fn normalized_title(&self) -> String {
        self.title.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductSearchQuery {
    pub category: Option<String>,
    pub keywords: Vec<String>,
    pub currency: Option<String>,
    pub min_commission_bps: Option<u32>,
    pub require_coupon: bool,
    pub min_rating_bps: Option<u32>,
    pub min_reviews: Option<u64>,
    pub min_stock: Option<u64>,
    pub max_price_minor: Option<i128>,
    pub max_content_cost_minor: i128,
    pub limit: usize,
    pub now_epoch: i64,
}

impl Default for ProductSearchQuery {
    fn default() -> Self {
        Self {
            category: None,
            keywords: Vec::new(),
            currency: None,
            min_commission_bps: None,
            require_coupon: false,
            min_rating_bps: None,
            min_reviews: None,
            min_stock: None,
            max_price_minor: None,
            max_content_cost_minor: 0,
            limit: 20,
            now_epoch: now_epoch(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankedCandidate {
    pub offer: AffiliateOffer,
    pub score_bps: u32,
    pub quality_score_bps: u32,
    pub commercial_score_bps: u32,
    pub coupon_score_bps: u32,
    pub reliability_score_bps: u32,
    pub content_fit_bps: u32,
    pub expected_commission_minor: i128,
    pub expected_contribution_minor: i128,
    pub quality_confidence_bps: u32,
    pub commercial_confidence_bps: u32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AffiliateSearchResult {
    pub query: ProductSearchQuery,
    pub candidates: Vec<RankedCandidate>,
    pub providers: Vec<String>,
    pub generated_at_epoch: i64,
}

#[derive(Debug)]
pub enum ProviderError {
    Configuration(String),
    Transport(String),
    InvalidData(String),
    Unauthorized(String),
    RateLimited,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(e) => write!(f, "provider configuration error: {e}"),
            Self::Transport(e) => write!(f, "provider transport error: {e}"),
            Self::InvalidData(e) => write!(f, "provider data error: {e}"),
            Self::Unauthorized(e) => write!(f, "provider authorization error: {e}"),
            Self::RateLimited => write!(f, "provider rate limited"),
        }
    }
}

#[async_trait]
pub trait AffiliateProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, query: &ProductSearchQuery) -> Result<Vec<AffiliateOffer>, ProviderError>;
}

pub struct AffiliateIntelligence {
    providers: Vec<Box<dyn AffiliateProvider>>,
}

impl AffiliateIntelligence {
    pub fn new(providers: Vec<Box<dyn AffiliateProvider>>) -> Self {
        Self { providers }
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.name().to_string()).collect()
    }

    pub async fn search(&self, query: ProductSearchQuery) -> Result<AffiliateSearchResult, ProviderError> {
        if query.limit == 0 || query.limit > 100 {
            return Err(ProviderError::Configuration("limit must be in 1..=100".into()));
        }

        let mut all = Vec::new();
        for provider in &self.providers {
            all.extend(provider.search(&query).await?);
        }

        let candidates = rank_candidates(all, &query);
        Ok(AffiliateSearchResult {
            query,
            candidates,
            providers: self.provider_names(),
            generated_at_epoch: now_epoch(),
        })
    }
}

pub fn rank_candidates(mut offers: Vec<AffiliateOffer>, query: &ProductSearchQuery) -> Vec<RankedCandidate> {
    offers.retain(|offer| matches_query(offer, query));

    let mut deduped: BTreeMap<String, AffiliateOffer> = BTreeMap::new();
    for offer in offers {
        let key = offer.gtin.clone().unwrap_or_else(|| {
            format!(
                "{}:{}:{}",
                offer.provider,
                offer.merchant_id,
                offer.product_id
            )
        });
        let replace = deduped.get(&key).map(|current| {
            let current_quality = quality_completeness(current);
            let incoming_quality = quality_completeness(&offer);
            incoming_quality > current_quality
                || (incoming_quality == current_quality
                    && offer.commission_rate_bps.unwrap_or(0) > current.commission_rate_bps.unwrap_or(0))
        }).unwrap_or(true);
        if replace {
            deduped.insert(key, offer);
        }
    }

    let offers: Vec<AffiliateOffer> = deduped.into_values().collect();
    if offers.is_empty() {
        return Vec::new();
    }

    let max_expected = offers.iter()
        .map(|o| expected_commission(o, query.now_epoch))
        .max()
        .unwrap_or(1)
        .max(1);
    let max_units_sold = offers.iter()
        .filter_map(|o| o.quality.units_sold)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut ranked = offers.into_iter().map(|offer| {
        let quality = quality_score(&offer.quality);
        let commission = offer.commission_rate_bps.unwrap_or(0);
        let expected_commission = expected_commission(&offer, query.now_epoch);
        let coupon_value = offer.coupon.as_ref()
            .filter(|c| c.is_active(query.now_epoch))
            .map(|c| c.value_minor(offer.price_minor))
            .unwrap_or(0);
        let net_price = offer.price_minor.saturating_sub(coupon_value).max(0);
        let contribution = expected_commission.saturating_sub(query.max_content_cost_minor);

        let commercial = ((expected_commission.saturating_mul(10_000)) / max_expected)
            .clamp(0, 10_000) as u32;
        let coupon_score = if coupon_value > 0 {
            ((coupon_value.saturating_mul(10_000)) / offer.price_minor.max(1)).clamp(0, 10_000) as u32
        } else {
            0
        };
        let reliability = reliability_score(&offer, query.now_epoch);
        let content_fit = content_fit_score(&offer, query);
        let demand = offer.quality.units_sold
            .map(|units| ((units.saturating_mul(10_000)) / max_units_sold).min(10_000) as u32)
            .unwrap_or(0);

        let contribution_score = ((contribution.max(0).saturating_mul(10_000)) / max_expected.max(1))
            .clamp(0, 10_000) as u32;
        let commercial_score = ((commercial as u64 * 7 + (commission.min(10_000) as u64) * 3) / 10) as u32;
        let score = (
            contribution_score as u64 * 25
            + quality as u64 * 45
            + commercial_score as u64 * 10
            + coupon_score as u64 * 8
            + reliability as u64 * 6
            + content_fit as u64 * 4
            + demand as u64 * 2
        ) / 100;

        let quality_confidence = ((quality_completeness(&offer) * 10_000) / 6).min(10_000);
        let commercial_confidence = match offer.commission_rate_bps {
            Some(_) => if offer.coupon.as_ref().map(|c| c.is_active(query.now_epoch)).unwrap_or(false) { 10_000 } else { 8_000 },
            None => 2_500,
        };

        RankedCandidate {
            offer,
            score_bps: score.min(10_000) as u32,
            quality_score_bps: quality,
            commercial_score_bps: commercial_score,
            coupon_score_bps: coupon_score,
            reliability_score_bps: reliability,
            content_fit_bps: content_fit,
            expected_commission_minor,
            expected_contribution_minor: contribution,
            quality_confidence_bps: quality_confidence,
            commercial_confidence_bps: commercial_confidence,
            evidence: vec![
                format!("expected_commission_minor={expected_commission}"),
                format!("active_coupon_value_minor={coupon_value}"),
                format!("net_price_minor={net_price}"),
                format!("quality_score_bps={quality}"),
            ],
        }
    }).collect::<Vec<_>>();

    ranked.sort_by(|a,b| {
        b.score_bps.cmp(&a.score_bps)
            .then_with(|| b.expected_contribution_minor.cmp(&a.expected_contribution_minor))
            .then_with(|| b.quality_score_bps.cmp(&a.quality_score_bps))
            .then_with(|| a.offer.product_id.cmp(&b.offer.product_id))
    });

    ranked.truncate(query.limit);
    ranked
}

fn matches_query(offer: &AffiliateOffer, query: &ProductSearchQuery) -> bool {
    if let Some(currency) = &query.currency {
        if offer.currency != *currency {
            return false;
        }
    }
    if let Some(category) = &query.category {
        if !offer.category.to_ascii_lowercase().contains(&category.to_ascii_lowercase()) {
            return false;
        }
    }
    let title = offer.normalized_title();
    if !query.keywords.is_empty()
        && !query.keywords.iter().all(|k| title.contains(&k.to_ascii_lowercase()))
    {
        return false;
    }
    if let Some(min_commission) = query.min_commission_bps {
        if offer.commission_rate_bps.unwrap_or(0) < min_commission {
            return false;
        }
    }
    if query.require_coupon
        && !offer.coupon.as_ref().map(|c| c.is_active(query.now_epoch)).unwrap_or(false)
    {
        return false;
    }
    if let Some(min_rating) = query.min_rating_bps {
        if offer.quality.rating_bps.unwrap_or(0) < min_rating {
            return false;
        }
    }
    if let Some(min_reviews) = query.min_reviews {
        if offer.quality.review_count.unwrap_or(0) < min_reviews {
            return false;
        }
    }
    if let Some(min_stock) = query.min_stock {
        if offer.quality.stock_quantity.unwrap_or(0) < min_stock {
            return false;
        }
    }
    if let Some(max_price) = query.max_price_minor {
        if offer.price_minor > max_price {
            return false;
        }
    }
    offer.available && offer.price_minor > 0
}

fn quality_completeness(offer: &AffiliateOffer) -> u32 {
    let q = &offer.quality;
    [
        q.rating_bps.is_some(),
        q.review_count.is_some(),
        q.seller_rating_bps.is_some(),
        q.refund_rate_bps.is_some(),
        q.stock_quantity.is_some(),
        q.updated_at_epoch.is_some(),
    ].into_iter().filter(|v| *v).count() as u32
}

fn quality_score(q: &QualitySignals) -> u32 {
    let rating = q.rating_bps.unwrap_or(0).min(5_000) as u64;
    let rating_score = rating.saturating_mul(2);
    let reviews_score = match q.review_count.unwrap_or(0) {
        0 => 0,
        1..=9 => 2_500,
        10..=99 => 5_000,
        100..=999 => 7_500,
        _ => 10_000,
    };
    let seller_score = q.seller_rating_bps.unwrap_or(0).min(10_000) as u64;
    let refund_score = 10_000_u64.saturating_sub(q.refund_rate_bps.unwrap_or(5_000).min(10_000) as u64);
    let complaint_score = 10_000_u64.saturating_sub(q.complaint_rate_bps.unwrap_or(5_000).min(10_000) as u64);
    let stock_score = match q.stock_quantity.unwrap_or(0) {
        0 => 0,
        1..=4 => 2_500,
        5..=49 => 6_000,
        50..=499 => 8_000,
        _ => 10_000,
    };
    ((rating_score * 40
        + reviews_score * 20
        + seller_score * 15
        + refund_score * 10
        + complaint_score * 5
        + stock_score * 10) / 100)
        .min(10_000) as u32
}

fn reliability_score(offer: &AffiliateOffer, now: i64) -> u32 {
    if !offer.available {
        return 0;
    }
    let freshness = match offer.quality.updated_at_epoch {
        Some(ts) if now >= ts => {
            let age = now.saturating_sub(ts);
            if age <= 86_400 { 10_000 }
            else if age <= 3 * 86_400 { 8_000 }
            else if age <= 7 * 86_400 { 6_000 }
            else if age <= 30 * 86_400 { 3_000 }
            else { 1_000 }
        }
        Some(_) => 1_000,
        None => 2_000,
    };
    let stock = if offer.quality.stock_quantity.unwrap_or(1) > 0 { 10_000 } else { 0 };
    ((freshness as u64 * 70 + stock as u64 * 30) / 100) as u32
}

fn content_fit_score(offer: &AffiliateOffer, query: &ProductSearchQuery) -> u32 {
    if query.keywords.is_empty() {
        return 5_000;
    }
    let text = format!(
        "{} {}",
        offer.title.to_ascii_lowercase(),
        offer.category.to_ascii_lowercase()
    );
    let matched = query.keywords.iter().filter(|k| text.contains(&k.to_ascii_lowercase())).count();
    ((matched * 10_000) / query.keywords.len()).min(10_000) as u32
}

fn expected_commission(offer: &AffiliateOffer, now: i64) -> i128 {
    let coupon_value = offer.coupon.as_ref()
        .filter(|c| c.is_active(now))
        .map(|c| c.value_minor(offer.price_minor))
        .unwrap_or(0);
    let net_price = offer.price_minor.saturating_sub(coupon_value).max(0);
    net_price.saturating_mul(offer.commission_rate_bps.unwrap_or(0) as i128) / 10_000
}

pub fn now_epoch() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

pub struct MockAffiliateProvider {
    offers: Vec<AffiliateOffer>,
}

impl MockAffiliateProvider {
    pub fn new(offers: Vec<AffiliateOffer>) -> Self {
        Self { offers }
    }
}

#[async_trait]
impl AffiliateProvider for MockAffiliateProvider {
    fn name(&self) -> &'static str { "mock" }
    async fn search(&self, _query: &ProductSearchQuery) -> Result<Vec<AffiliateOffer>, ProviderError> {
        Ok(self.offers.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AwinCsvProvider {
    pub feed_url: String,
    pub bearer_token: Option<String>,
    pub commission_group_rates_bps: HashMap<String, u32>,
    pub client: reqwest::Client,
    pub coupon_feed_url: Option<String>,
}

impl AwinCsvProvider {
    pub fn from_env() -> Result<Self, ProviderError> {
        let feed_url = std::env::var("AWIN_FEED_URL")
            .map_err(|_| ProviderError::Configuration("AWIN_FEED_URL is required".into()))?;
        let bearer_token = std::env::var("AWIN_FEED_BEARER").ok().filter(|v| !v.trim().is_empty());
        let mut commission_group_rates_bps = HashMap::new();
        if let Ok(raw) = std::env::var("AWIN_COMMISSION_GROUP_RATES_BPS") {
            for item in raw.split(',') {
                let mut parts = item.splitn(2, ':');
                if let (Some(group), Some(rate)) = (parts.next(), parts.next()) {
                    if let Ok(rate) = rate.parse::<u32>() {
                        commission_group_rates_bps.insert(group.trim().to_ascii_uppercase(), rate.min(10_000));
                    }
                }
            }
        }
        Ok(Self {
            feed_url,
            bearer_token,
            commission_group_rates_bps,
            client: reqwest::Client::new(),
            coupon_feed_url: std::env::var("AWIN_COUPON_FEED_URL").ok().filter(|v| !v.trim().is_empty()),
        })
    }

    async fn fetch_text(&self, url: &str) -> Result<String, ProviderError> {
        let mut request = self.client.get(url);
        if let Some(token) = &self.bearer_token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(|e| ProviderError::Transport(e.to_string()))?;
        if response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN {
            return Err(ProviderError::Unauthorized("Awin feed authorization failed".into()));
        }
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited);
        }
        if !response.status().is_success() {
            return Err(ProviderError::Transport(format!("http {}", response.status())));
        }
        let bytes = response.bytes().await.map_err(|e| ProviderError::Transport(e.to_string()))?;
        if bytes.len() > 50 * 1024 * 1024 {
            return Err(ProviderError::InvalidData("feed exceeds 50 MiB safety limit".into()));
        }
        String::from_utf8(bytes.to_vec()).map_err(|e| ProviderError::InvalidData(e.to_string()))
    }

    fn parse_feed(&self, body: &str, query: &ProductSearchQuery, coupons: &HashMap<String, Coupon>) -> Result<Vec<AffiliateOffer>, ProviderError> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(body.as_bytes());
        let headers = reader.headers().map_err(|e| ProviderError::InvalidData(e.to_string()))?.clone();
        let index = HeaderIndex::new(&headers);
        let now = query.now_epoch();
        let mut output = Vec::new();

        for row in reader.records() {
            let row = row.map_err(|e| ProviderError::InvalidData(e.to_string()))?;
            let product_id = index.required(&row, &["product_id", "pid"])?;
            let name = index.required(&row, &["product_name", "name"])?;
            let price = parse_minor(index.optional(&row, &["price"])).ok_or_else(|| ProviderError::InvalidData("Awin row missing valid price".into()))?;
            let currency = index.optional(&row, &["currency"]).unwrap_or("USD").trim().to_ascii_uppercase();
            if currency.len() != 3 {
                continue;
            }
            let category = index.optional(&row, &["merchant_category", "category"]).unwrap_or("").to_string();
            let merchant_id = index.optional(&row, &["advertiser_id", "merchant_id"]).unwrap_or("unknown").to_string();
            let commission_group = index.optional(&row, &["commission_group", "cg"]).unwrap_or("DEFAULT").to_ascii_uppercase();
            let commission_rate = index.optional(&row, &["commission_rate_bps"]).and_then(|v| v.parse::<u32>().ok())
                .or_else(|| self.commission_group_rates_bps.get(&commission_group).copied());

            let coupon = coupons.get(&merchant_id).cloned();
            let quality = QualitySignals {
                rating_bps: parse_rating_bps(index.optional(&row, &["average_rating", "rating"])),
                review_count: index.optional(&row, &["reviews"]).and_then(|v| v.parse().ok()),
                seller_rating_bps: parse_rating_bps(index.optional(&row, &["seller_rating"])),
                refund_rate_bps: index.optional(&row, &["refund_rate_bps"]).and_then(|v| v.parse().ok()),
                complaint_rate_bps: index.optional(&row, &["complaint_rate_bps"]).and_then(|v| v.parse().ok()),
                stock_quantity: index.optional(&row, &["stock_quantity", "stockquant"]).and_then(|v| v.parse().ok()),
                units_sold: index.optional(&row, &["units_sold"]).and_then(|v| v.parse().ok()),
                updated_at_epoch: index.optional(&row, &["last_updated"]).and_then(parse_epoch),
                evidence: vec!["Awin product feed".into()],
            };

            output.push(AffiliateOffer {
                provider: "awin".into(),
                merchant_id,
                product_id,
                gtin: index.optional(&row, &["ean", "upc"]).map(str::to_string).filter(|v| !v.is_empty()),
                category,
                title: name.to_string(),
                url: index.optional(&row, &["deep_link", "purl"]).unwrap_or("").to_string(),
                currency,
                price_minor: price,
                original_price_minor: parse_minor(index.optional(&row, &["original_price", "was_price"])),
                commission_rate_bps: commission_rate.filter(|v| *v <= 10_000),
                coupon,
                quality,
                available: index.optional(&row, &["in_stock", "instock"]).map(parse_bool).unwrap_or(true),
                collected_at_epoch: now,
            });
        }

        Ok(output)
    }
}

#[async_trait]
impl AffiliateProvider for AwinCsvProvider {
    fn name(&self) -> &'static str { "awin" }

    async fn search(&self, query: &ProductSearchQuery) -> Result<Vec<AffiliateOffer>, ProviderError> {
        let feed = self.fetch_text(&self.feed_url).await?;
        let coupons = if let Some(url) = &self.coupon_feed_url {
            let text = self.fetch_text(url).await?;
            parse_coupon_json(&text)?
        } else {
            HashMap::new()
        };
        self.parse_feed(&feed, query, &coupons)
    }
}

fn parse_coupon_json(body: &str) -> Result<HashMap<String, Coupon>, ProviderError> {
    let value: Value = serde_json::from_str(body).map_err(|e| ProviderError::InvalidData(e.to_string()))?;
    let rows = value.as_array().cloned().or_else(|| value.get("offers").and_then(Value::as_array).cloned()).unwrap_or_default();
    let mut coupons = HashMap::new();
    for row in rows {
        let merchant_id = row.get("merchant_id").or_else(|| row.get("advertiser_id")).and_then(Value::as_str).unwrap_or("").to_string();
        let code = row.get("code").or_else(|| row.get("voucher_code")).and_then(Value::as_str).unwrap_or("").to_string();
        if merchant_id.is_empty() || code.is_empty() {
            continue;
        }
        let coupon = Coupon {
            code,
            description: row.get("description").and_then(Value::as_str).unwrap_or("").to_string(),
            discount_bps: row.get("discount_bps").and_then(Value::as_u64).unwrap_or_else(|| {
                row.get("discount_percent").and_then(Value::as_f64).unwrap_or(0.0).mul_add(100.0, 0.0) as u64
            }).min(10_000) as u32,
            discount_minor: row.get("discount_minor").and_then(Value::as_i64).unwrap_or(0) as i128,
            min_order_minor: row.get("min_order_minor").and_then(Value::as_i64).unwrap_or(0) as i128,
            expires_at_epoch: row.get("expires_at_epoch").and_then(Value::as_i64),
            source: "awin-promotion-feed".into(),
        };
        let replace = coupons.get(&merchant_id)
            .map(|c| coupon.expires_at_epoch.unwrap_or(i64::MAX) > c.expires_at_epoch.unwrap_or(i64::MAX))
            .unwrap_or(true);
        if replace {
            coupons.insert(merchant_id, coupon);
        }
    }
    Ok(coupons)
}

struct HeaderIndex {
    map: HashMap<String, usize>,
}

impl HeaderIndex {
    fn new(headers: &csv::StringRecord) -> Self {
        let map = headers.iter().enumerate()
            .map(|(i,h)| (h.trim().to_ascii_lowercase(), i))
            .collect();
        Self { map }
    }
    fn optional<'a>(&self, row: &'a csv::StringRecord, names: &[&str]) -> Option<&'a str> {
        names.iter().find_map(|name| self.map.get(*name).and_then(|i| row.get(*i)).filter(|v| !v.trim().is_empty()))
    }
    fn required<'a>(&self, row: &'a csv::StringRecord, names: &[&str]) -> Result<String, ProviderError> {
        self.optional(row, names).map(str::to_string).ok_or_else(|| ProviderError::InvalidData(format!("missing feed field: {}", names.join("/"))))
    }
}

fn parse_minor(value: Option<&str>) -> Option<i128> {
    let value = value?.trim().replace(',', "");
    if value.is_empty() {
        return None;
    }
    if let Some((whole, frac)) = value.split_once('.') {
        let sign = whole.starts_with('-');
        let whole_abs = whole.trim_start_matches('-').parse::<i128>().ok()?;
        let mut frac = frac.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
        frac.truncate(2);
        while frac.len() < 2 { frac.push('0'); }
        let cents = frac.parse::<i128>().ok()?;
        let total = whole_abs.checked_mul(100)?.checked_add(cents)?;
        Some(if sign { -total } else { total })
    } else {
        value.parse::<i128>().ok()?.checked_mul(100)
    }
}

fn parse_rating_bps(value: Option<&str>) -> Option<u32> {
    let value = value?.trim();
    let parsed = value.parse::<f64>().ok()?;
    if !(0.0..=5.0).contains(&parsed) {
        return None;
    }
    Some((parsed * 1_000.0).round() as u32)
}

fn parse_bool(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "y" | "in stock" | "instock")
}

fn parse_epoch(value: &str) -> Option<i64> {
    if let Ok(v) = value.trim().parse::<i64>() {
        Some(if v > 2_000_000_000_000 { v / 1000 } else { v })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct TikTokShopOpenCollaborationProvider {
    pub app_key: String,
    pub app_secret: String,
    pub access_token: String,
    pub shop_cipher: String,
    pub api_base: String,
    pub api_path: String,
    pub client: reqwest::Client,
}

impl TikTokShopOpenCollaborationProvider {
    pub fn from_env() -> Result<Self, ProviderError> {
        let required = |key: &str| std::env::var(key).map_err(|_| ProviderError::Configuration(format!("{key} is required")));
        Ok(Self {
            app_key: required("TIKTOK_APP_KEY")?,
            app_secret: required("TIKTOK_APP_SECRET")?,
            access_token: required("TIKTOK_ACCESS_TOKEN")?,
            shop_cipher: required("TIKTOK_SHOP_CIPHER")?,
            api_base: std::env::var("TIKTOK_API_BASE").unwrap_or_else(|_| "https://open-api.tiktokglobalshop.com".into()),
            api_path: std::env::var("TIKTOK_AFFILIATE_SEARCH_PATH")
                .unwrap_or_else(|_| "/affiliate_seller/202405/open_collaborations/products/search".into()),
            client: reqwest::Client::new(),
        })
    }

    pub async fn search_with_raw_response(&self, query: &ProductSearchQuery) -> Result<Value, ProviderError> {
        let body = tiktok_body(query);
        let timestamp = now_epoch().to_string();

        let mut params = BTreeMap::new();
        params.insert("app_key".to_string(), self.app_key.clone());
        params.insert("shop_cipher".to_string(), self.shop_cipher.clone());
        params.insert("timestamp".to_string(), timestamp);
        params.insert("page_size".to_string(), query.limit.min(100).to_string());

        let sign = sign_tiktok_request(&self.api_path, &params, &body, &self.app_secret)?;
        params.insert("sign".to_string(), sign);

        let url = format!("{}{}?{}", self.api_base.trim_end_matches('/'), self.api_path, encode_query(&params));
        let response = self.client
            .post(url)
            .header("content-type", "application/json")
            .header("x-tts-access-token", &self.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Transport(e.to_string()))?;

        if response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN {
            return Err(ProviderError::Unauthorized("TikTok Shop access token or scope is not authorized".into()));
        }
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited);
        }
        let status = response.status();
        let value: Value = response.json().await.map_err(|e| ProviderError::InvalidData(e.to_string()))?;
        if !status.is_success() {
            return Err(ProviderError::Transport(format!("http {status}: {value}")));
        }
        if value.get("code").and_then(Value::as_i64).unwrap_or(-1) != 0 {
            return Err(ProviderError::Transport(
                value.get("message").and_then(Value::as_str).unwrap_or("TikTok Shop request failed").into()
            ));
        }
        Ok(value)
    }
}

#[async_trait]
impl AffiliateProvider for TikTokShopOpenCollaborationProvider {
    fn name(&self) -> &'static str { "tiktok-shop" }

    async fn search(&self, query: &ProductSearchQuery) -> Result<Vec<AffiliateOffer>, ProviderError> {
        let response = self.search_with_raw_response(query).await?;
        let rows = response.pointer("/data/products").and_then(Value::as_array).cloned().unwrap_or_default();
        let mut output = Vec::with_capacity(rows.len());

        for row in rows {
            let product_id = row.get("id").and_then(Value::as_str).unwrap_or("").to_string();
            let title = row.get("title").and_then(Value::as_str).unwrap_or("").to_string();
            if product_id.is_empty() || title.is_empty() {
                continue;
            }
            let sale_price = parse_decimal_minor(
                row.pointer("/sales_price/minimum_amount").and_then(Value::as_str)
            ).unwrap_or(0);
            let original_price = parse_decimal_minor(
                row.pointer("/original_price/minimum_amount").and_then(Value::as_str)
            );
            let currency = row.pointer("/sales_price/currency").and_then(Value::as_str).unwrap_or("USD").to_ascii_uppercase();
            let commission = row.pointer("/commission/rate").and_then(Value::as_i64).map(|v| v.clamp(0, 10_000) as u32);
            let category = row.pointer("/category_chains/0/local_name").and_then(Value::as_str).unwrap_or("").to_string();
            let units_sold = row.get("units_sold").and_then(Value::as_u64);
            let stock = row.get("available_stock").and_then(Value::as_u64);
            let url = row.get("detail_link").or_else(|| row.get("product_url")).and_then(Value::as_str).unwrap_or("").to_string();

            output.push(AffiliateOffer {
                provider: "tiktok-shop".into(),
                merchant_id: row.pointer("/shop/name").and_then(Value::as_str).unwrap_or("unknown").into(),
                product_id,
                gtin: None,
                category,
                title,
                url,
                currency,
                price_minor: sale_price,
                original_price_minor: original_price,
                commission_rate_bps: commission,
                coupon: None,
                quality: QualitySignals {
                    rating_bps: row.get("average_rating").and_then(Value::as_f64).map(|v| (v * 1_000.0).round() as u32),
                    review_count: row.get("review_count").and_then(Value::as_u64),
                    seller_rating_bps: row.pointer("/shop/rating").and_then(Value::as_f64).map(|v| (v * 1_000.0).round() as u32),
                    refund_rate_bps: None,
                    complaint_rate_bps: None,
                    stock_quantity: stock,
                    units_sold,
                    updated_at_epoch: None,
                    evidence: vec!["TikTok Shop Affiliate Open Collaboration API".into()],
                },
                available: true,
                collected_at_epoch: now_epoch(),
            });
        }

        Ok(output)
    }
}

fn tiktok_body(query: &ProductSearchQuery) -> Value {
    let mut object = serde_json::Map::new();
    if let Some(category) = &query.category {
        if category.parse::<u64>().is_ok() {
            object.insert("category".into(), json!({"id": category}));
        }
    }
    if !query.keywords.is_empty() {
        object.insert("title_keywords".into(), json!(query.keywords));
    }
    if let Some(max_price) = query.max_price_minor {
        object.insert("sales_price_range".into(), json!({
            "amount_lt": format!("{}.{}", max_price / 100, max_price.abs() % 100)
        }));
    }
    if let Some(min_commission) = query.min_commission_bps {
        object.insert("commission_rate_range".into(), json!({"rate_ge": min_commission}));
    }
    Value::Object(object)
}

fn parse_decimal_minor(value: Option<&str>) -> Option<i128> {
    parse_minor(value)
}

fn sign_tiktok_request(path: &str, params: &BTreeMap<String, String>, body: &Value, secret: &str) -> Result<String, ProviderError> {
    let mut canonical = path.to_string();
    for (key, value) in params {
        if key == "sign" || key == "access_token" {
            continue;
        }
        canonical.push_str(key);
        canonical.push_str(value);
    }
    canonical.push_str(&serde_json::to_string(body).map_err(|e| ProviderError::InvalidData(e.to_string()))?);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| ProviderError::Configuration("invalid TikTok app secret".into()))?;
    mac.update(canonical.as_bytes());
    Ok(hex_lower(&mac.finalize().into_bytes()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn encode_query(params: &BTreeMap<String, String>) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in params {
        serializer.append_pair(k, v);
    }
    serializer.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer(
        id: &str,
        commission: u32,
        rating: u32,
        reviews: u64,
        coupon: Option<Coupon>,
    ) -> AffiliateOffer {
        AffiliateOffer {
            provider: "mock".into(),
            merchant_id: "m1".into(),
            product_id: id.into(),
            gtin: Some(id.into()),
            category: "electronics".into(),
            title: format!("Product {id}"),
            url: format!("https://example.test/{id}"),
            currency: "USD".into(),
            price_minor: 10_000,
            original_price_minor: Some(12_000),
            commission_rate_bps: Some(commission),
            coupon,
            quality: QualitySignals {
                rating_bps: Some(rating),
                review_count: Some(reviews),
                seller_rating_bps: Some(9_000),
                refund_rate_bps: Some(200),
                complaint_rate_bps: Some(100),
                stock_quantity: Some(100),
                units_sold: Some(1_000),
                updated_at_epoch: Some(1_000_000),
                evidence: vec!["test".into()],
            },
            available: true,
            collected_at_epoch: 1_000_000,
        }
    }

    #[test]
    fn expired_coupon_is_not_eligible() {
        let mut q = ProductSearchQuery { require_coupon: true, now_epoch: 2_000, ..Default::default() };
        let expired = Coupon {
            code: "OLD10".into(),
            description: "expired".into(),
            discount_bps: 1_000,
            discount_minor: 0,
            min_order_minor: 0,
            expires_at_epoch: Some(1_000),
            source: "test".into(),
        };
        assert!(rank_candidates(vec![offer("a", 2_000, 4_500, 100, Some(expired))], &q).is_empty());
        q.require_coupon = false;
        assert_eq!(rank_candidates(vec![offer("a", 2_000, 4_500, 100, None)], &q).len(), 1);
    }

    #[test]
    fn high_commission_is_not_enough_when_quality_is_bad() {
        let q = ProductSearchQuery { now_epoch: 1_000_100, ..Default::default() };
        let bad = AffiliateOffer {
            quality: QualitySignals {
                rating_bps: Some(2_000),
                review_count: Some(1),
                seller_rating_bps: Some(3_000),
                refund_rate_bps: Some(4_000),
                complaint_rate_bps: Some(3_000),
                stock_quantity: Some(5),
                units_sold: Some(2_000),
                updated_at_epoch: Some(1_000_000),
                evidence: vec![],
            },
            ..offer("bad", 6_000, 4_500, 100, Some(Coupon {
                code: "HI".into(),
                description: "huge coupon".into(),
                discount_bps: 1_500,
                discount_minor: 0,
                min_order_minor: 0,
                expires_at_epoch: Some(2_000_000),
                source: "test".into(),
            }))
        };
        let good = offer("good", 2_500, 4_800, 2_000, Some(Coupon {
            code: "GOOD10".into(),
            description: "10%".into(),
            discount_bps: 1_000,
            discount_minor: 0,
            min_order_minor: 0,
            expires_at_epoch: Some(2_000_000),
            source: "test".into(),
        }));
        let ranked = rank_candidates(vec![bad, good], &q);
        assert_eq!(ranked[0].offer.product_id, "good");
    }

    #[test]
    fn duplicate_gtin_is_deduped() {
        let q = ProductSearchQuery::default();
        let a = offer("a", 2_000, 4_500, 100, None);
        let mut b = offer("b", 3_000, 4_500, 100, None);
        b.gtin = a.gtin.clone();
        let ranked = rank_candidates(vec![a, b], &q);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].offer.product_id, "b");
    }

    #[test]
    fn missing_commission_has_low_commercial_confidence() {
        let q = ProductSearchQuery::default();
        let mut item = offer("x", 2_000, 4_500, 100, None);
        item.commission_rate_bps = None;
        let ranked = rank_candidates(vec![item], &q);
        assert_eq!(ranked[0].commercial_confidence_bps, 2_500);
        assert_eq!(ranked[0].expected_commission_minor, 0);
    }

    #[test]
    fn category_and_commission_filters_are_enforced() {
        let q = ProductSearchQuery {
            category: Some("electronics".into()),
            min_commission_bps: Some(2_000),
            ..Default::default()
        };
        let good = offer("a", 2_500, 4_500, 100, None);
        let bad = AffiliateOffer { category: "beauty".into(), ..offer("b", 5_000, 4_500, 100, None) };
        assert_eq!(rank_candidates(vec![good, bad], &q).len(), 1);
        assert_eq!(rank_candidates(vec![good, bad], &q)[0].offer.product_id, "a");
    }

    #[test]
    fn score_is_deterministic() {
        let q = ProductSearchQuery { now_epoch: 1_000_100, ..Default::default() };
        let offers = vec![offer("b", 3_000, 4_500, 100, None), offer("a", 3_500, 4_500, 100, None)];
        let first = rank_candidates(offers.clone(), &q);
        let second = rank_candidates(offers, &q);
        assert_eq!(first, second);
    }

    #[test]
    fn malicious_coupon_does_not_create_negative_price() {
        let mut item = offer("x", 2_000, 4_500, 100, None);
        item.coupon = Some(Coupon {
            code: "ALL".into(),
            description: "100%".into(),
            discount_bps: 20_000,
            discount_minor: i128::MAX,
            min_order_minor: 0,
            expires_at_epoch: Some(i64::MAX),
            source: "untrusted".into(),
        });
        let q = ProductSearchQuery::default();
        let result = rank_candidates(vec![item], &q);
        assert_eq!(result[0].expected_commission_minor, 0);
    }
}
