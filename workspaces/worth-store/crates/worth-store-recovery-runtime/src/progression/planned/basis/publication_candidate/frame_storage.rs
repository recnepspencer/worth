//! Retained candidate frames and their descriptor backing share the planning ledger.
use super::{CandidateBuild, CandidateBuildDenial, RecoveryPublicationCandidateArtifact};
use sha2::{Digest, Sha256};
use worth_store_physical_format::RecordArtifactFile;

impl CandidateBuild<'_> {
    /// The encoder has already charged the Vec's actual backing.
    pub(super) fn push(
        &mut self,
        artifact: RecordArtifactFile,
        bytes: Vec<u8>,
    ) -> Result<(), CandidateBuildDenial> {
        let bytes = self.allowance.into_box(bytes)?;
        self.push_owned(artifact, bytes)
    }

    /// Observed frames move without copying or acquiring another charge.
    pub(super) fn push_owned(
        &mut self,
        artifact: RecordArtifactFile,
        bytes: Box<[u8]>,
    ) -> Result<(), CandidateBuildDenial> {
        if bytes.is_empty()
            || self
                .artifacts
                .iter()
                .any(|candidate| candidate.artifact == artifact)
        {
            return Err(CandidateBuildDenial::Invalid);
        }
        self.allowance.grow(&mut self.artifacts, 1)?;
        let payload_digest = Sha256::digest(bytes.as_ref()).into();
        self.artifacts.push(RecoveryPublicationCandidateArtifact {
            artifact,
            bytes,
            payload_digest,
        });
        Ok(())
    }
}
