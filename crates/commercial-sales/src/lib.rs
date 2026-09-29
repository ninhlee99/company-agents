#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod messaging;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalStatus { Draft, Sent, Accepted, Rejected, Expired }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus { Draft, Issued, PartiallyPaid, Paid, Void }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceProposal {
    pub id: Uuid,
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub title: String,
    pub currency: String,
    pub total_minor: i128,
    pub status: ProposalStatus,
    pub valid_until_epoch: i64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sponsorship {
    pub id: Uuid,
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub title: String,
    pub currency: String,
    pub committed_minor: i128,
    pub delivered_minor: i128,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Invoice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub currency: String,
    pub subtotal_minor: i128,
    pub paid_minor: i128,
    pub status: InvoiceStatus,
    pub due_epoch: i64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvoiceLine {
    pub description: String,
    pub quantity: u32,
    pub unit_price_minor: i128,
}

impl InvoiceLine {
    pub fn total_minor(&self) -> Result<i128, String> {
        self.unit_price_minor.checked_mul(self.quantity as i128)
            .ok_or_else(|| "invoice line total overflow".into())
    }
}

pub fn invoice_total(lines: &[InvoiceLine]) -> Result<i128, String> {
    lines.iter().try_fold(0i128, |total, line| {
        total.checked_add(line.total_minor()?)
            .ok_or_else(|| "invoice total overflow".into())
    })
}

pub fn transition_invoice(status: InvoiceStatus, paid_minor: i128, total_minor: i128) -> Result<InvoiceStatus, String> {
    if total_minor < 0 || paid_minor < 0 || paid_minor > total_minor {
        return Err("invalid invoice payment state".into());
    }
    if status == InvoiceStatus::Void {
        return Err("void invoice cannot receive payments".into());
    }
    if status == InvoiceStatus::Draft {
        return Ok(InvoiceStatus::Draft);
    }
    if paid_minor == 0 { Ok(InvoiceStatus::Issued) }
    else if paid_minor < total_minor { Ok(InvoiceStatus::PartiallyPaid) }
    else { Ok(InvoiceStatus::Paid) }
}


pub fn transition_proposal(
    current: ProposalStatus,
    next: ProposalStatus,
) -> Result<ProposalStatus, String> {
    use ProposalStatus::*;
    let allowed = matches!(
        (current, next),
        (Draft, Sent)
            | (Sent, Accepted)
            | (Sent, Rejected)
            | (Sent, Expired)
    );
    if allowed {
        Ok(next)
    } else if current == next {
        Ok(current)
    } else {
        Err(format!("invalid proposal transition: {:?} -> {:?}", current, next))
    }
}

pub fn transition_sponsorship(
    current: &str,
    next: &str,
) -> Result<String, String> {
    let allowed = matches!(
        (current, next),
        ("PROSPECT", "CONTRACTED")
            | ("CONTRACTED", "DELIVERING")
            | ("DELIVERING", "COMPLETED")
            | ("PROSPECT", "CANCELLED")
            | ("CONTRACTED", "CANCELLED")
            | ("DELIVERING", "CANCELLED")
    );
    if allowed || current == next {
        Ok(next.to_owned())
    } else {
        Err(format!("invalid sponsorship transition: {current} -> {next}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_are_checked() {
        let lines = vec![
            InvoiceLine { description: "Retainer".into(), quantity: 2, unit_price_minor: 1500 },
            InvoiceLine { description: "Setup".into(), quantity: 1, unit_price_minor: 500 },
        ];
        assert_eq!(invoice_total(&lines).unwrap(), 3500);
    }

    #[test]
    fn proposal_lifecycle_is_deterministic() {
        assert_eq!(transition_proposal(ProposalStatus::Draft, ProposalStatus::Sent).unwrap(), ProposalStatus::Sent);
        assert_eq!(transition_proposal(ProposalStatus::Sent, ProposalStatus::Accepted).unwrap(), ProposalStatus::Accepted);
        assert!(transition_proposal(ProposalStatus::Accepted, ProposalStatus::Sent).is_err());
    }

    #[test]
    fn sponsorship_delivery_lifecycle_is_deterministic() {
        assert_eq!(transition_sponsorship("PROSPECT", "CONTRACTED").unwrap(), "CONTRACTED");
        assert_eq!(transition_sponsorship("CONTRACTED", "DELIVERING").unwrap(), "DELIVERING");
        assert!(transition_sponsorship("COMPLETED", "DELIVERING").is_err());
    }

    #[test]
    fn payment_lifecycle_is_deterministic() {
        assert_eq!(transition_invoice(InvoiceStatus::Issued, 0, 1000).unwrap(), InvoiceStatus::Issued);
        assert_eq!(transition_invoice(InvoiceStatus::Issued, 400, 1000).unwrap(), InvoiceStatus::PartiallyPaid);
        assert_eq!(transition_invoice(InvoiceStatus::Issued, 1000, 1000).unwrap(), InvoiceStatus::Paid);
        assert!(transition_invoice(InvoiceStatus::Void, 1, 1000).is_err());
    }
}
