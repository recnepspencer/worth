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
    /// Holds `bytes` on this lease, every ancestor and the process until the
    /// reservation drops or is resized.
    pub fn reserve_memory(
        &self,
        bytes: u64,
    ) -> Result<ExecutionMemoryReservation, MemoryLimitDenial> {
        let mut memory = LeaseMemory {
            authority: Arc::clone(&self.authority.inner),
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
        if !Arc::ptr_eq(&self.authority, &lease.authority.inner) {
            return Err(LeaseDenial::UnrelatedNestedLease);
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

impl Drop for LeaseMemory {
    fn drop(&mut self) {
        let _ = self.resize(0);
    }
}
