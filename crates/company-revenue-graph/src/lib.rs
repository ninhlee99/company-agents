#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_REF_LEN: usize = 512;
const MAX_SOURCE_LEN: usize = 256;
const MAX_EVIDENCE_LEN: usize = 1024;
const MAX_RELATION_LEN: usize = 128;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RevenueNodeType {
    Cash,
    Commission,
    Order,
    Product,
    Campaign,
    Content,
    Hook,
    Creator,
    Audience,
    Traffic,
    Experiment,
    Decision,
    Trend,
}

impl RevenueNodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "CASH",
            Self::Commission => "COMMISSION",
            Self::Order => "ORDER",
            Self::Product => "PRODUCT",
            Self::Campaign => "CAMPAIGN",
            Self::Content => "CONTENT",
            Self::Hook => "HOOK",
            Self::Creator => "CREATOR",
            Self::Audience => "AUDIENCE",
            Self::Traffic => "TRAFFIC",
            Self::Experiment => "EXPERIMENT",
            Self::Decision => "DECISION",
            Self::Trend => "TREND",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "CASH" => Some(Self::Cash),
            "COMMISSION" => Some(Self::Commission),
            "ORDER" => Some(Self::Order),
            "PRODUCT" => Some(Self::Product),
            "CAMPAIGN" => Some(Self::Campaign),
            "CONTENT" => Some(Self::Content),
            "HOOK" => Some(Self::Hook),
            "CREATOR" => Some(Self::Creator),
            "AUDIENCE" => Some(Self::Audience),
            "TRAFFIC" => Some(Self::Traffic),
            "EXPERIMENT" => Some(Self::Experiment),
            "DECISION" => Some(Self::Decision),
            "TREND" => Some(Self::Trend),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevenueGraphEdge {
    pub id: Uuid,
    pub company_id: Uuid,
    pub edge_key: String,
    pub from_type: RevenueNodeType,
    pub from_ref: String,
    pub relation: String,
    pub to_type: RevenueNodeType,
    pub to_ref: String,
    pub value_minor: Option<i128>,
    pub currency: Option<String>,
    pub confidence_bps: u32,
    pub evidence_ref: String,
    pub source: String,
    pub observed_at_epoch: i64,
}

impl RevenueGraphEdge {
    pub fn deterministic_id(company_id: Uuid, edge_key: &str) -> Uuid {
        Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("company-revenue-graph:{company_id}:{edge_key}").as_bytes(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevenueGraphPath {
    pub depth: u32,
    pub edge: RevenueGraphEdge,
}

pub fn validate_edge(edge: &RevenueGraphEdge) -> Result<(), String> {
    if edge.id == Uuid::nil() || edge.company_id == Uuid::nil() {
        return Err("graph ids are required".into());
    }
    require_text("edge_key", &edge.edge_key, MAX_REF_LEN)?;
    require_text("from_ref", &edge.from_ref, MAX_REF_LEN)?;
    require_text("to_ref", &edge.to_ref, MAX_REF_LEN)?;
    require_text("relation", &edge.relation, MAX_RELATION_LEN)?;
    let canonical_key = build_edge_key(
        edge.from_type,
        &edge.from_ref,
        &edge.relation,
        edge.to_type,
        &edge.to_ref,
    );
    if edge.edge_key != canonical_key {
        return Err("edge_key must match the canonical graph edge fields".into());
    }
    require_text("evidence_ref", &edge.evidence_ref, MAX_EVIDENCE_LEN)?;
    require_text("source", &edge.source, MAX_SOURCE_LEN)?;
    if edge.from_ref == edge.to_ref && edge.from_type == edge.to_type {
        return Err("graph edges cannot self-loop".into());
    }
    if edge.confidence_bps > 10_000 {
        return Err("confidence_bps must be between 0 and 10000".into());
    }
    if edge.observed_at_epoch <= 0 {
        return Err("observed_at_epoch must be positive".into());
    }
    match edge.value_minor {
        Some(value) if value < 0 => return Err("graph value cannot be negative".into()),
        Some(_) if edge.currency.as_deref().is_none_or(|value| value.len() != 3) => {
            return Err("currency is required with value_minor and must be 3 characters".into())
        }
        Some(_) => {}
        None if edge.currency.is_some() => {
            return Err("currency is not allowed without value_minor".into())
        }
        None => {}
    }
    if let Some(currency) = edge.currency.as_deref() {
        if currency.chars().any(|c| !c.is_ascii_uppercase()) {
            return Err("currency must be uppercase ISO-style text".into());
        }
    }
    Ok(())
}

pub fn build_edge_key(
    from_type: RevenueNodeType,
    from_ref: &str,
    relation: &str,
    to_type: RevenueNodeType,
    to_ref: &str,
) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        from_type.as_str(),
        from_ref,
        relation.trim(),
        to_type.as_str(),
        to_ref
    )
}

fn require_text(field: &str, value: &str, max: usize) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} is required"));
    }
    if trimmed.len() > max {
        return Err(format!("{field} exceeds {max} bytes"));
    }
    if trimmed.bytes().any(|byte| byte == 0) {
        return Err(format!("{field} contains a NUL byte"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge() -> RevenueGraphEdge {
        let company = Uuid::new_v4();
        let key = build_edge_key(
            RevenueNodeType::Order,
            "order-1",
            "GENERATES",
            RevenueNodeType::Commission,
            "commission-1",
        );
        RevenueGraphEdge {
            id: RevenueGraphEdge::deterministic_id(company, &key),
            company_id: company,
            edge_key: key,
            from_type: RevenueNodeType::Order,
            from_ref: "order-1".into(),
            relation: "GENERATES".into(),
            to_type: RevenueNodeType::Commission,
            to_ref: "commission-1".into(),
            value_minor: Some(100),
            currency: Some("VND".into()),
            confidence_bps: 10_000,
            evidence_ref: "provider:conversion-1".into(),
            source: "affiliate".into(),
            observed_at_epoch: 1_760_000_000,
        }
    }

    #[test]
    fn deterministic_ids_are_stable() {
        let company = Uuid::new_v4();
        let key = "ORDER:o:GENERATES:COMMISSION:c";
        assert_eq!(
            RevenueGraphEdge::deterministic_id(company, key),
            RevenueGraphEdge::deterministic_id(company, key)
        );
    }

    #[test]
    fn edge_key_must_match_graph_fields() {
        let mut value = edge();
        value.edge_key = "ORDER:order-1:GENERATES:COMMISSION:other".into();
        assert!(validate_edge(&value).is_err());

        value = edge();
        value.from_ref = " order-1 ".into();
        assert!(validate_edge(&value).is_err());
    }

    #[test]
    fn invalid_edges_fail_closed() {
        let mut value = edge();
        value.confidence_bps = 10_001;
        assert!(validate_edge(&value).is_err());

        value = edge();
        value.value_minor = Some(-1);
        assert!(validate_edge(&value).is_err());

        value = edge();
        value.currency = Some("usd".into());
        assert!(validate_edge(&value).is_err());
    }

    #[test]
    fn empty_value_cannot_carry_currency() {
        let mut value = edge();
        value.value_minor = None;
        value.currency = Some("VND".into());
        assert!(validate_edge(&value).is_err());
    }

    #[test]
    fn json_roundtrip_is_lossless() {
        let value = edge();
        let encoded = serde_json::to_string(&value).unwrap();
        let decoded: RevenueGraphEdge = serde_json::from_str(&encoded).unwrap();
        assert_eq!(value, decoded);
    }
}
