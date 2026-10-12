use std::sync::Arc;

use super::{
    room, AuthorityInner, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionResourceLease,
    LeaseDenial, LeaseNode,
};
use crate::authority::memory::{ExecutionMemoryReservation, MemoryLimitDenial};

/// Memory a caller holds on a lease's lineage and the process ledger.
#[derive(Debug)]
pub(in crate::authority) struct LeaseMemory {
    authority: Arc<AuthorityInner>,
    node: Arc<LeaseNode>,
    bytes: u64,
}

impl ExecutionAuthority {
    /// The workers and memory this process admits.
    pub fn config(&self) -> ExecutionAuthorityConfig {
        self.inner.config
    }
}

impl ExecutionResourceLease<'_> {
    /// Bytes currently held on this lease, including descendant charges.
    /// This observation grants no room; reservations still check the whole lineage.
    pub fn charged_memory_bytes(&self) -> u64 {
        self.authority
            .ledger
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .nodes
            .get(&self.node.id)
            .map_or(0, |usage| usage.charged_memory_bytes)
    }

    /// Observes the room left by every ancestor and the process. It is not a
    /// reservation: admission checks those limits again under the ledger lock.
    pub fn available_memory_bytes(&self) -> u64 {
        let ledger = self
            .authority
            .ledger
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        room::available(
            &ledger,
            self.authority.config.charged_memory_bytes,
            std::iter::successors(Some(self.node.as_ref()), |node| node.parent.as_deref()),
        )
    }

    /// Holds `bytes` on this lease, every ancestor and the process until the
    /// reservation drops or is resized.
    pub fn reserve_memory(
        &self,
        bytes: u64,
    ) -> Result<ExecutionMemoryReservation, MemoryLimitDenial> {
        let mut memory = LeaseMemory {
            authority: Arc::clone(&self.authority),
            node: Arc::clone(&self.node),
            bytes: 0,
        };
        memory.resize(bytes)?;
        Ok(ExecutionMemoryReservation::lease(memory))
    }
}

impl LeaseMemory {
    pub(in crate::authority) const fn bytes(&self) -> u64 {
        self.bytes
    }

    fn lineage(&self) -> impl Iterator<Item = &LeaseNode> {
        std::iter::successors(Some(self.node.as_ref()), |node| node.parent.as_deref())
    }

    /// Holds `bytes` instead, checked under one ledger lock against every
    /// limit on the lineage, with what this holds counted as room.
    pub(in crate::authority) fn resize(&mut self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        let mut ledger = self
            .authority
            .ledger
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let held = self.bytes;
        room::check(
            &ledger,
            self.authority.config.charged_memory_bytes,
            held,
            self.lineage().map(|node| (node, held)),
            bytes,
        )?;
        ledger.charged_memory_bytes = ledger.charged_memory_bytes - held + bytes;
        for node in self.lineage() {
            let usage = ledger.nodes.entry(node.id).or_default();
            usage.charged_memory_bytes = usage.charged_memory_bytes - held + bytes;
            if usage.active_workers == 0 && usage.charged_memory_bytes == 0 {
                ledger.nodes.remove(&node.id);
            }
        }
        self.bytes = bytes;
        Ok(())
    }

    /// Holds `bytes` on `lease`'s lineage in place of what this held on its
    /// own, checked and moved under one ledger lock, so no held byte is ever
    /// released before the new charge stands. A refusal keeps what was held.
    pub(in crate::authority) fn rebind(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        bytes: u64,
    ) -> Result<(), LeaseDenial> {
        if !Arc::ptr_eq(&self.authority, &lease.authority) {
            return Err(LeaseDenial::refused(LeaseDenial::UnrelatedNestedLease));
        }
        let held = self.bytes;
        let old = self.lineage().map(|node| node.id).collect::<Vec<_>>();
        let new = std::iter::successors(Some(lease.node.as_ref()), |node| node.parent.as_deref())
            .collect::<Vec<_>>();
        let mut ledger = self
            .authority
            .ledger
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        room::check(
            &ledger,
            self.authority.config.charged_memory_bytes,
            held,
            new.iter()
                .map(|node| (*node, if old.contains(&node.id) { held } else { 0 })),
            bytes,
        )
        .map_err(LeaseDenial::MemoryExhausted)?;
        ledger.charged_memory_bytes = ledger.charged_memory_bytes - held + bytes;
        for id in old.iter().filter(|_| held != 0) {
            let usage = ledger.nodes.get_mut(id).expect("held memory has a node");
            usage.charged_memory_bytes -= held;
            if usage.active_workers == 0 && usage.charged_memory_bytes == 0 {
                ledger.nodes.remove(id);
            }
        }
        for node in new.iter().filter(|_| bytes != 0) {
            ledger
                .nodes
                .entry(node.id)
                .or_default()
                .charged_memory_bytes += bytes;
        }
        self.node = Arc::clone(&lease.node);
        self.bytes = bytes;
        Ok(())
    }
}

#[cfg(test)]
mod room_tests {
    use super::*;
    use std::num::NonZeroUsize;
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    #[test]
    fn available_room_includes_the_ancestors_other_holds() {
        let request = |bytes| super::super::LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, bytes, 100),
            ),
            cancellation: crate::CancellationToken::new(),
            deadline: None,
        };
        let parent = ExecutionResourceLease::serial(request(1_000)).unwrap();
        let parent_hold = parent.reserve_memory(300).unwrap();
        let child = parent.child(request(800)).unwrap();
        assert_eq!(child.available_memory_bytes(), 700);
        let child_hold = child.reserve_memory(100).unwrap();
        assert_eq!(child.available_memory_bytes(), 600);
        let sibling_hold = parent.reserve_memory(200).unwrap();
        assert_eq!(child.available_memory_bytes(), 400);
        assert_eq!(
            child.reserve_memory(401).unwrap_err(),
            MemoryLimitDenial::refused(401, 400, crate::MemoryLimitLevel::Policy { ancestor: 1 })
        );
        drop(sibling_hold);
        assert_eq!(child.available_memory_bytes(), 600);
        drop(child_hold);
        assert_eq!(child.available_memory_bytes(), 700);
        drop(parent_hold);
        assert_eq!(child.available_memory_bytes(), 800);
    }
}

impl Drop for LeaseMemory {
    fn drop(&mut self) {
        let _ = self.resize(0);
    }
}
