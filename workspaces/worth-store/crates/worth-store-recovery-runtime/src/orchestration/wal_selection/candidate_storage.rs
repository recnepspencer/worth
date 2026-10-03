//! Candidate classification retains its prepared frame vectors in native custody.

use super::resident_allocation::WalSelectionAllocation;
use super::tail_selection::{ResidentWalTail, WalTailSelectionDenial};
use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;
use worth_store::physical_runtime::recovery_wal::WalSegmentInspection;
use worth_store::physical_runtime::{
    IntegrityAdmittedRecoveryWalSegment, PhysicalRecoveryCoordination, RecoveryWalAllocationDenial,
};
use worth_store_recovery_physics::{
    classify_admitted_wal_segment, AdmittedWalFrameRejectionKind, AdmittedWalSegmentPolicyInput,
    PhysicalRecoveryResidueKind, PhysicalWalCandidatePreparation, PhysicalWalFrameFacts,
    PhysicalWalSegmentCandidate, PhysicalWalSegmentDisposition,
};

#[derive(Debug, Default)]
pub(crate) struct ResidentWalCandidates {
    // Data is disposed before the reservation on every ordinary/failure exit.
    pub(super) candidates: Vec<PhysicalWalSegmentCandidate>,
    pub(super) allocation: WalSelectionAllocation,
}

pub(crate) enum RecordedWalDisposition {
    Candidate {
        inspection: WalSegmentInspection,
        torn_bytes: u64,
    },
    Residue {
        kind: PhysicalRecoveryResidueKind,
        observed_bytes: u64,
        torn_bytes: u64,
    },
    Corrupt,
}

pub(crate) enum CandidateClassificationDenial {
    Allocation(RecoveryWalAllocationDenial),
    SourceBinding,
}

impl ResidentWalCandidates {
    pub(crate) fn prepare(
        owner: &PhysicalRecoveryCoordination,
        count: usize,
    ) -> Result<Self, RecoveryWalAllocationDenial> {
        let mut allocation = WalSelectionAllocation::default();
        let candidates = allocation.prepare_vector(owner, count)?;
        Ok(Self {
            candidates,
            allocation,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.candidates.len()
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.candidates.capacity()
    }

    #[cfg(test)]
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, PhysicalWalSegmentCandidate> {
        self.candidates.iter()
    }

    #[cfg(test)]
    pub(crate) fn charged_bytes(&self) -> u64 {
        self.allocation.charged_bytes()
    }

    pub(crate) fn classify(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
        identity: WalSegmentArtifactIdentity,
        observed_bytes: u64,
        terminal: bool,
        rejection: Option<AdmittedWalFrameRejectionKind>,
        admitted: Option<&IntegrityAdmittedRecoveryWalSegment>,
    ) -> Result<RecordedWalDisposition, CandidateClassificationDenial> {
        let disposition = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
            identity,
            observed_bytes,
            terminal,
            rejection,
            admitted.map(IntegrityAdmittedRecoveryWalSegment::inspection),
        ));
        match disposition {
            Some(PhysicalWalSegmentDisposition::Candidate {
                preparation,
                torn_bytes,
            }) => self.record_candidate(
                owner,
                preparation,
                admitted.expect("Physics candidate preparation requires an admitted prefix"),
                torn_bytes,
            ),
            Some(PhysicalWalSegmentDisposition::Residue {
                kind,
                observed_bytes,
                torn_bytes,
            }) => Ok(RecordedWalDisposition::Residue {
                kind,
                observed_bytes,
                torn_bytes,
            }),
            Some(PhysicalWalSegmentDisposition::Corrupt) => Ok(RecordedWalDisposition::Corrupt),
            None => Err(CandidateClassificationDenial::SourceBinding),
        }
    }

    fn record_candidate(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
        preparation: PhysicalWalCandidatePreparation,
        segment: &IntegrityAdmittedRecoveryWalSegment,
        torn_bytes: u64,
    ) -> Result<RecordedWalDisposition, CandidateClassificationDenial> {
        let before = self.allocation.charged_bytes();
        let facts = self.prepare_frame_facts(owner, segment)?;
        let Some(candidate) = preparation.bind_frame_facts(facts) else {
            // The failed binding disposed its facts before releasing their backing.
            self.allocation
                .settle_after_disposal(before)
                .map_err(CandidateClassificationDenial::Allocation)?;
            return Err(CandidateClassificationDenial::SourceBinding);
        };
        let inspection = candidate.inspection();
        assert!(
            self.candidates.len() < self.candidates.capacity(),
            "candidate roster must be prepared before classification"
        );
        self.candidates.push(candidate);
        Ok(RecordedWalDisposition::Candidate {
            inspection,
            torn_bytes,
        })
    }

    fn prepare_frame_facts(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
        segment: &IntegrityAdmittedRecoveryWalSegment,
    ) -> Result<Vec<PhysicalWalFrameFacts>, CandidateClassificationDenial> {
        let mut facts = self
            .allocation
            .prepare_vector(owner, segment.frames().len())
            .map_err(CandidateClassificationDenial::Allocation)?;
        for frame in segment.frames() {
            facts.push(
                PhysicalWalFrameFacts::new(frame.lsn_range(), frame.encoded_byte_count())
                    .expect("C9 frame has nonzero encoded bytes"),
            );
        }
        Ok(facts)
    }

    pub(crate) fn select_tail(
        self,
        owner: &PhysicalRecoveryCoordination,
        frontier: u64,
        cutoff: Option<u64>,
    ) -> Result<ResidentWalTail, WalTailSelectionDenial> {
        ResidentWalTail::from_candidates(self, owner, frontier, cutoff)
    }
}
