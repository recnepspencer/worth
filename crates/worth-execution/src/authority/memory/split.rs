//! Partition a freshly owned admission without charging its bytes twice.
use super::{ExecutionMemoryReservation, Held};

impl ExecutionMemoryReservation {
    /// Only untracked admissions can enter a storage path pool. Each returned
    /// piece has independent final-drop custody; no source request is retained.
    pub(crate) fn split_untracked(&mut self, bytes: u64) -> Option<Self> {
        if self.activity.is_some() || bytes > self.bytes() {
            return None;
        }
        let held = match &mut self.held {
            Held::Lease(memory) => Held::Lease(memory.split_owned(bytes)?),
            Held::Serial {
                budget,
                bytes: owned,
            } => {
                *owned -= bytes;
                Held::Serial {
                    budget: budget.clone(),
                    bytes,
                }
            }
        };
        Some(Self {
            held,
            activity: None,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn splitting_owned_admission_preserves_the_total_and_releases_each_piece_independently() {
        let bytes = (8 * std::mem::size_of::<u64>()) as u64;
        let budget = super::super::SerialMemoryBudget::new(Some(bytes));
        let mut original = budget.reserve(bytes).unwrap();
        let first = original.split_untracked(bytes / 4).unwrap();
        let second = original.split_untracked(bytes / 2).unwrap();
        assert_eq!(
            budget.charged_memory_bytes(),
            original.bytes() + first.bytes() + second.bytes()
        );
        assert!(original.split_untracked(bytes).is_none());
        drop(first);
        assert_eq!(
            budget.charged_memory_bytes(),
            original.bytes() + second.bytes()
        );
        drop(original);
        assert_eq!(budget.charged_memory_bytes(), second.bytes());
        drop(second);
        assert_eq!(budget.charged_memory_bytes(), 0);

        let lease = crate::tests::authority()
            .request_lease(crate::tests::accounted_request(1, 100))
            .unwrap();
        let mut original = lease.reserve_memory(bytes).unwrap();
        let part = original.split_untracked(bytes / 2).unwrap();
        assert_eq!(
            lease.charged_memory_bytes(),
            original.bytes() + part.bytes()
        );
        drop(original);
        assert_eq!(lease.charged_memory_bytes(), part.bytes());
        drop(part);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
