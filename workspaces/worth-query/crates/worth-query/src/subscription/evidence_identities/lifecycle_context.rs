use super::super::active_handle::ActiveSubscriptionLaneHandle;
use super::super::equivalence::QuerySubscriptionEquivalenceBasis;
use super::super::family::QuerySubscriptionFamily;
use super::super::input::LiveQueryAdmissionArtifact;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn lifecycle_context_query_identity(
    live: &LiveQueryAdmissionArtifact,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_query_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("live_family"),
                live.live_family().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("future_selection"),
                live.future_selection().projection_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis"),
                live.basis_posture().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("view_family"),
                live.view_family()
                    .map(|family| family.as_str())
                    .unwrap_or("none"),
            )
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_context_policy_identity(
    policy: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_policy_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("policy"), policy)
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_context_tenant_basis_identity(
    tenant_basis: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_tenant_basis_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("tenant_basis"), tenant_basis)
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_context_relationship_proof_identity(
    relationship_proof: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_relationship_proof_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("relationship_proof"),
                relationship_proof,
            )
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_context_collection_absent_identity(
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_collection_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("collection"), "none")
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_subscription_family_identity(
    family: &QuerySubscriptionFamily,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_family_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn lifecycle_subscription_equivalence_identity(
    basis: &QuerySubscriptionEquivalenceBasis,
) -> WorthQueryEvidenceIdentity {
    basis.evidence_identity().clone()
}

pub(in crate::subscription) fn lifecycle_active_lane_handle_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    handle: &ActiveSubscriptionLaneHandle,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_active_lane_handle_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
            .field_usize(
                WorthQueryEvidenceTag::new("index"),
                handle.lane_index() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("generation"),
                handle.registry_generation() as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

#[cfg(test)]
pub(in crate::subscription) fn lifecycle_absent_work_packet_identity() -> WorthQueryEvidenceIdentity
{
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "active_delivery_work_packet_absent_v1",
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_absent_performance_receipt_identity(
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_performance_receipt_absent_v1",
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_absent_preview_isolation_identity(
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_preview_isolation_absent_v1",
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_absent_preview_residue_identity(
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_preview_residue_absent_v1",
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_absent_continuation_identity() -> WorthQueryEvidenceIdentity
{
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_continuation_absent_v1",
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_performance_sequence_identity<'a>(
    receipts: impl IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_performance_receipt_v1",
            )
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("elements"), receipts)
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_labeled_counter_identity(
    role: &str,
    counter_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_counter_element_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity(WorthQueryEvidenceTag::new("counter"), counter_identity)
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_counter_sequence_identity<'a>(
    counters: impl IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_counter_snapshot_v1",
            )
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("elements"), counters)
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_support_matrix_identity<'a>(
    support_identities: impl IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_support_matrix_v1",
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("elements"),
                support_identities,
            )
            .seal()?,
    )
}
