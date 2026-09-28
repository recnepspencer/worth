use worth_foundational::facade::AspectKey;

use crate::input::envelope::BridgeCommittedPatchTarget;
use crate::mapping::{
    BridgeAspectRegistrationId, BridgeMappingId, BridgeMappingWideningClass, CoarseRoutingMode,
    SliceWideningPolicy, SubscriptionSliceKind, TruthDeltaSurfaceKind,
};
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;
use crate::routing::surfaces::TruthDeltaSurfaceIdentity;
use crate::routing::{FineGrainedMatchOutcome, FineGrainedMatchStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRouteRecordEntry {
    entity_identity: BridgeRouteRecordEntityIdentity,
    aspect_key: AspectKey,
    target: BridgeCommittedPatchTarget,
    source_target: BridgeCommittedPatchTarget,
    truth_surface_identity: TruthDeltaSurfaceIdentity,
    mapping_id: BridgeMappingId,
    signal_scope: String,
    routing_mode: CoarseRoutingMode,
    widening_class: Option<BridgeMappingWideningClass>,
    match_detail: FineGrainedMatchOutcome,
}

pub type BridgeRouteRecordMatch = FineGrainedMatchOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeRouteRecordEntityIdentity {
    RelationalRecord(RelationalBridgeRecordIdentityParts),
    TruthSurface(TruthDeltaSurfaceIdentity),
}

impl BridgeRouteRecordEntityIdentity {
    pub fn diagnostic_label(&self) -> String {
        match self {
            Self::RelationalRecord(record) => format!(
                "relational-record:{}:{}:{}:{}",
                match record.kind() {
                    crate::relational_source::identity_parts::RelationalBridgeRecordIdentityKind::Entity =>
                        "entity",
                    crate::relational_source::identity_parts::RelationalBridgeRecordIdentityKind::Relation =>
                        "relation",
                },
                record.partition_id(),
                record.local_slot(),
                record.generation()
            ),
            Self::TruthSurface(surface) => surface.as_str().to_string(),
        }
    }
}

impl BridgeRouteRecordEntry {
    pub(crate) fn new(
        entity_identity: BridgeRouteRecordEntityIdentity,
        aspect_key: AspectKey,
        target: BridgeCommittedPatchTarget,
        source_target: BridgeCommittedPatchTarget,
        truth_surface_identity: TruthDeltaSurfaceIdentity,
        mapping_id: BridgeMappingId,
        signal_scope: impl Into<String>,
        routing_mode: CoarseRoutingMode,
        widening_class: Option<BridgeMappingWideningClass>,
        match_detail: BridgeRouteRecordMatch,
    ) -> Self {
        Self {
            entity_identity,
            aspect_key,
            target,
            source_target,
            truth_surface_identity,
            mapping_id,
            signal_scope: signal_scope.into(),
            routing_mode,
            widening_class,
            match_detail,
        }
    }

    pub fn entity_identity(&self) -> &BridgeRouteRecordEntityIdentity {
        &self.entity_identity
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect_key
    }

    pub fn target_canonical_basis(&self) -> String {
        self.target.canonical_basis()
    }

    pub fn source_target_canonical_basis(&self) -> String {
        self.source_target.canonical_basis()
    }

    pub fn target(&self) -> &BridgeCommittedPatchTarget {
        &self.target
    }

    pub fn source_target(&self) -> &BridgeCommittedPatchTarget {
        &self.source_target
    }

    pub fn truth_surface_identity(&self) -> &str {
        self.truth_surface_identity.as_str()
    }

    pub(crate) fn truth_delta_surface_identity(&self) -> &TruthDeltaSurfaceIdentity {
        &self.truth_surface_identity
    }

    pub fn mapping_id(&self) -> &BridgeMappingId {
        &self.mapping_id
    }

    pub fn signal_scope(&self) -> &str {
        &self.signal_scope
    }

    pub fn routing_mode(&self) -> CoarseRoutingMode {
        self.routing_mode
    }

    pub fn widening_class(&self) -> Option<&BridgeMappingWideningClass> {
        self.widening_class.as_ref()
    }

    pub fn truth_surface_kind(&self) -> TruthDeltaSurfaceKind {
        match &self.match_detail {
            FineGrainedMatchOutcome::Matched {
                truth_surface_kind, ..
            }
            | FineGrainedMatchOutcome::WideningAdmitted {
                truth_surface_kind, ..
            }
            | FineGrainedMatchOutcome::SuppressedByRegistrationPolicy { truth_surface_kind } => {
                *truth_surface_kind
            }
        }
    }

    pub fn fine_grained_match_status(&self) -> FineGrainedMatchStatus {
        self.match_detail.status()
    }

    pub fn aspect_registration_id(&self) -> Option<&BridgeAspectRegistrationId> {
        match &self.match_detail {
            FineGrainedMatchOutcome::Matched {
                aspect_registration_id,
                ..
            }
            | FineGrainedMatchOutcome::WideningAdmitted {
                aspect_registration_id,
                ..
            } => Some(aspect_registration_id),
            FineGrainedMatchOutcome::SuppressedByRegistrationPolicy { .. } => None,
        }
    }

    pub fn subscription_slice_kind(&self) -> Option<&SubscriptionSliceKind> {
        self.match_detail.subscription_slice_kind()
    }

    pub fn slice_widening_policy(&self) -> Option<SliceWideningPolicy> {
        match &self.match_detail {
            FineGrainedMatchOutcome::WideningAdmitted {
                widening_policy, ..
            } => Some(*widening_policy),
            FineGrainedMatchOutcome::Matched { .. } => Some(SliceWideningPolicy::Disallow),
            FineGrainedMatchOutcome::SuppressedByRegistrationPolicy { .. } => None,
        }
    }

    pub fn match_detail(&self) -> &BridgeRouteRecordMatch {
        &self.match_detail
    }
}
