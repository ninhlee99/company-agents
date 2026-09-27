#![forbid(unsafe_code)]

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

const SCORE_MAX: u32 = 10_000;
const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_FEED_BYTES: usize = 64 * 1024 * 1024;

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
    pub effective_discount_bps: Option<u32>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankedProduct {
    pub product: Product,
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

#[async_trait]
pub trait AffiliateProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn products(&self) -> Result<Vec<Product>, AffiliateError>;
    async fn coupons(&self) -> Result<Vec<Coupon>, AffiliateError>;
}

pub async fn search(
    provider: Arc<dyn AffiliateProvider>,
    query: ProductSearchQuery,
) -> Result<SearchResponse, AffiliateError> {
    validate_query(&query)?;
    let products = provider.products().await?;
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

        let quality = quality_assessment(product);
        let economics = economics_assessment(product, &usable_coupons);
        let content_fit_bps = content_fit(product, query);
        let data_confidence_bps = confidence(product, &quality, &economics);
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
        if data_confidence_bps < 6_000 {
            reasons.push("limited evidence coverage; treat ranking as lower confidence".into());
        }

        let candidate = RankedProduct {
            product: product.clone(),
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

fn quality_assessment(product: &Product) -> QualityAssessment {
    let mut values = Vec::new();
    let mut weights = Vec::new();
    let mut reasons = Vec::new();

    if let Some(rating) = product.rating_bps {
        values.push(rating as u64 * 50);
        weights.push(50_u64);
        reasons.push(format!("provider rating={:.2}/5", rating as f64 / 2_000.0));
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

    let effective_discount_bps = coupons
        .iter()
        .filter_map(|coupon| coupon.discount_bps)
        .chain(product.savings_bps)
        .max();

    let commission_score = product.commission_rate_bps.unwrap_or(0).min(SCORE_MAX);
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
        expected_commission_minor.is_some(),
    ]
    .into_iter()
    .filter(|v| *v)
    .count() as u32
        * 3_333;

    EconomicsAssessment {
        score_bps,
        confidence_bps: confidence_bps.min(SCORE_MAX),
        expected_commission_minor,
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
) -> u32 {
    let source_signal = if product.source.trim().is_empty() {
        0
    } else {
        1_000
    };
    ((quality.confidence_bps as u64 + economics.confidence_bps as u64 + source_signal as u64) / 3)
        .min(SCORE_MAX as u64) as u32
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

fn coupon_is_active_on(coupon: &Coupon, as_of_date: Option<&str>) -> bool {
    if !coupon.active {
        return false;
    }
    let Some(today) = as_of_date else {
        return true;
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
        let product_feed_api_key = std::env::var("AWIN_PRODUCT_FEED_API_KEY")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(|v| v.trim().to_owned());
        if product_feed_url.is_none() && product_feed_api_key.is_none() {
            return Err(AffiliateError::Provider(
                "set AWIN_PRODUCT_FEED_URL or AWIN_PRODUCT_FEED_API_KEY".into(),
            ));
        }
        let feed_id = std::env::var("AWIN_FEED_ID")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(|v| v.trim().to_owned());
        let publisher_id = std::env::var("AWIN_PUBLISHER_ID")
            .map_err(|_| AffiliateError::Provider("AWIN_PUBLISHER_ID is required".into()))?;
        let access_token = std::env::var("AWIN_ACCESS_TOKEN")
            .map_err(|_| AffiliateError::Provider("AWIN_ACCESS_TOKEN is required".into()))?;
        let auto_fetch_commissions = std::env::var("AWIN_AUTO_FETCH_COMMISSIONS")
            .ok()
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(true);
        let max_commission_advertisers = std::env::var("AWIN_MAX_COMMISSION_ADVERTISERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .map(|v| v.clamp(1, 100))
            .unwrap_or(25);
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

    async fn discover_feed_url(&self) -> Result<String, AffiliateError> {
        let api_key = self
            .product_feed_api_key
            .as_deref()
            .ok_or_else(|| AffiliateError::Provider("Awin feed API key is missing".into()))?;
        let url = format!("https://productdata.awin.com/datafeed/list/apikey/{api_key}");
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| AffiliateError::Provider(e.to_string()))?;
        if !response.status().is_success() {
            return Err(AffiliateError::Provider(format!(
                "Awin feed list HTTP {}",
                response.status()
            )));
        }
        let body = response
            .bytes()
            .await
            .map_err(|e| AffiliateError::Provider(e.to_string()))?;
        if body.len() > 8 * 1024 * 1024 {
            return Err(AffiliateError::PayloadTooLarge);
        }
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(body.as_ref());
        let headers = reader
            .headers()
            .map_err(|e| AffiliateError::Parse(e.to_string()))?
            .iter()
            .map(|h| h.to_ascii_lowercase())
            .collect::<Vec<_>>();
        let url_index = headers
            .iter()
            .position(|h| h == "url" || h == "feed url")
            .ok_or_else(|| AffiliateError::Parse("Awin feed list missing URL column".into()))?;
        let feed_id_index = headers
            .iter()
            .position(|h| h == "feed id" || h == "id");
        let membership_index = headers
            .iter()
            .position(|h| h == "membership status" || h == "status");

        let mut fallback = None;
        for row in reader.records() {
            let row = row.map_err(|e| AffiliateError::Parse(e.to_string()))?;
            let candidate = row.get(url_index).unwrap_or_default().trim();
            if candidate.is_empty() {
                continue;
            }
            if let Some(required_id) = self.feed_id.as_deref() {
                if feed_id_index
                    .and_then(|index| row.get(index))
                    .is_some_and(|id| id.trim() == required_id)
                {
                    return Ok(candidate.to_owned());
                }
            }
            let joined = membership_index
                .and_then(|index| row.get(index))
                .is_some_and(|status| {
                    status.eq_ignore_ascii_case("joined")
                        || status.eq_ignore_ascii_case("active")
                });
            if joined {
                return Ok(candidate.to_owned());
            }
            fallback.get_or_insert_with(|| candidate.to_owned());
        }
        fallback.ok_or_else(|| AffiliateError::Provider("Awin feed list contains no usable feed".into()))
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
            commission_fixed_minor: None,
            commission_currency: None,
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
        let is_default =
            code.eq_ignore_ascii_case("default") || name.to_ascii_lowercase().contains("default");
        let fixed_amount = if kind.eq_ignore_ascii_case("fix")
            || kind.eq_ignore_ascii_case("fixed")
        {
            item.get("amount")
                .and_then(|v| v.as_f64())
                .filter(|v| v.is_finite() && *v >= 0.0)
        } else {
            None
        };
        let currency = item
            .get("currency")
            .and_then(|v| v.as_str())
            .map(ToOwned::to_owned);
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
            source: "test".into(),
            source_updated_at: None,
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
