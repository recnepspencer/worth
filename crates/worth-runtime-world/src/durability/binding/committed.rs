//! Actual physical location travels with the sole original publication witness.
use super::*;
use sha2::{Digest, Sha256};
use worth_foundational::facade::CanonicalDigestId;
use worth_proof::DurablyCommitted;

#[derive(Debug)]
pub(crate) struct WorldCommittedPublication {
    subject: WorldDurableArtifact,
    witness: DurablyCommitted<ActualWorldPublication>,
    position: u64,
    digest: CanonicalDigestId,
}
impl WorldCommittedPublication {
    pub(super) fn new(
        subject: WorldDurableArtifact,
        position: u64,
        digest: CanonicalDigestId,
    ) -> Self {
        Self {
            witness: DurablyCommitted::from_owner(ActualWorldPublication {
                subject: subject.clone(),
            }),
            subject,
            position,
            digest,
        }
    }
    pub(crate) fn subject(&self) -> &WorldDurableArtifact {
        &self.subject
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }
    pub(crate) fn into_witness(self) -> DurablyCommitted<ActualWorldPublication> {
        self.witness
    }
}
pub(super) fn matches_digest(
    subject: &WorldDurableArtifact,
    digest: CanonicalDigestId,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<bool, worth_execution::WorkCeilingDenial> {
    let mut result = None;
    execution.run_native_with_checkpoint(|context| {
        result = Some((|| {
            if context.has_accounting() {
                context
                    .checkpoint(subject.as_bytes().len() as u64)
                    .map_err(worth_execution::WorkCeilingDenial::Stopped)?;
            } else {
                context
                    .poll()
                    .map_err(worth_execution::WorkCeilingDenial::Stopped)?;
            }
            Ok(<[u8; 32]>::from(Sha256::digest(subject.as_bytes())) == *digest.bytes())
        })());
    })?;
    result.expect("request ran digest validation")
}
