#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus { Pending, Confirmed, Refunded }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributionModel { LastClick, FirstClick }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClickTouch {
    pub click_id: String,
    pub content_id: Uuid,
    pub creator_id: Uuid,
    pub product_id: String,
    pub occurred_at_epoch: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderEvent {
    pub order_id: String,
    pub click_id: Option<String>,
    pub product_id: String,
    pub gross_sales_minor: i128,
    pub commission_minor: i128,
    pub status: OrderStatus,
    pub occurred_at_epoch: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributedOrder {
    pub order_id: String,
    pub content_id: Option<Uuid>,
    pub creator_id: Option<Uuid>,
    pub product_id: String,
    pub gross_sales_minor: i128,
    pub commission_minor: i128,
    pub status: OrderStatus,
    pub attribution_confidence_bps: u32,
    pub attribution_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionResult {
    pub orders: Vec<AttributedOrder>,
    pub total_commission_minor: i128,
    pub total_gross_sales_minor: i128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributionError {
    InvalidInput(String),
}

impl std::fmt::Display for AttributionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(v) => write!(f, "{v}"),
        }
    }
}

fn status_rank(status: OrderStatus) -> u8 {
    match status {
        OrderStatus::Pending => 1,
        OrderStatus::Confirmed => 2,
        OrderStatus::Refunded => 3,
    }
}

