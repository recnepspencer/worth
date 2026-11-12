use super::super::active_budget::ActiveSubscriptionAllocationPosture;
use super::super::attachment_budget::DeliveryBackpressurePolicy;
use super::super::maintenance_delta::QuerySubscriptionMaintenanceDeltaKind;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn lifecycle_delivery_window_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    attachment_identity: &WorthQueryEvidenceIdentity,
    sequence: u64,
    delivery_window_width: u64,
    patch_group_width: u64,
    allocation_scope_width: u64,
    allocation_posture: ActiveSubscriptionAllocationPosture,
    backpressure_policy: DeliveryBackpressurePolicy,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_delivery_window_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("attachment"),
                attachment_identity,
            )
            .field_usize(WorthQueryEvidenceTag::new("sequence"), sequence as usize)
            .field_usize(
                WorthQueryEvidenceTag::new("window_width"),
                delivery_window_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("patch_width"),
                patch_group_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("allocation_width"),
                allocation_scope_width as usize,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("allocation_posture"),
                allocation_posture.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("backpressure"),
                backpressure_policy.as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn lifecycle_maintenance_delta_identity(
    kind: QuerySubscriptionMaintenanceDeltaKind,
    lane_identity: &WorthQueryEvidenceIdentity,
    scope_identity: &WorthQueryEvidenceIdentity,
    width: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_maintenance_delta_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("scope"), scope_identity)
            .field_usize(WorthQueryEvidenceTag::new("width"), width as usize)
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn lifecycle_maintenance_delta_scope_identity(
    scope_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_maintenance_delta_scope_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("scope"), scope_label)
        .seal()
}

pub(in crate::subscription) fn lifecycle_maintenance_delta_identity_typed(
    kind: QuerySubscriptionMaintenanceDeltaKind,
    lane_identity: &WorthQueryEvidenceIdentity,
    commit_identity: &WorthQueryEvidenceIdentity,
    collection_identity: &WorthQueryEvidenceIdentity,
    entity_identity: &WorthQueryEvidenceIdentity,
    width: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_maintenance_delta_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("commit"), commit_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                collection_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("entity"), entity_identity)
            .field_usize(WorthQueryEvidenceTag::new("width"), width as usize)
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}
