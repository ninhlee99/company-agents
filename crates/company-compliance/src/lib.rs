#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_TEXT: usize = 8_000;
const MAX_KEY: usize = 256;
const MAX_CATEGORIES: usize = 100;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceSurface {
    Content,
    Affiliate,
    Live,
    Advertising,
    Copyright,
    ProductEligibility,
    Claims,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceDecision {
    Allowed,
    Review,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceReason {
    PolicyUnavailable,
    MissingPolicyEvidence,
    MissingDisclosure,
    ProhibitedProduct,
    UnsupportedProduct,
    UnverifiedClaim,
    FakeEngagement,
    Simulcast,
    MissingRightsEvidence,
    HumanReviewRequired,
    AllowedByPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRuleSet {
    pub require_accurate_content: bool,
    pub require_affiliate_disclosure: bool,
    pub reject_fake_engagement: bool,
    pub reject_simulcast: bool,
    pub require_product_eligibility: bool,
    pub require_claim_evidence: bool,
    pub require_rights_evidence: bool,
    pub prohibited_product_categories: Vec<String>,
}

impl PolicyRuleSet {
    pub fn validate(&self) -> Result<(), String> {
        if self.prohibited_product_categories.len() > MAX_CATEGORIES {
            return Err("too many prohibited product categories".into());
        }
        for category in &self.prohibited_product_categories {
            require_text("prohibited_product_category", category, 256)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicySnapshot {
    pub id: Uuid,
    pub company_id: Uuid,
    pub policy_key: String,
    pub platform: String,
    pub jurisdiction: String,
    pub version: String,
    pub source_reference: String,
    pub evidence_hash: String,
    pub observed_at_epoch: i64,
    pub effective_at_epoch: i64,
    pub active: bool,
    pub rules: PolicyRuleSet,
}

impl PolicySnapshot {
    pub fn validate(&self) -> Result<(), String> {
        if self.id == Uuid::nil() || self.company_id == Uuid::nil() {
            return Err("policy snapshot identifiers are required".into());
        }
        require_text("policy_key", &self.policy_key, MAX_KEY)?;
        require_text("platform", &self.platform, 64)?;
        require_text("jurisdiction", &self.jurisdiction, 64)?;
        require_text("version", &self.version, 128)?;
        require_text("source_reference", &self.source_reference, 2048)?;
        require_text("evidence_hash", &self.evidence_hash, 256)?;
        if self.observed_at_epoch <= 0 || self.effective_at_epoch <= 0 {
            return Err("policy snapshot timestamps must be positive".into());
        }
        self.rules.validate()?;
        Ok(())
    }

    pub fn snapshot_key(&self) -> String {
        format!("{}:{}", self.policy_key, self.version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplianceInput {
    pub company_id: Uuid,
    pub surface: ComplianceSurface,
    pub platform: String,
    pub jurisdiction: String,
    pub policy_key: String,
    pub policy_snapshot_key: String,
    pub evidence_ref: String,
    pub text: String,
    pub product_category: Option<String>,
    pub disclosure_present: bool,
    pub claim_evidence_present: bool,
    pub product_eligibility_verified: bool,
    pub simulcast: bool,
    pub fake_engagement_detected: bool,
    pub rights_evidence_present: bool,
}

impl ComplianceInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.company_id == Uuid::nil() {
            return Err("compliance company_id is required".into());
        }
        require_text("platform", &self.platform, 64)?;
        require_text("jurisdiction", &self.jurisdiction, 64)?;
        require_text("policy_key", &self.policy_key, MAX_KEY)?;
        require_text("policy_snapshot_key", &self.policy_snapshot_key, MAX_KEY)?;
        require_text("evidence_ref", &self.evidence_ref, 2048)?;
        require_text("text", &self.text, MAX_TEXT)?;
        if let Some(category) = &self.product_category {
            require_text("product_category", category, 256)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplianceCheck {
    pub id: Uuid,
    pub company_id: Uuid,
    pub policy_snapshot_id: Option<Uuid>,
    pub input: ComplianceInput,
    pub decision: ComplianceDecision,
    pub reason: ComplianceReason,
    pub requires_human: bool,
    pub checked_at_epoch: i64,
}

pub fn evaluate(
    snapshot: Option<&PolicySnapshot>,
    input: &ComplianceInput,
    checked_at_epoch: i64,
) -> Result<ComplianceCheck, String> {
    input.validate()?;
    if checked_at_epoch <= 0 {
        return Err("compliance check timestamp must be positive".into());
    }

    let id = Uuid::new_v4();
    let Some(snapshot) = snapshot else {
        return Ok(check(
            id,
            input,
            None,
            ComplianceDecision::Unknown,
            ComplianceReason::PolicyUnavailable,
            true,
            checked_at_epoch,
        ));
    };

    snapshot.validate()?;
    if snapshot.company_id != input.company_id
        || snapshot.policy_key != input.policy_key
        || snapshot.platform != input.platform
        || snapshot.jurisdiction != input.jurisdiction
        || snapshot.snapshot_key() != input.policy_snapshot_key
    {
        return Err("compliance input does not match policy snapshot".into());
    }
    if !snapshot.active || snapshot.effective_at_epoch > checked_at_epoch {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Unknown,
            ComplianceReason::MissingPolicyEvidence,
            true,
            checked_at_epoch,
        ));
    }

    let rules = &snapshot.rules;
    if rules.require_affiliate_disclosure
        && matches!(input.surface, ComplianceSurface::Content | ComplianceSurface::Affiliate)
        && !input.disclosure_present
    {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Blocked,
            ComplianceReason::MissingDisclosure,
            false,
            checked_at_epoch,
        ));
    }

    if rules.reject_fake_engagement && input.fake_engagement_detected {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Blocked,
            ComplianceReason::FakeEngagement,
            false,
            checked_at_epoch,
        ));
    }

    if rules.reject_simulcast && matches!(input.surface, ComplianceSurface::Live) && input.simulcast {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Blocked,
            ComplianceReason::Simulcast,
            false,
            checked_at_epoch,
        ));
    }

    if rules.require_product_eligibility
        && matches!(
            input.surface,
            ComplianceSurface::Content
                | ComplianceSurface::Affiliate
                | ComplianceSurface::Live
                | ComplianceSurface::ProductEligibility
        )
        && !input.product_eligibility_verified
    {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Blocked,
            ComplianceReason::UnsupportedProduct,
            false,
            checked_at_epoch,
        ));
    }

    if let Some(category) = input.product_category.as_deref() {
        if rules.prohibited_product_categories.iter().any(|item| {
            item.eq_ignore_ascii_case(category.trim())
        }) {
            return Ok(check(
                id,
                input,
                Some(snapshot.id),
                ComplianceDecision::Blocked,
                ComplianceReason::ProhibitedProduct,
                false,
                checked_at_epoch,
            ));
        }
    }

    if rules.require_claim_evidence
        && matches!(input.surface, ComplianceSurface::Content | ComplianceSurface::Affiliate | ComplianceSurface::Advertising | ComplianceSurface::Claims)
        && contains_claim_language(&input.text)
        && !input.claim_evidence_present
    {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Blocked,
            ComplianceReason::UnverifiedClaim,
            false,
            checked_at_epoch,
        ));
    }

    if rules.require_rights_evidence
        && matches!(input.surface, ComplianceSurface::Content | ComplianceSurface::Live | ComplianceSurface::Copyright)
        && !input.rights_evidence_present
    {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Review,
            ComplianceReason::MissingRightsEvidence,
            true,
            checked_at_epoch,
        ));
    }

    if rules.require_accurate_content
        && matches!(input.surface, ComplianceSurface::Content | ComplianceSurface::Affiliate | ComplianceSurface::Live)
        && contains_claim_language(&input.text)
        && !input.claim_evidence_present
    {
        return Ok(check(
            id,
            input,
            Some(snapshot.id),
            ComplianceDecision::Blocked,
            ComplianceReason::UnverifiedClaim,
            false,
            checked_at_epoch,
        ));
    }

    Ok(check(
        id,
        input,
        Some(snapshot.id),
        ComplianceDecision::Allowed,
        ComplianceReason::AllowedByPolicy,
        false,
        checked_at_epoch,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyChangeImpact {
    pub policy_key: String,
    pub platform: String,
    pub jurisdiction: String,
    pub from_version: String,
    pub to_version: String,
    pub changed_rules: Vec<String>,
    pub tightened_rules: Vec<String>,
    pub relaxed_rules: Vec<String>,
    pub affected_workflows: Vec<String>,
    pub requires_revalidation: bool,
}

pub fn analyze_policy_change(
    previous: &PolicySnapshot,
    current: &PolicySnapshot,
) -> Result<PolicyChangeImpact, String> {
    previous.validate()?;
    current.validate()?;
    if previous.company_id != current.company_id
        || previous.policy_key != current.policy_key
        || previous.platform != current.platform
        || previous.jurisdiction != current.jurisdiction
    {
        return Err("policy change comparison requires the same company/policy surface".into());
    }
    if previous.version == current.version {
        return Err("policy change comparison requires different versions".into());
    }

    let changed_rules = diff_rule_sets(&previous.rules, &current.rules);
    let mut tightened_rules = Vec::new();
    let mut relaxed_rules = Vec::new();
    let mut affected_workflows = std::collections::BTreeSet::new();

    for rule in &changed_rules {
        match rule.as_str() {
            "require_accurate_content" => {
                if !previous.rules.require_accurate_content && current.rules.require_accurate_content {
                    tightened_rules.push(rule.clone());
                } else {
                    relaxed_rules.push(rule.clone());
                }
            }
            "require_affiliate_disclosure" => {
                if !previous.rules.require_affiliate_disclosure && current.rules.require_affiliate_disclosure {
                    tightened_rules.push(rule.clone());
                } else {
                    relaxed_rules.push(rule.clone());
                }
            }
            "reject_fake_engagement" => {
                if !previous.rules.reject_fake_engagement && current.rules.reject_fake_engagement {
                    tightened_rules.push(rule.clone());
                } else {
                    relaxed_rules.push(rule.clone());
                }
            }
            "reject_simulcast" => {
                if !previous.rules.reject_simulcast && current.rules.reject_simulcast {
                    tightened_rules.push(rule.clone());
                } else {
                    relaxed_rules.push(rule.clone());
                }
            }
            "require_product_eligibility" => {
                if !previous.rules.require_product_eligibility && current.rules.require_product_eligibility {
                    tightened_rules.push(rule.clone());
                } else {
                    relaxed_rules.push(rule.clone());
                }
            }
            "require_claim_evidence" => {
                if !previous.rules.require_claim_evidence && current.rules.require_claim_evidence {
                    tightened_rules.push(rule.clone());
                } else {
                    relaxed_rules.push(rule.clone());
                }
            }
            "require_rights_evidence" => {
                if !previous.rules.require_rights_evidence && current.rules.require_rights_evidence {
                    tightened_rules.push(rule.clone());
                } else {
                    relaxed_rules.push(rule.clone());
                }
            }
            "prohibited_product_categories" => {}
            _ => {}
        }
        for workflow in workflows_for_rule(rule) {
            affected_workflows.insert((*workflow).to_owned());
        }
    }

    if changed_rules.iter().any(|rule| rule == "prohibited_product_categories") {
        let previous_categories: std::collections::BTreeSet<String> = previous
            .rules
            .prohibited_product_categories
            .iter()
            .map(|value| value.trim().to_ascii_lowercase())
            .collect();
        let current_categories: std::collections::BTreeSet<String> = current
            .rules
            .prohibited_product_categories
            .iter()
            .map(|value| value.trim().to_ascii_lowercase())
            .collect();
        if current_categories.difference(&previous_categories).next().is_some() {
            tightened_rules.push("prohibited_product_categories".into());
        }
        if previous_categories.difference(&current_categories).next().is_some() {
            relaxed_rules.push("prohibited_product_categories".into());
        }
        for workflow in workflows_for_rule("prohibited_product_categories") {
            affected_workflows.insert((*workflow).to_owned());
        }
    }

    Ok(PolicyChangeImpact {
        policy_key: current.policy_key.clone(),
        platform: current.platform.clone(),
        jurisdiction: current.jurisdiction.clone(),
        from_version: previous.version.clone(),
        to_version: current.version.clone(),
        changed_rules,
        tightened_rules,
        relaxed_rules,
        affected_workflows: affected_workflows.into_iter().collect(),
        requires_revalidation: !tightened_rules.is_empty(),
    })
}

fn workflows_for_rule(rule: &str) -> &'static [&'static str] {
    match rule {
        "require_accurate_content" => &["content", "affiliate", "live"],
        "require_affiliate_disclosure" => &["content", "affiliate", "advertising"],
        "reject_fake_engagement" => &["content", "live", "advertising"],
        "reject_simulcast" => &["live"],
        "require_product_eligibility" => &["product_eligibility", "content", "affiliate", "live", "advertising"],
        "require_claim_evidence" => &["claims", "content", "affiliate", "advertising"],
        "require_rights_evidence" => &["copyright", "content", "live", "advertising"],
        "prohibited_product_categories" => &["product_eligibility", "content", "affiliate", "live", "advertising"],
        _ => &[],
    }
}

pub fn diff_rule_sets(previous: &PolicyRuleSet, current: &PolicyRuleSet) -> Vec<String> {
    let mut changed = Vec::new();
    if previous.require_accurate_content != current.require_accurate_content {
        changed.push("require_accurate_content".into());
    }
    if previous.require_affiliate_disclosure != current.require_affiliate_disclosure {
        changed.push("require_affiliate_disclosure".into());
    }
    if previous.reject_fake_engagement != current.reject_fake_engagement {
        changed.push("reject_fake_engagement".into());
    }
    if previous.reject_simulcast != current.reject_simulcast {
        changed.push("reject_simulcast".into());
    }
    if previous.require_product_eligibility != current.require_product_eligibility {
        changed.push("require_product_eligibility".into());
    }
    if previous.require_claim_evidence != current.require_claim_evidence {
        changed.push("require_claim_evidence".into());
    }
    if previous.require_rights_evidence != current.require_rights_evidence {
        changed.push("require_rights_evidence".into());
    }
    if previous.prohibited_product_categories != current.prohibited_product_categories {
        changed.push("prohibited_product_categories".into());
    }
    changed
}

fn check(
    id: Uuid,
    input: &ComplianceInput,
    snapshot_id: Option<Uuid>,
    decision: ComplianceDecision,
    reason: ComplianceReason,
    requires_human: bool,
    checked_at_epoch: i64,
) -> ComplianceCheck {
    ComplianceCheck {
        id,
        company_id: input.company_id,
        policy_snapshot_id: snapshot_id,
        input: input.clone(),
        decision,
        reason,
        requires_human,
        checked_at_epoch,
    }
}

fn contains_claim_language(text: &str) -> bool {
    let normalized = text.to_lowercase();
    [
        "100%",
        "chắc chắn",
        "cam kết",
        "đảm bảo",
        "tốt nhất",
        "số 1",
        "hiệu quả",
        "trị",
        "giảm",
        "hoàn toàn",
        "không bao giờ",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn require_text(field: &str, value: &str, max: usize) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} is required"));
    }
    if trimmed.len() > max {
        return Err(format!("{field} exceeds {max} bytes"));
    }
    if trimmed.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err(format!("{field} contains control characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> PolicySnapshot {
        PolicySnapshot {
            id: Uuid::new_v4(),
            company_id: Uuid::new_v4(),
            policy_key: "TIKTOK_SHOP_VN".into(),
            platform: "TIKTOK_SHOP".into(),
            jurisdiction: "VN".into(),
            version: "2026-09-22".into(),
            source_reference: "policy://verified".into(),
            evidence_hash: "sha256:verified".into(),
            observed_at_epoch: 1_750_000_000,
            effective_at_epoch: 1_750_000_000,
            active: true,
            rules: PolicyRuleSet {
                require_accurate_content: true,
                require_affiliate_disclosure: true,
                reject_fake_engagement: true,
                reject_simulcast: true,
                require_product_eligibility: true,
                require_claim_evidence: true,
                require_rights_evidence: true,
                prohibited_product_categories: vec!["prohibited".into()],
            },
        }
    }

    fn input(company_id: Uuid) -> ComplianceInput {
        ComplianceInput {
            company_id,
            surface: ComplianceSurface::Content,
            platform: "TIKTOK_SHOP".into(),
            jurisdiction: "VN".into(),
            policy_key: "TIKTOK_SHOP_VN".into(),
            policy_snapshot_key: "TIKTOK_SHOP_VN:2026-09-22".into(),
            evidence_ref: "content-evidence".into(),
            text: "Giá tốt".into(),
            product_category: Some("standard".into()),
            disclosure_present: true,
            claim_evidence_present: false,
            product_eligibility_verified: true,
            simulcast: false,
            fake_engagement_detected: false,
            rights_evidence_present: true,
        }
    }

    #[test]
    fn missing_policy_is_unknown_and_human_gated() {
        let company = Uuid::new_v4();
        let result = evaluate(None, &input(company), 1_750_000_010).unwrap();
        assert_eq!(result.decision, ComplianceDecision::Unknown);
        assert!(result.requires_human);
    }

    #[test]
    fn missing_disclosure_is_blocked() {
        let company = Uuid::new_v4();
        let mut value = input(company);
        value.disclosure_present = false;
        let result = evaluate(Some(&snapshot_for(company)), &value, 1_750_000_010).unwrap();
        assert_eq!(result.reason, ComplianceReason::MissingDisclosure);
        assert_eq!(result.decision, ComplianceDecision::Blocked);
    }

    #[test]
    fn prohibited_product_is_blocked() {
        let company = Uuid::new_v4();
        let mut value = input(company);
        value.product_category = Some("prohibited".into());
        let result = evaluate(Some(&snapshot_for(company)), &value, 1_750_000_010).unwrap();
        assert_eq!(result.reason, ComplianceReason::ProhibitedProduct);
    }

    #[test]
    fn unverified_claim_is_blocked() {
        let company = Uuid::new_v4();
        let mut value = input(company);
        value.text = "Sản phẩm này hiệu quả 100%".into();
        let result = evaluate(Some(&snapshot_for(company)), &value, 1_750_000_010).unwrap();
        assert_eq!(result.reason, ComplianceReason::UnverifiedClaim);
    }

    #[test]
    fn simulcast_is_blocked_for_live() {
        let company = Uuid::new_v4();
        let mut value = input(company);
        value.surface = ComplianceSurface::Live;
        value.simulcast = true;
        let result = evaluate(Some(&snapshot_for(company)), &value, 1_750_000_010).unwrap();
        assert_eq!(result.reason, ComplianceReason::Simulcast);
    }

    #[test]
    fn future_policy_is_unknown() {
        let company = Uuid::new_v4();
        let mut value = snapshot_for(company);
        value.effective_at_epoch = 2_000_000_000;
        let result = evaluate(Some(&value), &input(company), 1_750_000_010).unwrap();
        assert_eq!(result.decision, ComplianceDecision::Unknown);
        assert!(result.requires_human);
    }

    #[test]
    fn missing_rights_evidence_requires_review() {
        let company = Uuid::new_v4();
        let mut value = input(company);
        value.rights_evidence_present = false;
        let result = evaluate(Some(&snapshot_for(company)), &value, 1_750_000_010).unwrap();
        assert_eq!(result.decision, ComplianceDecision::Review);
        assert_eq!(result.reason, ComplianceReason::MissingRightsEvidence);
        assert!(result.requires_human);
    }

    #[test]
    fn policy_snapshot_mismatch_is_rejected() {
        let company = Uuid::new_v4();
        let mut value = input(company);
        value.policy_snapshot_key = "TIKTOK_SHOP_VN:old".into();
        assert!(evaluate(Some(&snapshot_for(company)), &value, 1_750_000_010).is_err());
    }

    #[test]
    fn policy_change_impact_flags_tightening_and_affected_workflows() {
        let company = Uuid::new_v4();
        let previous = snapshot_for(company);
        let mut current = previous.clone();
        current.version = "2026-10-01".into();
        current.rules.require_affiliate_disclosure = false;
        current.rules.prohibited_product_categories.push("new-prohibited".into());
        let impact = analyze_policy_change(&previous, &current).unwrap();
        assert!(impact.relaxed_rules.contains(&"require_affiliate_disclosure".into()));
        assert!(impact.tightened_rules.contains(&"prohibited_product_categories".into()));
        assert!(impact.requires_revalidation);
        assert!(impact.affected_workflows.contains(&"affiliate".into()));
        assert!(impact.affected_workflows.contains(&"product_eligibility".into()));
    }

    #[test]
    fn policy_change_impact_rejects_cross_surface_comparison() {
        let company = Uuid::new_v4();
        let previous = snapshot_for(company);
        let mut current = snapshot_for(company);
        current.version = "2026-10-02".into();
        current.platform = "OTHER".into();
        assert!(analyze_policy_change(&previous, &current).is_err());
    }

    #[test]
    fn rules_can_be_diffed() {
        let company = Uuid::new_v4();
        let previous = snapshot_for(company);
        let mut current = previous.clone();
        current.version = "new".into();
        current.rules.reject_simulcast = false;
        current.rules.prohibited_product_categories.push("new".into());
        let changed = diff_rule_sets(&previous.rules, &current.rules);
        assert!(changed.contains(&"reject_simulcast".into()));
        assert!(changed.contains(&"prohibited_product_categories".into()));
    }

    fn snapshot_for(company_id: Uuid) -> PolicySnapshot {
        let mut value = snapshot();
        value.company_id = company_id;
        value
    }
}
