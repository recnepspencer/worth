use super::super::observer_evidence_accumulation::{
    RecoveryObserverArtifactEvidence, RecoveryObserverResidueObservation,
};
use super::super::physical_format;
use super::wal_prefix_progression::WalPrefixProgression;

pub(super) fn finish(
    bytes: &[u8],
    prefix: WalPrefixProgression,
) -> RecoveryObserverArtifactEvidence {
    let prefix = prefix.finish(bytes.len() as u64);
    let mut evidence = RecoveryObserverArtifactEvidence {
        generation_links: prefix.generation_links,
        wal_prefix: Some(prefix.wal),
        wal_topology: prefix.topology,
        ..RecoveryObserverArtifactEvidence::empty()
    };
    if prefix.offset < bytes.len() {
        evidence.residue = RecoveryObserverResidueObservation {
            bytes: (bytes.len() - prefix.offset) as u64,
            digest: physical_format::digest_bytes(&bytes[prefix.offset..]),
        };
    }
    evidence
}
