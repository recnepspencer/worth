use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::support::{
    QuerySubscriptionBridgeParityClass, QuerySubscriptionBridgeParityCounters,
    QuerySubscriptionBridgeParityFailureKind,
};
#[cfg(test)]
use super::witness::BridgeWitnessAssemblyPosture;

pub(super) fn bridge_parity_counter_identity(
    counters: &QuerySubscriptionBridgeParityCounters,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_bridge_parity_counters_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("comparison_count"),
                counters.subscription_bridge_parity_comparison_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admitted_count"),
                counters.subscription_bridge_parity_admitted_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("denial_count"),
                counters.subscription_bridge_parity_denial_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_distinction_preservation_count"),
                counters.subscription_bridge_family_distinction_preservation_count() as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

#[cfg(test)]
pub(super) fn bridge_parity_width_identity(
    compared_family_dimension_count: usize,
    compared_slice_dimension_count: usize,
    compared_basis_dimension_count: usize,
    compared_signal_dimension_count: usize,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_bridge_parity_width_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_dimensions"),
                compared_family_dimension_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("slice_dimensions"),
                compared_slice_dimension_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("basis_dimensions"),
                compared_basis_dimension_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("signal_dimensions"),
                compared_signal_dimension_count,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

#[cfg(test)]
pub(super) fn bridge_parity_receipt_identity(
    witness_assembly_posture: BridgeWitnessAssemblyPosture,
    parity_class: QuerySubscriptionBridgeParityClass,
    comparison_width_identity: &WorthQueryEvidenceIdentity,
    semantic_rebuild_count: usize,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_bridge_parity_receipt_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("witness_assembly_posture"),
                witness_assembly_posture.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("parity_class"),
                parity_class.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("comparison_width"),
                comparison_width_identity,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("semantic_rebuild_count"),
                semantic_rebuild_count,
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

#[cfg(test)]
pub(super) fn bridge_parity_comparison_identity(
    parity_class: QuerySubscriptionBridgeParityClass,
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_declaration_identity: &WorthQueryEvidenceIdentity,
    witness_identity: &WorthQueryEvidenceIdentity,
    activation_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_bridge_parity_comparison_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("parity_class"),
                parity_class.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query_declaration"),
                query_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_declaration"),
                bridge_declaration_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("witness"), witness_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("activation"),
                activation_identity,
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

#[cfg(test)]
pub(super) fn bridge_parity_explanation_identity(
    comparison_identity: &WorthQueryEvidenceIdentity,
    witness_identity: &WorthQueryEvidenceIdentity,
    receipt_identity: &WorthQueryEvidenceIdentity,
    counter_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_bridge_parity_explanation_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("comparison"),
                comparison_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("witness"), witness_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), receipt_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counter_identity)
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(super) fn bridge_parity_failure_identity(
    failure_kind: QuerySubscriptionBridgeParityFailureKind,
    parity_class: QuerySubscriptionBridgeParityClass,
    reason: &str,
    source_identity: &WorthQueryEvidenceIdentity,
    evidence: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_bridge_parity_failure_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("failure_kind"),
            failure_kind.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("parity_class"),
            parity_class.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("reason"), reason)
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("evidence"), evidence.iter())
        .seal()
}
