use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
};
use worth_relational::facade::identity::{EntityId, KindId};

use super::WorthQueryMandatoryReviewDraft;
use crate::domain_computation::primary_graph::WorthQueryMandatoryReview;

#[derive(Debug)]
pub(in crate::domain_computation) struct WorthQueryMandatoryReviewBinding {
    mandatory: WorthQueryMandatoryReview,
    draft: WorthQueryMandatoryReviewDraft,
}

impl WorthQueryMandatoryReviewDraft {
    pub(in crate::domain_computation) fn bind(
        self,
        mandatory: WorthQueryMandatoryReview,
    ) -> WorthQueryMandatoryReviewBinding {
        WorthQueryMandatoryReviewBinding {
            mandatory,
            draft: self,
        }
    }
}

impl WorthQueryMandatoryReviewBinding {
    pub(in crate::domain_computation) const fn mandatory(&self) -> &WorthQueryMandatoryReview {
        &self.mandatory
    }
    pub(in crate::domain_computation) const fn elevation(&self) -> EntityId {
        self.draft.elevation
    }
    pub(in crate::domain_computation) const fn review(&self) -> EntityId {
        self.draft.review
    }
    pub(in crate::domain_computation) const fn reviewer(&self) -> EntityId {
        self.draft.reviewer
    }
    pub(in crate::domain_computation) const fn reviewed_at(&self) -> &AspectValue {
        &self.draft.reviewed_at
    }
    pub(in crate::domain_computation) fn restore_committed_review(
        mut self,
        reviewed_at: AspectValue,
    ) -> Self {
        self.draft.reviewed_at = reviewed_at;
        self
    }
    pub(in crate::domain_computation) const fn reviewed_at_field(&self) -> &AspectFieldLocator {
        &self.draft.reviewed_at_field
    }
    pub(in crate::domain_computation) const fn terminal_status(&self) -> &AspectValue {
        &self.draft.terminal_status
    }
    pub(in crate::domain_computation) const fn completed_status(&self) -> &AspectValue {
        &self.draft.completed_status
    }
    pub(in crate::domain_computation) fn review_entity(&self) -> &str {
        &self.draft.review_entity
    }
    pub(in crate::domain_computation) const fn review_status_field(&self) -> &AspectFieldLocator {
        &self.draft.review_status_field
    }
    pub(in crate::domain_computation) const fn approver_relation(&self) -> KindId {
        self.draft.approver_relation
    }
    pub(in crate::domain_computation) const fn reviewer_relation(&self) -> KindId {
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

    pub(in crate::domain_computation) fn into_mandatory(self) -> WorthQueryMandatoryReview {
        self.mandatory
    }
}
