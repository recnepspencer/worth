use sha2::{Digest, Sha256};

use super::super::artifact_walk::ObservedRecoveryArtifact;
use super::super::observer_evidence_accumulation::EvidenceDigestBuilder;

const ARTIFACT_SET_DOMAIN: &[u8] = b"worth.store.recovery-observer.artifact-set.v1";

pub(super) fn digests(artifacts: &[ObservedRecoveryArtifact]) -> ([u8; 32], [u8; 32]) {
    let mut artifact_set = Sha256::new();
    artifact_set.update(ARTIFACT_SET_DOMAIN);
    artifact_set.update((artifacts.len() as u64).to_le_bytes());
    let mut artifact_identity =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.artifact-identity.v1");
    for artifact in artifacts {
        let mut identity = Vec::with_capacity(8 + artifact.path().len() + 8 + 32);
        identity.extend_from_slice(&(artifact.path().len() as u64).to_le_bytes());
        identity.extend_from_slice(artifact.path().as_bytes());
        identity.extend_from_slice(&artifact.byte_length().to_le_bytes());
        identity.extend_from_slice(&artifact.digest());
        artifact_set.update(&identity);
        artifact_identity.record(&identity);
    }
    (
        artifact_set.finalize().into(),
        artifact_identity.finish().digest(),
    )
}
