use super::{
    CompactionReadInterlockDenial, CompactionRecoveryEvidence, CompactionRewritePublication,
};
use crate::CurrentPhysicalRoot;

#[derive(Debug, Clone)]
pub struct CompactionCutoverStabilityProof {
    publication: CompactionRewritePublication,
    recovery_evidence: CompactionRecoveryEvidence,
}

impl CompactionCutoverStabilityProof {
    const OWNER_CASE: super::CompactionOwnerCaseDeclaration =
        super::CompactionOwnerCaseDeclaration::declared_by_owner(
            super::CompactionOwnerCaseId::AdmitRecoveryVisibility,
            super::CompactionCutoverState::PublicationCommitted,
            super::CompactionCutoverState::RecoveryVisibilityAdmitted,
        );

    pub const fn cutover_state(&self) -> super::CompactionCutoverState {
        super::CompactionCutoverState::RecoveryVisibilityAdmitted
    }

    pub const fn owner_case_observation(&self) -> super::CompactionOwnerCaseObservation {
        super::CompactionOwnerCaseObservation::issued_by_owner(Self::OWNER_CASE)
    }

    pub fn admit(
        publication: CompactionRewritePublication,
        recovery_evidence: CompactionRecoveryEvidence,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        match recovery_evidence {
            CompactionRecoveryEvidence::SelectedProduct(_) => Ok(Self {
                publication,
                recovery_evidence,
            }),
            CompactionRecoveryEvidence::Residue(kind) => {
                Err(CompactionReadInterlockDenial::BackendResidueCandidateSelection(kind))
            }
        }
    }

    pub const fn pre_cutover_root(&self) -> CurrentPhysicalRoot {
        self.publication.publication().old_root()
    }

    pub const fn post_cutover_root(&self) -> CurrentPhysicalRoot {
        self.publication.publication().new_root()
    }

    pub const fn publication(&self) -> &CompactionRewritePublication {
        &self.publication
    }

    pub const fn recovery_evidence(&self) -> CompactionRecoveryEvidence {
        self.recovery_evidence
    }

    pub fn plan_post_cutover_read(
        &self,
    ) -> Result<crate::StablePhysicalReadPlan, crate::PhysicalReadPlanAdmissionDenial> {
        let authority = crate::admit_post_compaction_read_stability_authority(self)
            .expect("sealed cutover stability proof issues read stability authority");
        let candidates = self.publication.delta().plan().candidates();
        let resident_bytes = self
            .publication
            .delta()
            .plan()
            .source_integrity()
            .stable_read_receipt()
            .map(|receipt| receipt.counters().guarded_bytes())
            .unwrap_or(0);
        crate::physical_read_plan::admit_known_footprint_read(
            &authority,
            self.post_cutover_root(),
            candidates.references().iter().copied(),
            resident_bytes,
            candidates.references().len(),
        )
    }
}

pub(super) fn owner_cases() -> impl Iterator<Item = super::CompactionOwnerCaseDeclaration> {
    std::iter::once(CompactionCutoverStabilityProof::OWNER_CASE)
}
