//! Workers and memory a pattern holds on the process ledger and a lease's
//! lineage. Workers are checked on every level before memory. A run's entry
//! never waits on workers: finding none free, it runs inline on the thread
//! that asked, which exists whatever the ledger says, and holds only its
//! memory. Only an extra slot a run tries to add is refused as busy.

use std::sync::Arc;

use super::{room, ExecutionResourceLease, LeaseDenial, ReservedMemory, ResourceReservation};

/// What a reservation does when the workers it claims are taken.
pub(in crate::authority) trait OnBusy {
    type Refusal: From<LeaseDenial>;

    /// `Ok` drops the worker claim and holds the memory alone.
    fn busy() -> Result<(), Self::Refusal>;
}

/// A run's entry: busy workers send it inline on its own thread.
pub(in crate::authority) enum RunInline {}

impl OnBusy for RunInline {
    type Refusal = LeaseDenial;

    fn busy() -> Result<(), LeaseDenial> {
        Ok(())
    }
}

/// An extra worker slot: busy workers refuse it.
pub(in crate::authority) enum ClaimSlot {}

impl OnBusy for ClaimSlot {
    type Refusal = SlotRefusal;

    fn busy() -> Result<(), SlotRefusal> {
        // The dispatcher absorbs this answer by running with fewer workers.
        Err(SlotRefusal::WorkersBusy)
    }
}

/// Why an extra worker slot was not claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SlotRefusal {
    /// Every worker the process or the lease's lineage admits is in use.
    WorkersBusy,
    Denied(LeaseDenial),
}

impl From<LeaseDenial> for SlotRefusal {
    fn from(denial: LeaseDenial) -> Self {
        Self::Denied(denial)
    }
}

impl ExecutionResourceLease<'_> {
    pub(super) fn reserve<Busy: OnBusy>(
        &self,
        process_workers: usize,
        worker_nodes: Vec<u64>,
        worker_count_per_node: usize,
        memory_bytes: Option<u64>,
    ) -> Result<ResourceReservation, Busy::Refusal> {
        if process_workers == 0 && worker_count_per_node == 0 && memory_bytes == Some(0) {
            return Ok(ResourceReservation {
                authority: Arc::clone(&self.authority),
                memory: Some(ReservedMemory {
                    lineage: Vec::new(),
                    bytes: 0,
                }),
                worker_lineage: Vec::new(),
                worker_count_per_node: 0,
                process_workers: 0,
            });
        }
        let lineage = self.lineage();
        let config = self.authority.config;
        let mut ledger = self
            .authority
            .ledger
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let fits = |used: usize, added: usize, limit: usize| {
            used.checked_add(added).is_some_and(|count| count <= limit)
        };
        let workers_fit = fits(
            ledger.active_workers,
            process_workers,
            config.max_workers.get(),
        ) && lineage.iter().all(|node| {
            let used = ledger
                .nodes
                .get(&node.id)
                .map_or(0, |usage| usage.active_workers);
            let added = if worker_nodes.contains(&node.id) {
                worker_count_per_node
            } else {
                0
            };
            fits(used, added, node.max_workers)
        });
        let (process_workers, worker_nodes, worker_count_per_node) = if workers_fit {
            (process_workers, worker_nodes, worker_count_per_node)
        } else {
            Busy::busy()?;
            (0, Vec::new(), 0)
        };
        if let Some(memory_bytes) = memory_bytes {
            room::check(
                &ledger,
                config.charged_memory_bytes,
                0,
                lineage.iter().map(|node| (*node, 0)),
                memory_bytes,
            )
            .map_err(LeaseDenial::MemoryExhausted)?;
        }
        ledger.active_workers += process_workers;
        if let Some(bytes) = memory_bytes {
            ledger.charged_memory_bytes += bytes;
        }
        for node in lineage.iter().filter(|node| {
            memory_bytes.is_some_and(|bytes| bytes != 0) || worker_nodes.contains(&node.id)
        }) {
            let usage = ledger.nodes.entry(node.id).or_default();
            if worker_nodes.contains(&node.id) {
                usage.active_workers += worker_count_per_node;
            }
            if let Some(bytes) = memory_bytes {
                usage.charged_memory_bytes += bytes;
            }
        }
        Ok(ResourceReservation {
            authority: Arc::clone(&self.authority),
            memory: memory_bytes.map(|bytes| ReservedMemory {
                // Zero-byte memory has no ledger entry to retain. A sibling
                // reservation may remove an otherwise idle lineage node.
                lineage: if bytes == 0 {
                    Vec::new()
                } else {
                    lineage.iter().map(|node| node.id).collect()
                },
                bytes,
            }),
            worker_lineage: worker_nodes,
            worker_count_per_node,
            process_workers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_byte_inline_entry_outlives_the_worker_that_made_it_inline() {
        let lease = ExecutionResourceLease::serial(crate::tests::request(1, 1024, 100)).unwrap();
        let worker = lease.reserve_worker_entry().unwrap();
        let inline = lease.reserve_entry(0).unwrap();
        drop(worker);
        drop(inline);
        assert_eq!(lease.charged_memory_bytes(), 0);
        assert!(lease.reserve_memory(1024).is_ok());
    }
}
