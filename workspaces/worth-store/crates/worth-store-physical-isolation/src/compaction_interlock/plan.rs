use super::{
    CompactionCandidateRangeSet, CompactionProtectedReferenceSet, CompactionReadInterlockCounters,
    CompactionReadInterlockDenial, CompactionSourceIntegrityAdmission,
};
use crate::{RootEpoch, StablePhysicalReadReceipt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompactionSourceEvidencePosture {
    IntegrityClearedStableRead {
        receipt: StablePhysicalReadReceipt,
        admission: CompactionSourceIntegrityAdmission,
    },
    Quarantined(CompactionSourceIntegrityAdmission),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionSourceIntegrityEvidence {
    posture: CompactionSourceEvidencePosture,
}

impl CompactionSourceIntegrityEvidence {
    pub fn from_stable_read_receipt_and_integrity_admission(
        receipt: StablePhysicalReadReceipt,
        admission: CompactionSourceIntegrityAdmission,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        let release = receipt.read_plan_release();
        if release.protected_references_released() == 0 {
            return Err(CompactionReadInterlockDenial::EmptyCandidateRangeSet);
        }
        if !admission.permits_compaction_movement() {
            return Ok(Self {
                posture: CompactionSourceEvidencePosture::Quarantined(admission),
            });
        }
        if admission.inspected_bytes() == 0 {
            return Err(CompactionReadInterlockDenial::SourceEvidenceMismatch);
        }
        Ok(Self {
            posture: CompactionSourceEvidencePosture::IntegrityClearedStableRead {
                receipt,
                admission,
            },
        })
    }

    pub const fn from_quarantine_admission(admission: CompactionSourceIntegrityAdmission) -> Self {
        Self {
            posture: CompactionSourceEvidencePosture::Quarantined(admission),
        }
    }

    const fn posture(self) -> CompactionSourceEvidencePosture {
        self.posture
    }

    pub const fn stable_read_receipt(self) -> Option<StablePhysicalReadReceipt> {
        match self.posture {
            CompactionSourceEvidencePosture::IntegrityClearedStableRead { receipt, .. } => {
                Some(receipt)
            }
            CompactionSourceEvidencePosture::Quarantined(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionReadInterlockPlan {
    identity: super::plan_identity::CompactionPlanIdentity,
    protected: CompactionProtectedReferenceSet,
    candidates: CompactionCandidateRangeSet,
    source_epoch: RootEpoch,
    target_epoch: RootEpoch,
    counters: CompactionReadInterlockCounters,
    reclaim_deferred: bool,
    source_integrity: CompactionSourceIntegrityEvidence,
}

impl CompactionReadInterlockPlan {
    pub const fn cutover_state(&self) -> super::CompactionCutoverState {
        super::CompactionCutoverState::PlanAdmitted
    }

    pub fn admit(
        protected: CompactionProtectedReferenceSet,
        candidates: CompactionCandidateRangeSet,
        source_epoch: RootEpoch,
        target_epoch: RootEpoch,
        integrity: CompactionSourceIntegrityEvidence,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        if candidates.candidate_references() == 0 {
            return Err(CompactionReadInterlockDenial::EmptyCandidateRangeSet);
        }
        match integrity.posture() {
            CompactionSourceEvidencePosture::Quarantined(_) => {
                return Err(CompactionReadInterlockDenial::QuarantinedCandidateRange);
            }
            CompactionSourceEvidencePosture::IntegrityClearedStableRead { receipt, admission } => {
                let release = receipt.read_plan_release();
                let owner = admission
                    .locality_owner()
                    .ok_or(CompactionReadInterlockDenial::SourceEvidenceMismatch)?;
                if !protected.contains_owner(owner) {
                    return Err(CompactionReadInterlockDenial::SourceEvidenceMismatch);
                }
                if !candidates.is_fully_covered_by_owner(owner) {
                    return Err(CompactionReadInterlockDenial::SourceEvidenceMismatch);
                }
                if release.root() != protected.root()
                    || release.footprint_basis() != protected.footprint_basis()
                {
                    return Err(CompactionReadInterlockDenial::SourceEvidenceMismatch);
                }
            }
        }
        if protected.root().epoch() != source_epoch {
            return Err(CompactionReadInterlockDenial::StaleCompactionSourceEpoch {
                expected: protected.root().epoch(),
                observed: source_epoch,
            });
        }
        if target_epoch.get() <= source_epoch.get() {
            return Err(CompactionReadInterlockDenial::StaleEpochReuse {
                source_epoch,
                reused_epoch: target_epoch,
            });
        }
        let counters = candidates.intersect_protected(&protected);
        let reclaim_deferred = counters.overlapping_ranges() > 0;
        Ok(Self {
            identity: super::plan_identity::CompactionPlanIdentity::issue(),
            protected,
            candidates,
            source_epoch,
            target_epoch,
            counters,
            reclaim_deferred,
            source_integrity: integrity,
        })
    }

    pub fn deny_in_place_overwrite(
        self,
    ) -> (
        CompactionReadInterlockDenial,
        CompactionReadInterlockCounters,
    ) {
        (
            CompactionReadInterlockDenial::InPlaceOverwriteOfProtectedStructure,
            self.counters.with_denied_in_place_overwrite(),
        )
    }

    pub const fn protected(&self) -> &CompactionProtectedReferenceSet {
        &self.protected
    }

    pub const fn candidates(&self) -> &CompactionCandidateRangeSet {
        &self.candidates
    }

    pub const fn source_epoch(&self) -> RootEpoch {
        self.source_epoch
    }

    pub const fn target_epoch(&self) -> RootEpoch {
        self.target_epoch
    }

    pub const fn counters(&self) -> CompactionReadInterlockCounters {
        self.counters
    }

    pub const fn reclaim_deferred(&self) -> bool {
        self.reclaim_deferred
    }

    pub const fn source_integrity(&self) -> CompactionSourceIntegrityEvidence {
        self.source_integrity
    }
}
