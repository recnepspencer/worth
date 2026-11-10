use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::evidence_identities::patch_group_identity;
use super::maintenance_delta::QuerySubscriptionMaintenanceDeltaKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryPatchGroupKind {
    DetailFieldPatchGroup,
    InspectorFocusedPatchGroup,
    CollectionMembershipPatchGroup,
    CollectionOrderPatchGroup,
    GroupedMembershipPatchGroup,
    BoundedMaterializationScopePatchGroup,
    ContinuationPatchGroup,
    DeliveryGapPatchGroup,
    TimeOnlyDeliveryGroup,
    MixedCauseDeliveryGroup,
}

impl QueryPatchGroupKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DetailFieldPatchGroup => "detail_field_patch_group",
            Self::InspectorFocusedPatchGroup => "inspector_focused_patch_group",
            Self::CollectionMembershipPatchGroup => "collection_membership_patch_group",
            Self::CollectionOrderPatchGroup => "collection_order_patch_group",
            Self::GroupedMembershipPatchGroup => "grouped_membership_patch_group",
            Self::BoundedMaterializationScopePatchGroup => {
                "bounded_materialization_scope_patch_group"
            }
            Self::ContinuationPatchGroup => "continuation_patch_group",
            Self::DeliveryGapPatchGroup => "delivery_gap_patch_group",
            Self::TimeOnlyDeliveryGroup => "time_only_delivery_group",
            Self::MixedCauseDeliveryGroup => "mixed_cause_delivery_group",
        }
    }

    pub(super) fn from_delta_kind(kind: QuerySubscriptionMaintenanceDeltaKind) -> Self {
        match kind {
            QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta => Self::DetailFieldPatchGroup,
            QuerySubscriptionMaintenanceDeltaKind::InspectorFocusDelta => {
                Self::InspectorFocusedPatchGroup
            }
            QuerySubscriptionMaintenanceDeltaKind::CollectionMembershipDelta => {
                Self::CollectionMembershipPatchGroup
            }
            QuerySubscriptionMaintenanceDeltaKind::CollectionOrderDelta => {
                Self::CollectionOrderPatchGroup
            }
            QuerySubscriptionMaintenanceDeltaKind::GroupedMembershipDelta => {
                Self::GroupedMembershipPatchGroup
            }
            QuerySubscriptionMaintenanceDeltaKind::BoundedMaterializationScopeDelta => {
                Self::BoundedMaterializationScopePatchGroup
            }
            QuerySubscriptionMaintenanceDeltaKind::ContinuationDelta => {
                Self::ContinuationPatchGroup
            }
            QuerySubscriptionMaintenanceDeltaKind::GapNoticeDelta => Self::DeliveryGapPatchGroup,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPatchGroup {
    kind: QueryPatchGroupKind,
    width: u64,
    patch_group_identity: WorthQueryEvidenceIdentity,
}

impl QueryPatchGroup {
    pub(crate) fn new(
        kind: QueryPatchGroupKind,
        source_identity: &WorthQueryEvidenceIdentity,
        width: u64,
    ) -> Self {
        let patch_group_identity = patch_group_identity(kind, source_identity, width);
        Self {
            kind,
            width,
            patch_group_identity,
        }
    }

    pub fn kind(&self) -> QueryPatchGroupKind {
        self.kind
    }

    pub fn width(&self) -> u64 {
        self.width
    }

    pub fn patch_group_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.patch_group_identity
    }
}
