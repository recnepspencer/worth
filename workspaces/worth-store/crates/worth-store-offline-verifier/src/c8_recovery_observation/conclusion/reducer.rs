use super::super::artifact_walk::ObservedRecoveryArtifact;
use super::super::observer_evidence::RecoveryObserverEvidenceDigest;
use super::super::wal_topology;
use super::artifacts;
use super::evidence::EvidenceAccumulator;
use super::model::RecoveryObserverConclusion;

pub(super) fn conclude(
    artifacts_to_observe: &[ObservedRecoveryArtifact],
) -> Result<RecoveryObserverConclusion, super::super::RecoveryObserverWalTopologyDenial> {
    wal_topology::validate(artifacts_to_observe)?;
    let (artifact_set_digest, artifact_identity_digest) = artifacts::digests(artifacts_to_observe);
    let mut evidence = EvidenceAccumulator::new();
    for artifact in artifacts_to_observe {
        evidence.observe(artifact);
    }
    let artifact_identities = RecoveryObserverEvidenceDigest::from_parts(
        artifacts_to_observe.len() as u64,
        artifact_identity_digest,
    );
    Ok(RecoveryObserverConclusion::new(
        artifact_set_digest,
        evidence.finish(artifact_identities),
    ))
}
