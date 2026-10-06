//! Workers and memory a pattern holds on the process ledger and a lease's
//! lineage. Workers are checked on every level before memory. A run's entry
//! never waits on workers: finding none free, it runs inline on the thread
//! that asked, which exists whatever the ledger says, and holds only its
//! memory. Only an extra slot a run tries to add is refused as busy.

use std::sync::Arc;

use super::{room, ExecutionResourceLease, LeaseDenial, ResourceReservation};

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
        memory_bytes: u64,
    ) -> Result<ResourceReservation, Busy::Refusal> {
        if process_workers == 0 && worker_count_per_node == 0 && memory_bytes == 0 {
            return Ok(ResourceReservation {
                authority: Arc::clone(&self.authority.inner),
                memory_lineage: Vec::new(),
                worker_lineage: Vec::new(),
                worker_count_per_node: 0,
                process_workers: 0,
                memory_bytes: 0,
            });
        }
        let lineage = self.lineage();
        let config = self.authority.inner.config;
        let mut ledger = self
            .authority
            .inner
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
        room::check(
            &ledger,
            config.charged_memory_bytes,
            0,
            lineage.iter().map(|node| (*node, 0)),
            memory_bytes,
        )
        .map_err(LeaseDenial::MemoryExhausted)?;
        ledger.active_workers += process_workers;
        ledger.charged_memory_bytes += memory_bytes;
        for node in &lineage {
            let usage = ledger.nodes.entry(node.id).or_default();
            if worker_nodes.contains(&node.id) {
                usage.active_workers += worker_count_per_node;
            }
            usage.charged_memory_bytes += memory_bytes;
        }
        Ok(ResourceReservation {
            authority: Arc::clone(&self.authority.inner),
            memory_lineage: lineage.iter().map(|node| node.id).collect(),
            worker_lineage: worker_nodes,
            worker_count_per_node,
            process_workers,
            memory_bytes,
        })
    }
}
