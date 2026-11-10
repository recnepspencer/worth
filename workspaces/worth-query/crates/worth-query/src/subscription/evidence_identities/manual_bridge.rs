use super::super::bridge_slice::BridgeSubscriptionSliceKind;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

#[allow(clippy::too_many_arguments)]
pub(in crate::subscription) fn manual_bridge_witness_identity(
    query_family: &str,
    bridge_family: &str,
    basis_posture: &str,
    signal_strategy_class: &str,
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_declaration_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    activation_identity: &WorthQueryEvidenceIdentity,
    assembly_posture: &str,
    bridge_slices: &[BridgeSubscriptionSliceKind],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_manual_bridge_witness_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("query_family"), query_family)
        .field_shape(WorthQueryEvidenceTag::new("bridge_family"), bridge_family)
        .field_shape(WorthQueryEvidenceTag::new("basis_posture"), basis_posture)
        .field_shape(
            WorthQueryEvidenceTag::new("signal_strategy_class"),
            signal_strategy_class,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_declaration"),
            query_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_declaration"),
            bridge_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_binding"),
            basis_binding_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("signal_strategy"),
            signal_strategy_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("activation"),
            activation_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("assembly_posture"),
            assembly_posture,
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("bridge_slices"),
            bridge_slices
                .iter()
                .map(BridgeSubscriptionSliceKind::as_str),
        )
        .seal()
}
