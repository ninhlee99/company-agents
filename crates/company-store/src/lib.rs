    pub ttfc_seconds: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyControlRecord {
    pub controls: company_safety_controls::SafetyControls,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevenueGraphSummary {
    pub edge_count: i64,
    pub value_backed_edge_count: i64,
    pub latest_observed_at_epoch: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TikTokConnectionRecord {
    pub company_id: Uuid,
    pub open_id: String,
    pub scopes: String,
    pub token_type: String,
    pub access_token_expires_at_epoch: i64,
    pub refresh_token_expires_at_epoch: i64,
    pub status: String,
    pub last_error: Option<String>,
    pub updated_at: String,