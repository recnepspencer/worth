use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::WorthQueryRuntimeMixedCauseLaneKind;

pub(in crate::runtime) fn runtime_mixed_cause_atomic_identity(
    delivery_cause_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_mixed_cause_atomic_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("delivery_cause"),
                delivery_cause_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

#[cfg(test)]
pub(in crate::runtime) fn runtime_mixed_cause_ordering_identity(
    ordering: &BridgeMixedCauseOrdering,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_mixed_cause_ordering_v1",
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("ordering"),
                &ordering.ordering_identity().bridge_admission_evidence(),
            )
            .seal()?,
    )
}

#[cfg(test)]
pub(in crate::runtime) fn runtime_mixed_cause_delivery_window_identity(
    delivery_window: &BridgeMixedCauseDeliveryWindowPlan,
    ordering: &BridgeMixedCauseOrdering,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_mixed_cause_delivery_window_v1",
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("ordering"),
                &ordering.ordering_identity().bridge_admission_evidence(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("delivery_window"),
                &delivery_window
                    .delivery_window_identity()
                    .bridge_admission_evidence(),
            )
            .seal()?,
    )
}

#[cfg(test)]
pub(in crate::runtime) fn runtime_mixed_cause_ordered_cause_identity(
    cause: &BridgeOrderedMixedCause,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_mixed_cause_ordered_cause_v1",
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("ordered_cause"),
                &cause.ordered_cause_identity().bridge_admission_evidence(),
            )
            .seal()?,
    )
}

#[cfg(test)]
pub(in crate::runtime) fn runtime_mixed_cause_suppressed_cause_identity(
    cause: &BridgeSuppressedMixedCause,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_mixed_cause_suppressed_cause_v1",
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("suppressed_cause"),
                &cause
                    .suppressed_cause_identity()
                    .bridge_admission_evidence(),
            )
            .seal()?,
    )
}

#[cfg(test)]
pub(in crate::runtime) fn runtime_mixed_cause_denied_cause_identity(
    cause: &BridgeDeniedMixedCause,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_mixed_cause_denied_cause_v1",
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("denied_cause"),
                &cause.denied_cause_identity().bridge_admission_evidence(),
            )
            .seal()?,
    )
}

pub(in crate::runtime) fn runtime_mixed_cause_delivery_identity(
    delivery: &WorthQueryRuntimeMixedCauseDelivery,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_mixed_cause_delivery_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("ordering"),
                delivery.ordering_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("window"),
                delivery.delivery_window_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lane"),
                mixed_cause_lane_label(delivery.lane_kind()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("coalescing"),
                delivery.coalescing_kind().as_public_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("ordered_kind"),
                delivery
                    .ordered_member_kinds()
                    .iter()
                    .map(|member_kind| member_kind.as_public_str()),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("ordered_cause"),
                delivery.ordered_cause_identities().iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("suppressed"),
                delivery.suppressed_cause_identities().iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("denied"),
                delivery.denied_cause_identities().iter(),
            )
            .seal()?,
    )
}

fn mixed_cause_lane_label(lane: WorthQueryRuntimeMixedCauseLaneKind) -> &'static str {
    match lane {
        WorthQueryRuntimeMixedCauseLaneKind::Authoritative => "authoritative",
        WorthQueryRuntimeMixedCauseLaneKind::Preview => "preview",
    }
}
