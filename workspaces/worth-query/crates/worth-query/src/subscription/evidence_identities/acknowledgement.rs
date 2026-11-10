use super::super::attachment_budget::DeliveryBackpressurePolicy;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
#[cfg(test)]
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn lifecycle_acknowledgement_frontier_identity(
    attachment_identity: &WorthQueryEvidenceIdentity,
    sequence: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_acknowledgement_frontier_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("attachment"),
                attachment_identity,
            )
            .field_usize(WorthQueryEvidenceTag::new("sequence"), sequence as usize)
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn subscription_performance_receipt_source_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    consumer_identity: &WorthQueryEvidenceIdentity,
    backpressure_policy: &DeliveryBackpressurePolicy,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_performance_receipt_source_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("consumer"), consumer_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("backpressure"),
                backpressure_policy.as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

#[cfg(test)]
pub(in crate::subscription) fn lifecycle_continuation_endpoint_identity(
    role: &str,
    source_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_continuation_endpoint_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .seal()
}

#[cfg(test)]
pub(in crate::subscription) fn lifecycle_continuation_ordinary_checkpoint_identity(
    active_lane_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_continuation_ordinary_checkpoint_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("active_lane"),
                active_lane_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
