use sha2::{Digest, Sha256};

use super::block::{ReleaseCustodyHeadBlockReferenceV1, REFERENCE_BYTES};
use super::entry::{ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, ENTRY_BYTES};
use super::ReleaseCustodyHeadDenial;

const DOMAIN: &[u8] = b"store.physical.release-custody-head-roster.v1";

/// Incremental canonical commitment; a digest alone is not selected-tree authority.
pub struct ReleaseCustodyHeadRosterDigestV1 {
    digest: Sha256,
    last: Option<ReleaseCustodyHeadKeyV1>,
    count: u64,
    max_entries: u64,
}

impl ReleaseCustodyHeadRosterDigestV1 {
    pub fn new(root: Option<ReleaseCustodyHeadBlockReferenceV1>, max_entries: u64) -> Self {
        let mut digest = Sha256::new();
        digest.update(DOMAIN);
        let mut encoded = [0_u8; REFERENCE_BYTES];
        if let Some(root) = root {
            digest.update([1]);
            root.encode_into(&mut encoded);
        } else {
            digest.update([0]);
        }
        digest.update(encoded);
        Self {
            digest,
            last: None,
            count: 0,
            max_entries,
        }
    }

    pub fn push(
        &mut self,
        entry: ReleaseCustodyHeadEntryV1,
    ) -> Result<(), ReleaseCustodyHeadDenial> {
        if self.count >= self.max_entries {
            return Err(ReleaseCustodyHeadDenial::Capacity);
        }
        if self.last.is_some_and(|prior| prior >= entry.key()) {
            return Err(ReleaseCustodyHeadDenial::CanonicalOrder);
        }
        let mut bytes = [0_u8; ENTRY_BYTES];
        entry.encode_into(&mut bytes);
        self.digest.update(bytes);
        self.last = Some(entry.key());
        self.count += 1;
        Ok(())
    }

    pub fn finish(mut self) -> (u64, [u8; 32]) {
        self.digest.update(self.count.to_le_bytes());
        (self.count, self.digest.finalize().into())
    }
}
