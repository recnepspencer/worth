use std::sync::Arc;

use super::{room, ExecutionResourceLease, LeaseDenial, ResourceReservation};

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
        if !Arc::ptr_eq(&self.authority, &reservation.authority)
            || reservation.process_workers != 0
            || reservation.worker_count_per_node != 0
            || !reservation.worker_lineage.is_empty()
        {
            return Err(LeaseDenial::refused(LeaseDenial::UnrelatedNestedLease));
        }
        let lineage = self.lineage();
        let mut ledger = self
            .authority
            .ledger
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Each level counts this charge's old bytes as room.
        let memory = reservation
            .memory
            .as_mut()
            .expect("retained memory reservation");
        let held = memory.bytes;
        room::check(
            &ledger,
            self.authority.config.charged_memory_bytes,
            held,
            lineage.iter().map(|node| {
                let old = if memory.lineage.contains(&node.id) {
                    held
                } else {
                    0
                };
                (*node, old)
            }),
            memory_bytes,
        )
        .map_err(LeaseDenial::MemoryExhausted)?;
        let old_budget = memory
            .lineage
            .iter()
            .find_map(|id| ledger.serial_budgets.get(id))
            .cloned();
        let new_budget = self.lineage_serial_budget_node();
        match (old_budget.as_ref(), new_budget.map(|(_, b)| b)) {
            (Some(old_b), Some(new_b)) if old_b.same_lineage(new_b) => {
                if held != 0 || memory_bytes != 0 {
                    new_b.resize(held, memory_bytes).map_err(|denial| {
                        LeaseDenial::MemoryExhausted(denial.offset_ancestor(lineage.len() as u32))
                    })?;
                }
            }
            (old_opt, new_opt) => {
                if let Some(new_b) = new_opt
                    && memory_bytes != 0
                {
                    new_b.resize(0, memory_bytes).map_err(|denial| {
                        LeaseDenial::MemoryExhausted(denial.offset_ancestor(lineage.len() as u32))
                    })?;
                }
                if let Some(old_b) = old_opt
                    && held != 0
                {
                    let _ = old_b.resize(held, 0);
                }
            }
        }
        let process_used = ledger.charged_memory_bytes - held;
        let process_after = process_used + memory_bytes;
        ledger.charged_memory_bytes = process_after;
        for id in &memory.lineage {
            let usage = ledger
                .nodes
                .get_mut(id)
                .expect("retained reservation has a node");
            usage.charged_memory_bytes -= held;
            if usage.active_workers == 0 && usage.charged_memory_bytes == 0 {
                ledger.nodes.remove(id);
                ledger.serial_budgets.remove(id);
            }
        }
        if memory_bytes != 0 {
            if let Some((budget_node_id, budget)) = new_budget {
                ledger
                    .serial_budgets
                    .entry(budget_node_id)
                    .or_insert_with(|| budget.clone());
            }
            for node in &lineage {
                ledger
                    .nodes
                    .entry(node.id)
                    .or_default()
                    .charged_memory_bytes += memory_bytes;
            }
        }
        memory.lineage = if memory_bytes == 0 {
            Vec::new()
        } else {
            lineage.iter().map(|node| node.id).collect()
        };
        memory.bytes = memory_bytes;
        let retired = ledger.take_unused_nodes();
        drop(ledger);
        drop(retired);
        Ok(())
    }
}
