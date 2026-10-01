use sha2::{Digest, Sha256};

/// Flat, precharged duplicate detector. Hashing chooses a probe start only;
/// exact generation/block equality decides membership. A full probe is bounded
/// by the admitted table length even under adversarial collisions.
pub(super) struct SeenHeadBlocks {
    slots: Vec<Option<(u64, u64)>>,
}

impl SeenHeadBlocks {
    pub(super) fn from_reserved(
        mut slots: Vec<Option<(u64, u64)>>,
        max_nodes: u64,
    ) -> Option<Self> {
        let count = slot_count(max_nodes)?;
        if !slots.is_empty() || slots.capacity() < count {
            return None;
        }
        slots.resize(count, None);
        Some(Self { slots })
    }

    pub(super) fn insert(&mut self, key: (u64, u64)) -> Option<bool> {
        let mut hasher = Sha256::new();
        hasher.update(key.0.to_le_bytes());
        hasher.update(key.1.to_le_bytes());
        let digest: [u8; 32] = hasher.finalize().into();
        let mut slot =
            (u64::from_le_bytes(digest[..8].try_into().unwrap()) as usize) % self.slots.len();
        for _ in 0..self.slots.len() {
            match self.slots[slot] {
                Some(existing) if existing == key => return Some(false),
                None => {
                    self.slots[slot] = Some(key);
                    return Some(true);
                }
                Some(_) => slot = (slot + 1) % self.slots.len(),
            }
        }
        None
    }

    pub(super) fn capacity_bytes(&self) -> Option<u64> {
        u64::try_from(self.slots.capacity())
            .ok()?
            .checked_mul(std::mem::size_of::<Option<(u64, u64)>>() as u64)
    }

    pub(super) fn into_slots(self) -> Vec<Option<(u64, u64)>> {
        self.slots
    }
}

pub(super) fn slot_count(max_nodes: u64) -> Option<usize> {
    usize::try_from(max_nodes.checked_mul(2)?)
        .ok()
        .filter(|count| *count > 0)
}

pub(super) fn requested_bytes(max_nodes: u64) -> Option<u64> {
    u64::try_from(slot_count(max_nodes)?)
        .ok()?
        .checked_mul(std::mem::size_of::<Option<(u64, u64)>>() as u64)
}
