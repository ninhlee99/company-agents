#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClickEvent {
    pub click_id: String,
    pub company_id: String,
    pub product_id: String,
    pub advertiser_id: String,
    pub content_id: String,
    pub occurred_at: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversionEvent {
    pub company_id: String,
    pub conversion_id: String,
    pub click_id: Option<String>,
    pub order_id: String,
    pub product_id: String,
    pub advertiser_id: String,
    pub occurred_at: String,
    pub order_value_minor: i128,
    pub commission_minor: i128,
    pub refunded_minor: i128,
    pub cancelled: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttributionModel {
    LastClick,
    FirstClick,
    Linear,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Attribution {
    pub click_id: String,
    pub product_id: String,
    pub content_id: String,
    pub attributed_order_value_minor: i128,
    pub attributed_commission_minor: i128,
    pub confidence_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconciledConversion {
    pub conversion_id: String,
    pub attributed: Vec<Attribution>,
    pub net_commission_minor: i128,
    pub reconciliation_variance_minor: i128,
    pub status: ReconciliationStatus,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReconciliationStatus {
    Verified,
    Partial,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributionError {
    InvalidInput(String),
    Overflow,
}

impl std::fmt::Display for AttributionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(v) => write!(f, "invalid attribution input: {v}"),
            Self::Overflow => write!(f, "attribution arithmetic overflow"),
        }
    }
}

impl std::error::Error for AttributionError {}

pub fn attribute_conversion(
    conversion: &ConversionEvent,
    clicks: &[ClickEvent],
    model: AttributionModel,
) -> Result<ReconciledConversion, AttributionError> {
    if conversion.company_id.trim().is_empty()
        || conversion.conversion_id.trim().is_empty()
        || conversion.order_id.trim().is_empty()
        || conversion.product_id.trim().is_empty()
    {
        return Err(AttributionError::InvalidInput(
            "conversion identifiers are required".into(),
        ));
    }
    if conversion.order_value_minor < 0
        || conversion.commission_minor < 0
        || conversion.refunded_minor < 0
        || conversion.refunded_minor > conversion.order_value_minor
    {
        return Err(AttributionError::InvalidInput(
            "conversion economic values are invalid".into(),
        ));
    }
    let conversion_time = parse_rfc3339(&conversion.occurred_at).ok_or_else(|| {
        AttributionError::InvalidInput("conversion occurred_at must be RFC3339".into())
    })?;

    let mut seen_clicks = HashSet::new();
    let mut matching = clicks
        .iter()
        .filter(|click| click.company_id == conversion.company_id)
        .filter(|click| click.product_id == conversion.product_id)
        .filter(|click| click.advertiser_id == conversion.advertiser_id)
        .filter_map(|click| {
            let click_time = parse_rfc3339(&click.occurred_at)?;
            if click_time <= conversion_time && seen_clicks.insert(click.click_id.clone()) {
                Some((click_time, click.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    matching.sort_by(|(a_time, a), (b_time, b)| {
        a_time.cmp(b_time).then_with(|| a.click_id.cmp(&b.click_id))
    });
    let matching = matching
        .into_iter()
        .map(|(_, click)| click)
        .collect::<Vec<_>>();

    let selected = match (&conversion.click_id, model) {
        (Some(id), _) => matching
            .into_iter()
            .filter(|c| &c.click_id == id)
            .collect::<Vec<_>>(),
        (None, AttributionModel::LastClick) => matching.into_iter().rev().take(1).collect(),
        (None, AttributionModel::FirstClick) => matching.into_iter().take(1).collect(),
        (None, AttributionModel::Linear) => matching,
    };

    if selected.is_empty() {
        return Ok(ReconciledConversion {
            conversion_id: conversion.conversion_id.clone(),
            attributed: Vec::new(),
            net_commission_minor: 0,
            reconciliation_variance_minor: conversion.commission_minor,
            status: ReconciliationStatus::Partial,
            idempotency_key: conversion_idempotency_key(conversion),
        });
    }

    let net_order_value = conversion
        .order_value_minor
        .checked_sub(conversion.refunded_minor)
        .ok_or(AttributionError::Overflow)?;
    let net_commission = if conversion.cancelled {
        0
    } else {
        conversion.commission_minor
    };

    let n = selected.len() as i128;
    let mut allocated_value = 0_i128;
    let mut allocated_commission = 0_i128;
    let mut out = Vec::with_capacity(selected.len());

    for (index, click) in selected.iter().enumerate() {
        let value = if index + 1 == selected.len() {
            net_order_value
                .checked_sub(allocated_value)
                .ok_or(AttributionError::Overflow)?
        } else {
            net_order_value
                .checked_div(n)
                .ok_or(AttributionError::Overflow)?
        };
        let commission = if index + 1 == selected.len() {
            net_commission
                .checked_sub(allocated_commission)
                .ok_or(AttributionError::Overflow)?
        } else {
            net_commission
                .checked_div(n)
                .ok_or(AttributionError::Overflow)?
        };
        allocated_value = allocated_value
            .checked_add(value)
            .ok_or(AttributionError::Overflow)?;
        allocated_commission = allocated_commission
            .checked_add(commission)
            .ok_or(AttributionError::Overflow)?;

        out.push(Attribution {
            click_id: click.click_id.clone(),
            product_id: click.product_id.clone(),
            content_id: click.content_id.clone(),
            attributed_order_value_minor: value,
            attributed_commission_minor: commission,
            confidence_bps: if conversion.click_id.is_some() {
                10_000
            } else {
                8_000
            },
        });
    }

    let variance = conversion
        .commission_minor
        .checked_sub(allocated_commission)
        .ok_or(AttributionError::Overflow)?;

    Ok(ReconciledConversion {
        conversion_id: conversion.conversion_id.clone(),
        attributed: out,
        net_commission_minor: allocated_commission,
        reconciliation_variance_minor: variance,
        status: if variance == 0 {
            ReconciliationStatus::Verified
        } else {
            ReconciliationStatus::Partial
        },
        idempotency_key: conversion_idempotency_key(conversion),
    })
}

fn parse_rfc3339(value: &str) -> Option<time::OffsetDateTime> {
    time::OffsetDateTime::parse(value.trim(), &time::format_description::well_known::Rfc3339).ok()
}

pub fn conversion_idempotency_key(conversion: &ConversionEvent) -> String {
    let canonical = serde_json::to_vec(conversion).unwrap_or_default();
    let digest = Sha256::digest(canonical);
    format!("conversion:{:x}", digest)
}

pub fn summarize_by_content(records: &[ReconciledConversion]) -> HashMap<String, (i128, i128)> {
    let mut out = HashMap::new();
    for record in records {
        for item in &record.attributed {
            let entry = out
                .entry(item.content_id.clone())
                .or_insert((0_i128, 0_i128));
            entry.0 = entry.0.saturating_add(item.attributed_order_value_minor);
            entry.1 = entry.1.saturating_add(item.attributed_commission_minor);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn click(id: &str, content: &str, at: &str) -> ClickEvent {
        ClickEvent {
            click_id: id.into(),
            company_id: "c".into(),
            product_id: "p".into(),
            advertiser_id: "a".into(),
            content_id: content.into(),
            occurred_at: at.into(),
            source: "test".into(),
        }
    }

    fn conversion(id: &str) -> ConversionEvent {
        ConversionEvent {
            company_id: "c".into(),
            conversion_id: id.into(),
            click_id: None,
            order_id: format!("order-{id}"),
            product_id: "p".into(),
            advertiser_id: "a".into(),
            occurred_at: "2026-09-27T10:00:00Z".into(),
            order_value_minor: 1_001,
            commission_minor: 101,
            refunded_minor: 0,
            cancelled: false,
            source: "test".into(),
        }
    }

    #[test]
    fn last_click_attribution_is_deterministic() {
        let result = attribute_conversion(
            &conversion("1"),
            &[
                click("c1", "video-a", "2026-09-27T09:00:00Z"),
                click("c2", "video-b", "2026-09-27T09:30:00Z"),
            ],
            AttributionModel::LastClick,
        )
        .unwrap();
        assert_eq!(result.attributed.len(), 1);
        assert_eq!(result.attributed[0].content_id, "video-b");
        assert_eq!(result.net_commission_minor, 101);
        assert_eq!(result.status, ReconciliationStatus::Verified);
    }

    #[test]
    fn linear_model_conserves_order_value_and_commission() {
        let result = attribute_conversion(
            &conversion("2"),
            &[
                click("c1", "a", "2026-09-27T09:00:00Z"),
                click("c2", "b", "2026-09-27T09:30:00Z"),
            ],
            AttributionModel::Linear,
        )
        .unwrap();
        assert_eq!(
            result
                .attributed
                .iter()
                .map(|v| v.attributed_order_value_minor)
                .sum::<i128>(),
            1_001
        );
        assert_eq!(
            result
                .attributed
                .iter()
                .map(|v| v.attributed_commission_minor)
                .sum::<i128>(),
            101
        );
    }

    #[test]
    fn refund_and_cancel_are_net_of_revenue() {
        let mut c = conversion("3");
        c.order_value_minor = 1_000;
        c.refunded_minor = 400;
        c.commission_minor = 100;
        let result =
            attribute_conversion(&c, &[click("c1", "a", "2026-09-27T09:00:00Z")], AttributionModel::LastClick)
                .unwrap();
        assert_eq!(result.attributed[0].attributed_order_value_minor, 600);

        c.cancelled = true;
        let result =
            attribute_conversion(&c, &[click("c1", "a", "t")], AttributionModel::LastClick)
                .unwrap();
        assert_eq!(result.net_commission_minor, 0);
    }

    #[test]
    fn explicit_click_has_maximum_confidence() {
        let mut c = conversion("4");
        c.click_id = Some("c1".into());
        let result =
            attribute_conversion(&c, &[click("c1", "a", "t")], AttributionModel::LastClick)
                .unwrap();
        assert_eq!(result.attributed[0].confidence_bps, 10_000);
    }

    #[test]
    fn invalid_conversion_timestamp_is_rejected() {
        let mut event = conversion("6");
        event.occurred_at = "not-a-date".into();
        assert!(matches!(
            attribute_conversion(&event, &[], AttributionModel::LastClick),
            Err(AttributionError::InvalidInput(_))
        ));
    }

    #[test]
    fn duplicate_click_ids_do_not_double_count_linear_attribution() {
        let event = conversion("7");
        let clicks = vec![
            click("c1", "a", "2026-09-27T09:00:00Z"),
            click("c1", "a", "2026-09-27T09:30:00Z"),
            click("c2", "b", "2026-09-27T09:45:00Z"),
        ];
        let result = attribute_conversion(&event, &clicks, AttributionModel::Linear).unwrap();
        assert_eq!(result.attributed.len(), 2);
        assert_eq!(
            result
                .attributed
                .iter()
                .map(|v| v.attributed_order_value_minor)
                .sum::<i128>(),
            1_001
        );
    }

    #[test]
    fn unmatched_conversion_is_partial_not_invented() {
        let result =
            attribute_conversion(&conversion("5"), &[], AttributionModel::LastClick).unwrap();
        assert_eq!(result.status, ReconciliationStatus::Partial);
        assert_eq!(result.net_commission_minor, 0);
        assert_eq!(result.reconciliation_variance_minor, 101);
    }
}