pub fn attribute(
    clicks: &[ClickTouch],
    orders: &[OrderEvent],
    model: AttributionModel,
    window_secs: i64,
) -> Result<AttributionResult, AttributionError> {
    if window_secs <= 0 || window_secs > 30 * 86_400 {
        return Err(AttributionError::InvalidInput("attribution window must be 1..=30 days".into()));
    }

    let mut unique_clicks: HashMap<&str, &ClickTouch> = HashMap::new();
    for click in clicks {
        if click.click_id.trim().is_empty() || click.product_id.trim().is_empty() {
            continue;
        }
        unique_clicks.entry(&click.click_id)
            .and_modify(|existing| {
                if click.occurred_at_epoch < existing.occurred_at_epoch {
                    *existing = click;
                }
            })
            .or_insert(click);
    }

    let mut unique_orders: HashMap<&str, &OrderEvent> = HashMap::new();
    for order in orders {
        if order.order_id.trim().is_empty() || order.product_id.trim().is_empty() {
            continue;
        }
        if order.gross_sales_minor < 0 || order.commission_minor < 0 {
            return Err(AttributionError::InvalidInput("order amounts cannot be negative".into()));
        }
        unique_orders
            .entry(&order.order_id)
            .and_modify(|existing| {
                if (order.occurred_at_epoch, status_rank(order.status))
                    > (existing.occurred_at_epoch, status_rank(existing.status))
                {
                    *existing = order;
                }
            })
            .or_insert(order);
    }

    let mut ordered: Vec<&OrderEvent> = unique_orders.into_values().collect();
    ordered.sort_by(|a,b| a.order_id.cmp(&b.order_id));

    let mut out = Vec::with_capacity(ordered.len());
    for order in ordered {
        let mut match_click = order.click_id.as_deref()
            .and_then(|id| unique_clicks.get(id).copied())
            .filter(|click| click.product_id == order.product_id);

        if match_click.is_none() {
            let mut eligible: Vec<&ClickTouch> = unique_clicks.values()
                .copied()
                .filter(|click| {
                    click.product_id == order.product_id
                        && click.occurred_at_epoch <= order.occurred_at_epoch
                        && order.occurred_at_epoch.saturating_sub(click.occurred_at_epoch) <= window_secs
                })
                .collect();
            eligible.sort_by(|a,b| a.occurred_at_epoch.cmp(&b.occurred_at_epoch).then_with(|| a.click_id.cmp(&b.click_id)));
            match_click = match model {
                AttributionModel::LastClick => eligible.last().copied(),
                AttributionModel::FirstClick => eligible.first().copied(),
            };
        }

        let (content_id, creator_id, confidence, reason) = match match_click {
            Some(click) if order.click_id.as_deref() == Some(click.click_id.as_str()) => {
                (Some(click.content_id), Some(click.creator_id), 10_000, "direct-click-id")
            }
            Some(click) => {
                (Some(click.content_id), Some(click.creator_id), 7_500, "deterministic-window-match")
            }
            None => (None, None, 0, "unattributed"),
        };

        let signed_commission = match order.status {
            OrderStatus::Refunded => order.commission_minor.saturating_neg(),
            OrderStatus::Pending | OrderStatus::Confirmed => order.commission_minor,
        };
        let signed_sales = match order.status {
            OrderStatus::Refunded => order.gross_sales_minor.saturating_neg(),
            OrderStatus::Pending | OrderStatus::Confirmed => order.gross_sales_minor,
        };

        out.push(AttributedOrder {
            order_id: order.order_id.clone(),
            content_id,
            creator_id,
            product_id: order.product_id.clone(),
            gross_sales_minor: signed_sales,
            commission_minor: signed_commission,
            status: order.status,
            attribution_confidence_bps: confidence,
            attribution_reason: reason.into(),
        });
    }

    let total_commission_minor = out.iter().fold(0_i128, |acc, order| acc.saturating_add(order.commission_minor));
    let total_gross_sales_minor = out.iter().fold(0_i128, |acc, order| acc.saturating_add(order.gross_sales_minor));

    Ok(AttributionResult { orders: out, total_commission_minor, total_gross_sales_minor })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> (Uuid, Uuid) { (Uuid::new_v4(), Uuid::new_v4()) }

    #[test]
    fn last_click_is_deterministic() {
        let (content_a, creator) = ids();
        let (content_b, _) = ids();
        let clicks = vec![
            ClickTouch { click_id:"a".into(), content_id:content_a, creator_id:creator, product_id:"p".into(), occurred_at_epoch:100 },
            ClickTouch { click_id:"b".into(), content_id:content_b, creator_id:creator, product_id:"p".into(), occurred_at_epoch:110 },
        ];
        let order = OrderEvent {
            order_id:"o1".into(), click_id:None, product_id:"p".into(),
            gross_sales_minor:1000, commission_minor:100, status:OrderStatus::Confirmed, occurred_at_epoch:120
        };
        let result = attribute(&clicks, &[order], AttributionModel::LastClick, 60).unwrap();
        assert_eq!(result.orders[0].content_id, Some(content_b));
        assert_eq!(result.total_commission_minor, 100);
    }

    #[test]
    fn first_click_and_window_are_respected() {
        let (content, creator) = ids();
        let clicks = vec![
            ClickTouch { click_id:"a".into(), content_id:content, creator_id:creator, product_id:"p".into(), occurred_at_epoch:10 },
            ClickTouch { click_id:"b".into(), content_id:content, creator_id:creator, product_id:"p".into(), occurred_at_epoch:20 },
        ];
        let order = OrderEvent {
            order_id:"o1".into(), click_id:None, product_id:"p".into(),
            gross_sales_minor:100, commission_minor:10, status:OrderStatus::Confirmed, occurred_at_epoch:30
        };
        let result = attribute(&clicks, &[order], AttributionModel::FirstClick, 60).unwrap();
        assert_eq!(result.orders[0].attribution_reason, "deterministic-window-match");
        assert_eq!(result.orders[0].content_id, Some(content));
    }

    #[test]
    fn refund_reverses_economic_value() {
        let (content, creator) = ids();
        let click = ClickTouch { click_id:"c".into(), content_id:content, creator_id:creator, product_id:"p".into(), occurred_at_epoch:100 };
        let order = OrderEvent {
            order_id:"o1".into(), click_id:Some("c".into()), product_id:"p".into(),
            gross_sales_minor:1000, commission_minor:100, status:OrderStatus::Refunded, occurred_at_epoch:120
        };
        let result = attribute(&[click], &[order], AttributionModel::LastClick, 60).unwrap();
        assert_eq!(result.total_commission_minor, -100);
        assert_eq!(result.total_gross_sales_minor, -1000);
    }

    #[test]
    fn refund_state_supersedes_confirmed_state() {
        let (content, creator) = ids();
        let click = ClickTouch { click_id:"c".into(), content_id:content, creator_id:creator, product_id:"p".into(), occurred_at_epoch:100 };
        let confirmed = OrderEvent {
            order_id:"o1".into(), click_id:Some("c".into()), product_id:"p".into(),
            gross_sales_minor:1000, commission_minor:100, status:OrderStatus::Confirmed, occurred_at_epoch:120
        };
        let refunded = OrderEvent {
            order_id:"o1".into(), click_id:Some("c".into()), product_id:"p".into(),
            gross_sales_minor:1000, commission_minor:100, status:OrderStatus::Refunded, occurred_at_epoch:150
        };
        let result = attribute(&[click], &[confirmed, refunded], AttributionModel::LastClick, 60).unwrap();
        assert_eq!(result.orders.len(), 1);
        assert_eq!(result.total_commission_minor, -100);
    }

    #[test]
    fn duplicate_order_is_idempotent() {
        let (content, creator) = ids();
        let click = ClickTouch { click_id:"c".into(), content_id:content, creator_id:creator, product_id:"p".into(), occurred_at_epoch:100 };
        let order = OrderEvent {
            order_id:"o1".into(), click_id:Some("c".into()), product_id:"p".into(),
            gross_sales_minor:1000, commission_minor:100, status:OrderStatus::Confirmed, occurred_at_epoch:120
        };
        let result = attribute(&[click], &[order.clone(), order], AttributionModel::LastClick, 60).unwrap();
        assert_eq!(result.orders.len(), 1);
        assert_eq!(result.total_commission_minor, 100);
    }
}
