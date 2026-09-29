use std::num::NonZeroU64;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_decl::facade::{
    application_schema::{
        ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
        ApplicationInboundOccurrenceBinding, ApplicationInboundOccurrenceLimits,
        ApplicationInboundOccurrenceProtocol, ApplicationRetainedEffectBinding,
    },
    worth_query_effect, worth_query_structured_value_binding,
};

use crate::{
    model::{AccountId, BankPrincipalId, PaymentId},
    schema::BankSchema,
};

pub const APPROVED_PAYMENT_SETTLEMENT_RAIL: &str = "approved-payment-settlement-rail";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedPaymentSettlementRequest {
    payment: PaymentId,
    source: AccountId,
    destination: AccountId,
    approver: BankPrincipalId,
    amount_minor: i64,
}

impl ApprovedPaymentSettlementRequest {
    pub const fn new(
        payment: PaymentId,
        source: AccountId,
        destination: AccountId,
        approver: BankPrincipalId,
        amount_minor: i64,
    ) -> Self {
        Self {
            payment,
            source,
            destination,
            approver,
            amount_minor,
        }
    }

    pub const fn payment(self) -> PaymentId {
        self.payment
    }
}

worth_query_structured_value_binding!(pub ApprovedPaymentSettlementRequestBinding for ApprovedPaymentSettlementRequest { identity: "bank.payment.effect.approved-settlement.payload.v1" });

impl ApplicationRetainedEffectBinding for ApprovedPaymentSettlementRequestBinding {
    fn retained_bytes(_: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<ApprovedPaymentSettlementRequest>()).unwrap_or(u64::MAX)
    }
}

impl ApplicationExternalEffectBinding for ApprovedPaymentSettlementRequestBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("bank.payment.approved-settlement"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 268;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MAX_EXTERNAL_BYTES as usize);
        push_identity(&mut bytes, &value.payment.canonical_text());
        push_identity(&mut bytes, &value.source.canonical_text());
        push_identity(&mut bytes, &value.destination.canonical_text());
        bytes.extend_from_slice(&value.approver.get().to_be_bytes());
        bytes.extend_from_slice(&value.amount_minor.to_be_bytes());
        bytes
    }
}

fn push_identity(bytes: &mut Vec<u8>, identity: &str) {
    bytes.push(u8::try_from(identity.len()).expect("a canonical Bank identity fits one byte"));
    bytes.extend_from_slice(identity.as_bytes());
}

worth_query_effect!(pub ApprovedPaymentSettlementEffect for BankSchema, payload ApprovedPaymentSettlementRequestBinding);

/// One fixed Bank declaration shared by the payment operation and its workflow wait.
pub fn approved_payment_inbound_binding(
) -> ApplicationInboundOccurrenceBinding<ApprovedPaymentSettlementEffect> {
    ApplicationInboundOccurrenceBinding::new(
        ApplicationInboundOccurrenceProtocol::new(
            BoundaryProtocolIdentity::new("bank.payment.approved-settlement"),
            BoundaryProtocolVersion::new(1),
        ),
        "rail-primary",
        ApplicationInboundOccurrenceLimits {
            maximum_envelope_bytes: NonZeroU64::new(4_096).unwrap(),
            maximum_payload_bytes: NonZeroU64::new(268).unwrap(),
            maximum_outstanding_dispatch_provenance: NonZeroU64::new(1_024).unwrap(),
            maximum_accepted_occurrences: NonZeroU64::new(1_024).unwrap(),
            maximum_accepted_bytes: NonZeroU64::new(5 * 1_024 * 1_024).unwrap(),
            maximum_concurrent_publications: NonZeroU64::new(32).unwrap(),
            maximum_discovery_work: NonZeroU64::new(1_024).unwrap(),
            replay_window_milliseconds: NonZeroU64::new(60_000).unwrap(),
            maximum_cleanup_work: NonZeroU64::new(1_024).unwrap(),
        },
    )
    .expect("payment rail inbound bounds are usable")
}

#[cfg(test)]
mod tests {
    use worth_query_decl::facade::application_schema::{
        ApplicationExternalEffectBinding, ApplicationRetainedEffectBinding,
    };

    use super::{ApprovedPaymentSettlementRequest, ApprovedPaymentSettlementRequestBinding};
    use crate::model::{AccountId, BankPrincipalId, PaymentId};

    #[test]
    fn approved_payment_settlement_has_a_fixed_external_encoding() {
        let request = ApprovedPaymentSettlementRequest::new(
            PaymentId::new(1).unwrap(),
            AccountId::new(2).unwrap(),
            AccountId::new(3).unwrap(),
            BankPrincipalId::new(4).unwrap(),
            5,
        );
        let encoded = ApprovedPaymentSettlementRequestBinding::external_effect_bytes(&request);
        assert!(encoded.len() as u64 <= 268);
        assert_eq!(
            ApprovedPaymentSettlementRequestBinding::retained_bytes(&request),
            u64::try_from(std::mem::size_of::<ApprovedPaymentSettlementRequest>()).unwrap()
        );
        assert_eq!(
            encoded,
            [
                b"\tfixture:1".as_slice(),
                b"\tfixture:2".as_slice(),
                b"\tfixture:3".as_slice(),
                4_u64.to_be_bytes().as_slice(),
                5_i64.to_be_bytes().as_slice(),
            ]
            .concat()
        );
    }
}
