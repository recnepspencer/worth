use worth_store_physical_format::{PhysicalRecordFormatDeclaration, RecordArtifactFile};

use super::super::CandidateBuildDenial;
use crate::entry::{
    PhysicalRecoverySuccessorCandidateDenial, PhysicalRecoverySuccessorCandidateMismatch,
};
use crate::progression::planned::basis::RecoveryObservedCandidateArtifact;

pub(super) struct CanonicalCandidateMatch<'observed> {
    pub(super) format: PhysicalRecordFormatDeclaration,
    generation: u64,
    observed: &'observed [RecoveryObservedCandidateArtifact],
    matched: usize,
    largest_scratch_bytes: u64,
}

impl<'observed> CanonicalCandidateMatch<'observed> {
    pub(super) fn new(
        format: PhysicalRecordFormatDeclaration,
        generation: u64,
        artifacts: &'observed [RecoveryObservedCandidateArtifact],
    ) -> Result<Self, CandidateBuildDenial> {
        if artifacts
            .windows(2)
            .any(|pair| pair[0].artifact >= pair[1].artifact)
        {
            return Err(CandidateBuildDenial::Invalid);
        }
        Ok(Self {
            format,
            generation,
            observed: artifacts,
            matched: 0,
            largest_scratch_bytes: 0,
        })
    }

    pub(super) fn match_artifact(
        &mut self,
        artifact: RecordArtifactFile,
        expected: Vec<u8>,
    ) -> Result<(), CandidateBuildDenial> {
        if expected.is_empty() {
            return Err(CandidateBuildDenial::Invalid);
        }
        self.largest_scratch_bytes = self.largest_scratch_bytes.max(
            expected
                .len()
                .try_into()
                .map_err(|_| CandidateBuildDenial::Invalid)?,
        );
        let Ok(index) = self
            .observed
            .binary_search_by_key(&artifact, |observed| observed.artifact)
        else {
            return Err(self.conflict(
                artifact,
                PhysicalRecoverySuccessorCandidateMismatch::SuccessorArtifactInventory,
            ));
        };
        if self.observed[index].bytes.as_ref() != expected.as_slice() {
            return Err(self.conflict(
                artifact,
                PhysicalRecoverySuccessorCandidateMismatch::SuccessorArtifactBytes,
            ));
        }
        self.matched = self
            .matched
            .checked_add(1)
            .ok_or(CandidateBuildDenial::Invalid)?;
        Ok(())
    }

    pub(super) fn finish(self) -> Result<u64, CandidateBuildDenial> {
        if self.matched != self.observed.len() {
            return Err(self.conflict(
                RecordArtifactFile::RootManifest {
                    generation: self.generation,
                },
                PhysicalRecoverySuccessorCandidateMismatch::SuccessorArtifactInventory,
            ));
        }
        Ok(self.largest_scratch_bytes)
    }

    fn conflict(
        &self,
        artifact: RecordArtifactFile,
        mismatch: PhysicalRecoverySuccessorCandidateMismatch,
    ) -> CandidateBuildDenial {
        CandidateBuildDenial::SuccessorCandidate(
            PhysicalRecoverySuccessorCandidateDenial::Conflict {
                artifact,
                generation: self.generation,
                mismatch,
            },
        )
    }
}
