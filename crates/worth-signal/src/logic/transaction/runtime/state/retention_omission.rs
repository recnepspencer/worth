use serde::Serialize;
use sha2::{Digest, Sha256};

/// Bounded evidence that exact diagnostic receipts left an opt-in retention tail.
/// It cannot reconstruct an omitted receipt or authorize a transition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub(super) struct RetentionOmission {
    count: u64,
    high_water: u64,
    digest: [u8; 32],
}

impl RetentionOmission {
    pub(super) fn absorb<T: Serialize>(&mut self, domain: &'static [u8], ordinal: u64, value: &T) {
        let mut sha = Sha256::new();
        sha.update(b"worth.signal.retention-omission.v1");
        sha.update((domain.len() as u64).to_le_bytes());
        sha.update(domain);
        sha.update(self.digest);
        sha.update(ordinal.to_le_bytes());
        let encoded = serde_json::to_vec(value).expect("retention evidence serializes");
        sha.update((encoded.len() as u64).to_le_bytes());
        sha.update(encoded);
        self.digest = sha.finalize().into();
        self.count = self.count.saturating_add(1);
        self.high_water = self.high_water.max(ordinal);
    }

    pub(super) const fn count(self) -> u64 {
        self.count
    }

    pub(super) fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub(super) const fn high_water(self) -> u64 {
        self.high_water
    }

    pub(super) const fn digest(self) -> [u8; 32] {
        self.digest
    }
}
