use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
};
use worth_relational::facade::identity::EntityId;

use super::WorthQueryElevationCloseDraft;
use crate::domain_computation::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryElevationClosureKind,
};

#[derive(Debug)]
pub(in crate::domain_computation) struct WorthQueryElevationCloseBinding {
    approved: WorthQueryApprovedElevation,
    draft: WorthQueryElevationCloseDraft,
}

impl WorthQueryElevationCloseDraft {
    pub(in crate::domain_computation) fn bind(
        self,
        approved: WorthQueryApprovedElevation,
    ) -> WorthQueryElevationCloseBinding {
        WorthQueryElevationCloseBinding {
            approved,
            draft: self,
        }
    }
}

impl WorthQueryElevationCloseBinding {
    pub(in crate::domain_computation) const fn approved(&self) -> &WorthQueryApprovedElevation {
        &self.approved
    }
    pub(in crate::domain_computation) const fn elevation(&self) -> EntityId {
        self.draft.elevation
    }
    pub(in crate::domain_computation) const fn review(&self) -> EntityId {
        self.draft.review
    }
    pub(in crate::domain_computation) const fn closer(&self) -> EntityId {
        self.draft.closer
    }
    pub(in crate::domain_computation) const fn closure_kind(
        &self,
    ) -> WorthQueryElevationClosureKind {
        self.draft.closure_kind
    }
    pub(in crate::domain_computation) const fn closed_at(&self) -> &AspectValue {
        &self.draft.closed_at
    }
    pub(in crate::domain_computation) const fn closed_at_field(&self) -> &AspectFieldLocator {
        &self.draft.closed_at_field
    }
    pub(in crate::domain_computation) const fn closed_status(&self) -> &AspectValue {
        &self.draft.closed_status
    }
    pub(in crate::domain_computation) fn restore_committed_close(
        mut self,
        status: AspectValue,
        closed_at: AspectValue,
    ) -> Result<Self, Self> {
        self.draft.closure_kind = if status == self.draft.revoked_status {
            WorthQueryElevationClosureKind::Revoked
        } else if status == self.draft.expired_status {
            WorthQueryElevationClosureKind::Expired
        } else {
            return Err(self);
        };
        self.draft.closed_status = status;
        self.draft.closed_at = closed_at;
        Ok(self)
    }
    pub(in crate::domain_computation) const fn approved_status(&self) -> &AspectValue {
        &self.draft.approved_status
    }
    pub(in crate::domain_computation) fn elevation_entity(&self) -> &str {
        &self.draft.elevation_entity
    }
    pub(in crate::domain_computation) const fn status_field(&self) -> &AspectFieldLocator {
        &self.draft.status_field
    }
    pub(in crate::domain_computation) const fn approver_relation(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.draft.approver_relation
    }
    pub(in crate::domain_computation) const fn reviewer_relation(
        &self,
    ) -> worth_relational::facade::identity::KindId {
        self.draft.reviewer_relation
    }
    pub(in crate::domain_computation) fn required_decision_reads(
        &self,
    ) -> &[ApplicationOperationDecisionReadTarget] {
        &self.draft.required_decision_reads
    }
    pub(in crate::domain_computation) fn required_program_targets(
        &self,
    ) -> &[ApplicationOperationProgramTarget] {
        &self.draft.required_program_targets
    }
    pub(in crate::domain_computation) const fn lifecycle_effect(&self) -> Option<&worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>{
        self.draft.lifecycle_effect.as_ref()
    }

    pub(in crate::domain_computation) fn into_approved(self) -> WorthQueryApprovedElevation {
        self.approved
    }
}
