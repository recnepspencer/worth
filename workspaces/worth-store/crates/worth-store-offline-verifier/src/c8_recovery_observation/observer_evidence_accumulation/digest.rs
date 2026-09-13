use sha2::{Digest, Sha256};

use super::super::observer_evidence::RecoveryObserverEvidenceDigest;

#[derive(Debug)]
pub(crate) struct EvidenceDigestBuilder {
    digest: Sha256,
    observations: u64,
}

impl EvidenceDigestBuilder {
    pub(crate) fn new(domain: &[u8]) -> Self {
        let mut digest = Sha256::new();
        digest.update(domain);
        Self {
            digest,
            observations: 0,
        }
    }

    pub(crate) fn record(&mut self, bytes: &[u8]) {
        self.digest.update((bytes.len() as u64).to_le_bytes());
        self.digest.update(bytes);
        self.observations = self.observations.saturating_add(1);
    }

    pub(crate) fn finish(self) -> RecoveryObserverEvidenceDigest {
        if self.observations == 0 {
            return RecoveryObserverEvidenceDigest::empty();
        }
        let mut digest = self.digest;
        digest.update(self.observations.to_le_bytes());
        RecoveryObserverEvidenceDigest::from_parts(self.observations, digest.finalize().into())
    }
}
