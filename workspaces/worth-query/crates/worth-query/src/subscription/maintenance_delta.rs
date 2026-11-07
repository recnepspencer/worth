use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::active_digest::ActiveSubscriptionLaneDigest;
use super::active_posture::ActiveSubscriptionDeliveryPosture;
use super::delivery_dimensions::MaintenanceDeltaWidth;
use super::evidence_identities::{
    lifecycle_maintenance_delta_identity, lifecycle_maintenance_delta_identity_typed,
    lifecycle_maintenance_delta_scope_identity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionMaintenanceDeltaKind {
    DetailFieldDelta,
    InspectorFocusDelta,
    CollectionMembershipDelta,
    CollectionOrderDelta,
    GroupedMembershipDelta,
    BoundedMaterializationScopeDelta,
    ContinuationDelta,
    GapNoticeDelta,
}

impl QuerySubscriptionMaintenanceDeltaKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DetailFieldDelta => "detail_field_delta",
            Self::InspectorFocusDelta => "inspector_focus_delta",
            Self::CollectionMembershipDelta => "collection_membership_delta",
            Self::CollectionOrderDelta => "collection_order_delta",
            Self::GroupedMembershipDelta => "grouped_membership_delta",
            Self::BoundedMaterializationScopeDelta => "bounded_materialization_scope_delta",
            Self::ContinuationDelta => "continuation_delta",
            Self::GapNoticeDelta => "gap_notice_delta",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionMaintenanceDelta {
    kind: QuerySubscriptionMaintenanceDeltaKind,
    active_lane_digest: ActiveSubscriptionLaneDigest,
    scope_identity: WorthQueryEvidenceIdentity,
    width: MaintenanceDeltaWidth,
    maintenance_delta_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionMaintenanceDelta {
    pub fn admitted(
        kind: QuerySubscriptionMaintenanceDeltaKind,
        active_lane_digest: ActiveSubscriptionLaneDigest,
        scope_identity: &WorthQueryEvidenceIdentity,
        width: MaintenanceDeltaWidth,
    ) -> Self {
        let maintenance_delta_identity = lifecycle_maintenance_delta_identity(
            kind,
            active_lane_digest.evidence_identity(),
            scope_identity,
            width.get(),
        );
        Self {
            kind,
            active_lane_digest,
            scope_identity: scope_identity.clone(),
            width,
            maintenance_delta_identity,
        }
    }

    pub fn admitted_with_scope_label(
        kind: QuerySubscriptionMaintenanceDeltaKind,
        active_lane_digest: ActiveSubscriptionLaneDigest,
        scope_label: &str,
        width: MaintenanceDeltaWidth,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::admitted(
            kind,
            active_lane_digest,
            &lifecycle_maintenance_delta_scope_identity(scope_label)?,
            width,
        ))
    }

    pub fn admitted_with_typed_scope(
        kind: QuerySubscriptionMaintenanceDeltaKind,
        active_lane_digest: ActiveSubscriptionLaneDigest,
        commit_identity: &WorthQueryEvidenceIdentity,
        collection_identity: &WorthQueryEvidenceIdentity,
        entity_identity: &WorthQueryEvidenceIdentity,
        width: MaintenanceDeltaWidth,
    ) -> Self {
        let maintenance_delta_identity = lifecycle_maintenance_delta_identity_typed(
            kind,
            active_lane_digest.evidence_identity(),
            commit_identity,
            collection_identity,
            entity_identity,
            width.get(),
        );
        Self {
            kind,
            active_lane_digest,
            scope_identity: collection_identity.clone(),
            width,
            maintenance_delta_identity,
        }
    }

    pub fn kind(&self) -> QuerySubscriptionMaintenanceDeltaKind {
        self.kind
    }

    pub(crate) fn active_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.active_lane_digest
    }

    pub(super) fn scope_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.scope_identity
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.maintenance_delta_identity
    }

    pub fn width(&self) -> u64 {
        self.width.get()
    }

    pub(super) fn delivery_posture(&self) -> ActiveSubscriptionDeliveryPosture {
        match self.kind {
            QuerySubscriptionMaintenanceDeltaKind::InspectorFocusDelta => {
                ActiveSubscriptionDeliveryPosture::FocusedInspectorPatch
            }
            QuerySubscriptionMaintenanceDeltaKind::GroupedMembershipDelta => {
                ActiveSubscriptionDeliveryPosture::GroupedPatch
            }
            QuerySubscriptionMaintenanceDeltaKind::BoundedMaterializationScopeDelta => {
                ActiveSubscriptionDeliveryPosture::BoundedMaterializationPatch
            }
            _ => ActiveSubscriptionDeliveryPosture::QueryShapedPatch,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryMaintenanceDeltaLoweringReport {
    maintenance_delta_identity: WorthQueryEvidenceIdentity,
    lowering_report_identity: WorthQueryEvidenceIdentity,
}

impl QueryMaintenanceDeltaLoweringReport {
    pub(super) fn new(
        delta: &QuerySubscriptionMaintenanceDelta,
    ) -> Result<Self, IdentityDerivationDenial> {
        let lowering_report_identity = WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "query_maintenance_delta_lowering_report_v1",
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("maintenance_delta"),
            delta.evidence_identity(),
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("kind"),
            delta.kind().as_str(),
        )
        .field_usize(
            crate::evidence_identity::WorthQueryEvidenceTag::new("width"),
            delta.width() as usize,
        )
        .seal()?;
        Ok(Self {
            maintenance_delta_identity: delta.evidence_identity().clone(),
            lowering_report_identity,
        })
    }

    pub fn maintenance_delta_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.maintenance_delta_identity
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowering_report_identity
    }
}
