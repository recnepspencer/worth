use std::collections::{BTreeMap, VecDeque};

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::ReadyInvalidationBatch;
use crate::data::telemetry::InvalidationPerformedCounter;

use super::deduplication::{same_canonical_work, ReadyWorkKey};

pub(crate) struct ReadyQueueEntry {
    pub(crate) task_index: usize,
    pub(crate) ready: ReadyInvalidationBatch,
}

pub(crate) struct ReadyInvalidationQueue {
    order: VecDeque<ReadyWorkKey>,
    entries: BTreeMap<ReadyWorkKey, ReadyQueueEntry>,
}

impl ReadyInvalidationQueue {
    pub(crate) fn new() -> Self {
        Self {
            order: VecDeque::new(),
            entries: BTreeMap::new(),
        }
    }

    /// Release owner-local frontier storage on success or a stopped admission.
    /// This does not count abandoned entries as performed queue removals.
    pub(crate) fn release(&mut self, graph: &mut SignalGraph) {
        self.order.clear();
        self.entries.clear();
        graph.with_telemetry(|telemetry| {
            telemetry.invalidation.retained_ready_frontier_width = 0;
        });
        graph
            .invalidation_performed_counter_state()
            .set(InvalidationPerformedCounter::RetainedReadyFrontierWidth, 0);
    }

    #[cfg(test)]
    pub(crate) fn insert(
        &mut self,
        graph: &mut SignalGraph,
        entry: ReadyQueueEntry,
    ) -> Result<bool, SignalError> {
        self.insert_with_execution_work(graph, entry, None, None)
    }

    pub(crate) fn insert_with_execution_work(
        &mut self,
        graph: &mut SignalGraph,
        entry: ReadyQueueEntry,
        work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
        preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
    ) -> Result<bool, SignalError> {
        if let Some(work) = work {
            work.checkpoint(self.entries.len().checked_ilog2().unwrap_or(0) as u64 + 2)
                .map_err(|_| {
                    SignalError::invalid_input("ready queue work stopped before admission")
                })?;
        }
        let key = ReadyWorkKey::from_ready(&entry.ready);
        if let Some(existing) = self.entries.get(&key) {
            if !same_canonical_work(&existing.ready, &entry.ready) {
                return Err(SignalError::invalid_input(
                    "same-epoch invalidation dedup encountered different causal authority",
                ));
            }
            graph.with_telemetry(|telemetry| {
                telemetry.invalidation.work_items_admitted += 1;
                telemetry.invalidation.work_items_merged += 1;
                telemetry.invalidation.ready_work_deduplicated += 1;
            });
            let observed = graph.invalidation_performed_counter_state();
            observed.add(InvalidationPerformedCounter::WorkItemsAdmitted, 1);
            observed.add(InvalidationPerformedCounter::WorkItemsMerged, 1);
            return Ok(false);
        }
        if let Some(budget) = preparation {
            let bytes = crate::data::retained_storage::btree_structure_charge::<
                ReadyWorkKey,
                ReadyQueueEntry,
            >(1)
            .map_err(|_| SignalError::invalid_input("ready queue memory overflow"))?;
            budget.claim(bytes.bytes())?;
            if self.order.len() == self.order.capacity() {
                let capacity = self.order.capacity().saturating_mul(2).max(4);
                budget.claim_vec::<ReadyWorkKey>(capacity)?;
                self.order
                    .reserve_exact(capacity.saturating_sub(self.order.len()));
            }
        }
        if self.entries.is_empty() {
            graph
                .invalidation_performed_counter_state()
                .add(InvalidationPerformedCounter::BatchLocalAllocations, 1);
        }
        self.order.push_back(key);
        self.entries.insert(key, entry);
        let width = self.entries.len() as u64;
        graph.with_telemetry(|telemetry| {
            telemetry.invalidation.work_items_admitted += 1;
            telemetry.invalidation.ready_items_enqueued += 1;
            telemetry.invalidation.maximum_ready_frontier_width = telemetry
                .invalidation
                .maximum_ready_frontier_width
                .max(width);
            telemetry.invalidation.retained_ready_frontier_width = width;
        });
        let observed = graph.invalidation_performed_counter_state();
        observed.add(InvalidationPerformedCounter::WorkItemsAdmitted, 1);
        observed.add(InvalidationPerformedCounter::ReadyItemsEnqueued, 1);
        observed.record_max(
            InvalidationPerformedCounter::MaximumReadyFrontierWidth,
            width,
        );
        observed.record_max(InvalidationPerformedCounter::PeakBatchMemoryItems, width);
        observed.set(
            InvalidationPerformedCounter::RetainedReadyFrontierWidth,
            width,
        );
        Ok(true)
    }

    #[cfg(test)]
    pub(crate) fn pop(
        &mut self,
        graph: &mut SignalGraph,
    ) -> Result<Option<ReadyQueueEntry>, SignalError> {
        self.pop_with_execution_work(graph, None)
    }

    pub(crate) fn pop_with_execution_work(
        &mut self,
        graph: &mut SignalGraph,
        work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    ) -> Result<Option<ReadyQueueEntry>, SignalError> {
        if let Some(work) = work {
            work.checkpoint(self.entries.len().checked_ilog2().unwrap_or(0) as u64 + 2)
                .map_err(|_| {
                    SignalError::invalid_input("ready queue work stopped before removal")
                })?;
        }
        let Some(key) = self.order.pop_front() else {
            return Ok(None);
        };
        let entry = self.entries.remove(&key).ok_or_else(|| {
            SignalError::internal("ready invalidation queue order drifted from stored entries")
        })?;
        let retained_width = self.entries.len() as u64;
        graph.with_telemetry(|telemetry| {
            telemetry.invalidation.ready_items_popped += 1;
            telemetry.invalidation.retained_ready_frontier_width = retained_width;
        });
        let observed = graph.invalidation_performed_counter_state();
        observed.add(InvalidationPerformedCounter::ReadyItemsPopped, 1);
        observed.set(
            InvalidationPerformedCounter::RetainedReadyFrontierWidth,
            self.entries.len() as u64,
        );
        Ok(Some(entry))
    }
}
