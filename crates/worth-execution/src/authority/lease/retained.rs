use std::sync::Arc;

use super::{ExecutionResourceLease, LeaseDenial, ResourceReservation};

impl ExecutionResourceLease<'_> {
    pub(crate) fn lineage_depth(&self) -> usize {
        let mut depth = 0;
        let mut node = Some(self.node.as_ref());
        while let Some(current) = node {
            depth += 1;
            node = current.parent.as_deref();
        }
        depth
    }

    /// Atomically resize or transfer an owned retained-memory charge to this
    /// lease. The old charge remains intact if the new lineage cannot admit it.
    pub(crate) fn rebind_retained_memory(
        &self,
        reservation: &mut ResourceReservation,
        memory_bytes: u64,
    ) -> Result<(), LeaseDenial> {
        if !Arc::ptr_eq(&self.authority.inner, &reservation.authority)
            || reservation.process_workers != 0
            || reservation.worker_count_per_node != 0
            || !reservation.worker_lineage.is_empty()
        {
            return Err(LeaseDenial::UnrelatedNestedLease);
        }
        let lineage = self.lineage();
        let mut ledger = self
            .authority
            .inner
            .ledger
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let process_after = ledger
            .charged_memory_bytes
            .checked_sub(reservation.memory_bytes)
            .and_then(|used| used.checked_add(memory_bytes))
            .ok_or(LeaseDenial::ResourceExhausted)?;
        if process_after > self.authority.inner.config.charged_memory_bytes {
            return Err(LeaseDenial::ResourceExhausted);
        }
        for node in &lineage {
            let old = if reservation.memory_lineage.contains(&node.id) {
                reservation.memory_bytes
            } else {
                0
            };
            let used = ledger
                .nodes
                .get(&node.id)
                .map_or(0, |usage| usage.charged_memory_bytes);
            let after = used
                .checked_sub(old)
                .and_then(|value| value.checked_add(memory_bytes))
                .ok_or(LeaseDenial::ResourceExhausted)?;
            if after > node.charged_memory_bytes {
                return Err(LeaseDenial::ResourceExhausted);
            }
        }
        ledger.charged_memory_bytes = process_after;
        for id in &reservation.memory_lineage {
            let usage = ledger
                .nodes
                .get_mut(id)
                .expect("retained reservation has a node");
            usage.charged_memory_bytes -= reservation.memory_bytes;
            if usage.active_workers == 0 && usage.charged_memory_bytes == 0 {
                ledger.nodes.remove(id);
            }
        }
        if memory_bytes != 0 {
            for node in &lineage {
                ledger
                    .nodes
                    .entry(node.id)
                    .or_default()
                    .charged_memory_bytes += memory_bytes;
            }
        }
        reservation.memory_lineage = if memory_bytes == 0 {
            Vec::new()
        } else {
            lineage.iter().map(|node| node.id).collect()
        };
        reservation.memory_bytes = memory_bytes;
        Ok(())
    }
}
