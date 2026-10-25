//! Non-authorizing observations of the same memory lineages reservations use.
use super::{ExecutionMemoryReservation, ExecutionResourceLease, SerialMemoryBudget};

impl SerialMemoryBudget {
    /// Room remaining across every policy in this serial lineage. This is an
    /// observation only; reservations recheck all ledgers atomically.
    pub fn available_memory_bytes(&self) -> Option<u64> {
        self.ledgers
            .iter()
            .filter_map(|ledger| {
                ledger.ceiling.map(|ceiling| {
                    ceiling.saturating_sub(
                        *ledger
                            .charged
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    )
                })
            })
            .min()
    }
}

impl ExecutionMemoryReservation {
    /// Observes every memory ceiling of the active allocation scope, including
    /// serial ancestors or a lease's process. It grants no reservation.
    pub fn available_in_scope(lease: Option<&ExecutionResourceLease<'_>>) -> Option<u64> {
        // A leased computation can sit inside a serial scope. Its lease room
        // alone must not erase the finite enclosing serial policy from a grant.
        lease
            .and_then(ExecutionResourceLease::available_memory_bytes)
            .into_iter()
            .chain(crate::backend::available_serial_memory_bytes())
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serial_room_includes_every_ancestor_and_sibling_hold() {
        let root = SerialMemoryBudget::new(Some(100));
        let middle = SerialMemoryBudget::new(Some(80)).within_parent(&root);
        let child = SerialMemoryBudget::new(None).within_parent(&middle);
        let root_hold = root.reserve(30).unwrap();
        assert_eq!(child.available_memory_bytes(), Some(70));
        let middle_hold = middle.reserve(20).unwrap();
        assert_eq!(child.available_memory_bytes(), Some(50));
        let child_hold = child.reserve(40).unwrap();
        assert_eq!(child.available_memory_bytes(), Some(10));
        assert!(child.reserve(11).is_err());
        drop((root_hold, middle_hold, child_hold));
        assert_eq!(child.available_memory_bytes(), Some(80));
        assert_eq!(SerialMemoryBudget::new(None).available_memory_bytes(), None);
    }
}
