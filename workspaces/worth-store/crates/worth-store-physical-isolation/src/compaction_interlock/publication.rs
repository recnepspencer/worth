use super::{
    CompactionCutoverDelta, CompactionInterlockFoundationalEvidence,
    CompactionReadInterlockCounters, CompactionReadInterlockDenial,
};
use crate::PhysicalPublicationPlanCompletion;

#[derive(Debug, Clone)]
pub struct CompactionRewritePublication {
    delta: CompactionCutoverDelta,
    publication: PhysicalPublicationPlanCompletion,
    counters: CompactionReadInterlockCounters,
}

impl CompactionRewritePublication {
    const OWNER_CASE: super::CompactionOwnerCaseDeclaration =
        super::CompactionOwnerCaseDeclaration::declared_by_owner(
            super::CompactionOwnerCaseId::PublishRewrite,
            super::CompactionCutoverState::RewriteLowered,
            super::CompactionCutoverState::PublicationCommitted,
        );

    pub const fn cutover_state(&self) -> super::CompactionCutoverState {
        super::CompactionCutoverState::PublicationCommitted
    }

    pub const fn owner_case_observation(&self) -> super::CompactionOwnerCaseObservation {
        super::CompactionOwnerCaseObservation::issued_by_owner(Self::OWNER_CASE)
    }

    pub fn publish_rewrite(
        delta: CompactionCutoverDelta,
        publication: PhysicalPublicationPlanCompletion,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        let delta = delta.bind_publication(&publication)?;
        let counters = delta.plan().counters().with_publication_plan_completion();
        Ok(Self {
            delta,
            publication,
            counters,
        })
    }

    pub const fn delta(&self) -> &super::CompactionCutoverDelta {
        &self.delta
    }

    pub const fn publication(&self) -> &PhysicalPublicationPlanCompletion {
        &self.publication
    }

    pub const fn counters(&self) -> CompactionReadInterlockCounters {
        self.counters
    }

    /// Plan the admitted candidate footprint against this publication's new
    /// root. This is local planning, not a live Store reader or byte access.
    pub fn plan_post_cutover_read(
        &self,
    ) -> Result<crate::StablePhysicalReadPlan, crate::PhysicalReadPlanAdmissionDenial> {
        let authority = crate::admit_post_publication_read_stability_authority(&self.publication)
            .expect("publication-derived local read authority is infallible");
        let plan = self.delta.plan();
        let candidates = plan.candidates();
        let resident_bytes = plan
            .source_integrity()
            .stable_read_receipt()
            .expect("admitted compaction retains its source completion")
            .counters()
            .resident_bytes();
        crate::physical_read_plan::admit_known_footprint_read(
            &authority,
            self.publication.new_root(),
            candidates.references().iter().copied(),
            resident_bytes,
            candidates.references().len(),
        )
    }

    pub const fn foundational_evidence(&self) -> CompactionInterlockFoundationalEvidence {
        CompactionInterlockFoundationalEvidence::after_store_decision(self.counters)
    }
}

pub(super) fn owner_cases() -> impl Iterator<Item = super::CompactionOwnerCaseDeclaration> {
    std::iter::once(CompactionRewritePublication::OWNER_CASE)
}
