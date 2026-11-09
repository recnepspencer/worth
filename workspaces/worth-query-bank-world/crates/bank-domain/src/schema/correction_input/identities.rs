//! Preserve original serialized identity forms while using domain constructors.
use crate::{
    estate::EstateCaseId,
    model::{AccountId, BankPrincipalId, PaymentId},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename = "CreatedDomainIdentity")]
pub(super) enum RecordedCreatedId {
    Fixture(u64),
    Operation {
        idempotency_key: [u8; 32],
        ordinal: u32,
    },
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename = "AccountId")]
pub(super) struct RecordedAccountId(pub(super) RecordedCreatedId);
impl RecordedAccountId {
    pub(super) fn restore(self) -> Option<AccountId> {
        match self.0 {
            RecordedCreatedId::Fixture(value) => AccountId::new(value),
            RecordedCreatedId::Operation {
                idempotency_key,
                ordinal,
            } => Some(AccountId::from_operation(idempotency_key, ordinal)),
        }
    }
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
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename = "BankPrincipalId")]
pub(super) struct RecordedPrincipalId(pub(super) u64);
impl RecordedPrincipalId {
    pub(super) fn restore(self) -> Option<BankPrincipalId> {
        BankPrincipalId::new(self.0)
    }
}
#[derive(Deserialize)]
#[serde(rename = "EstateCaseId")]
pub(super) struct RecordedEstateId(u64);
impl RecordedEstateId {
    pub(super) fn restore(self) -> Option<EstateCaseId> {
        EstateCaseId::new(self.0)
    }
}
