//! Decode retained payment input identities through Bank's checked constructors.
use crate::model::{BankPrincipalId, PaymentId};
use serde::Deserialize;

pub(super) fn payment_identity<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<crate::model::PaymentId, D::Error> {
    RecordedPaymentId::deserialize(decoder)?
        .restore()
        .ok_or_else(|| serde::de::Error::custom("invalid payment identity"))
}

pub(super) fn principal_identity<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<crate::model::BankPrincipalId, D::Error> {
    RecordedPrincipalId::deserialize(decoder)?
        .restore()
        .ok_or_else(|| serde::de::Error::custom("invalid principal identity"))
}

#[derive(Deserialize)]
#[serde(rename = "CreatedDomainIdentity")]
pub(super) enum RecordedCreatedId {
    Fixture(u64),
    Operation {
        idempotency_key: [u8; 32],
        ordinal: u32,
    },
}
#[derive(Deserialize)]
#[serde(rename = "PaymentId")]
pub(super) struct RecordedPaymentId(RecordedCreatedId);
impl RecordedPaymentId {
    pub(super) fn restore(self) -> Option<PaymentId> {
        match self.0 {
            RecordedCreatedId::Fixture(value) => PaymentId::new(value),
            RecordedCreatedId::Operation {
                idempotency_key,
                ordinal,
            } => Some(PaymentId::from_operation(idempotency_key, ordinal)),
        }
    }
}
#[derive(Deserialize)]
#[serde(rename = "BankPrincipalId")]
pub(super) struct RecordedPrincipalId(pub(super) u64);
impl RecordedPrincipalId {
    pub(super) fn restore(self) -> Option<BankPrincipalId> {
        BankPrincipalId::new(self.0)
    }
}
