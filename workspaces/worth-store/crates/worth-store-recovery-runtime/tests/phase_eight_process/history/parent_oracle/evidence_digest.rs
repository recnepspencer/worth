use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DigestObservation {
    observations: u64,
    digest: [u8; 32],
}

impl DigestObservation {
    pub(crate) const fn observations(self) -> u64 {
        self.observations
    }

    pub(crate) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(crate) const fn empty() -> Self {
        Self {
            observations: 0,
            digest: [0; 32],
        }
    }
}

#[derive(Debug)]
pub(crate) struct DigestBuilder {
    digest: Sha256,
    observations: u64,
}

impl DigestBuilder {
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

    pub(crate) fn finish(self) -> DigestObservation {
        if self.observations == 0 {
            return DigestObservation::empty();
        }
        let mut digest = self.digest;
        digest.update(self.observations.to_le_bytes());
        DigestObservation {
            observations: self.observations,
            digest: digest.finalize().into(),
        }
    }
}

pub(crate) fn digest_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
