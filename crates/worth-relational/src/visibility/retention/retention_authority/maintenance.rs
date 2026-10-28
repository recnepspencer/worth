//! Explicit request-owned reclamation preserves actual progress on every stop.
use super::{RelationalRetentionPassStop as Stop, VisibilityRetentionAuthority};
use crate::execution::{denial::kernel_stop, RelationalExecutionDenialCause as Cause};
use crate::execution::{
    request_stage::{checkpoint, optional_stage},
    result_memory::OriginalResultMemory,
};
use crate::history::data::RecordAllocationClass as Class;
use crate::identity::data::{PartitionId, VersionId};
use crate::storage::data::RetentionPassOutcome;
use crate::storage::substrate::{EntityRecordKind, RecordKind, RelationRecordKind};
use worth_execution::{ExecutionRequest, MapKernelContext};

/// Actual work performed by a pass. Counts are observations, not a cost promise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalRetentionPassProgress {
    outcome: RetentionPassOutcome,
    slots_scanned: usize,
    states_refreshed: usize,
}
impl RelationalRetentionPassProgress {
    pub fn outcome(&self) -> &RetentionPassOutcome {
        &self.outcome
    }
    pub fn slots_scanned(&self) -> usize {
        self.slots_scanned
    }
    pub fn states_refreshed(&self) -> usize {
        self.states_refreshed
    }
    fn empty() -> Self {
        Self {
            outcome: RetentionPassOutcome {
                entity_reclaimable: 0,
                entity_reclaimed: 0,
                entity_chunks_scanned: 0,
                relation_reclaimable: 0,
                relation_reclaimed: 0,
                relation_chunks_scanned: 0,
            },
            slots_scanned: 0,
            states_refreshed: 0,
        }
    }
}

/// Reclamation already performed remains performed. This is never a rollback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalStoppedRetentionPass {
    progress: RelationalRetentionPassProgress,
    stop: Stop,
}
impl RelationalStoppedRetentionPass {
    pub fn progress(&self) -> &RelationalRetentionPassProgress {
        &self.progress
    }
    pub fn stop(&self) -> Stop {
        self.stop
    }
}

impl VisibilityRetentionAuthority<'_> {
    /// Attempt the configured retention policy under the caller's original
    /// request. A live authoritative map edition refuses further writes. Pinned
    /// historical roots retain their own existing visibility/retention contract.
    pub fn try_run_pass(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalRetentionPassProgress, RelationalStoppedRetentionPass> {
        let mut progress = RelationalRetentionPassProgress::empty();
        let result = optional_stage(
            execution,
            |context, memory| self.run_requested(context, memory, &mut progress),
            |_| 0,
        );
        // Prefix lives outside the callback: final original-request refusal may
        // discard its return value, but cannot erase already performed work.
        let stop = match result {
            Ok(Ok(())) => return Ok(progress),
            Ok(Err(stop)) => stop,
            Err(cause) => Stop::Request(cause),
        };
        Err(RelationalStoppedRetentionPass { progress, stop })
    }

    fn run_requested(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        memory: &mut OriginalResultMemory<'_, '_>,
        progress: &mut RelationalRetentionPassProgress,
    ) -> Result<(), Stop> {
        let fence = self
            .runtime
            .visibility
            .retention_fence_version(self.runtime.current_version_id());
        let edition = self.runtime.storage_access().current_edition();
        let partitions_count = edition.len();
        checkpoint(context, partitions_count as u64)
            .map_err(|cause| Stop::Request(kernel_stop(cause)))?;
        let (partitions, held) =
            crate::storage::overlay::partition_ids_in_result_memory(&edition, memory.loan())
                .map_err(Stop::Request)?;
        drop(edition);
        for &partition in &partitions {
            self.run_kind::<EntityRecordKind>(
                Class::Entity,
                partition,
                fence,
                context,
                memory,
                progress,
            )?;
            self.run_kind::<RelationRecordKind>(
                Class::Relation,
                partition,
                fence,
                context,
                memory,
                progress,
            )?;
        }
        drop(partitions);
        memory.release(held).map_err(Stop::Request)
    }

    fn run_kind<K: RecordKind>(
        &self,
        class: Class,
        partition: PartitionId,
        fence: VersionId,
        context: &mut MapKernelContext<'_, '_>,
        memory: &mut OriginalResultMemory<'_, '_>,
        progress: &mut RelationalRetentionPassProgress,
    ) -> Result<(), Stop> {
        let (slots, held) = self
            .requested_slots::<K>(partition, context, memory)
            .map_err(Stop::Request)?;
        let chunk_size = match class {
            Class::Entity => self.runtime.storage_access().entity_chunk_size(),
            Class::Relation => self.runtime.storage_access().relation_chunk_size(),
        }
        .max(1);
        let mut chunk = None;
        let mut reclaimed = 0;
        for &slot in &slots {
            checkpoint(context, 1).map_err(|cause| Stop::Request(kernel_stop(cause)))?;
            let current = slot / chunk_size;
            if chunk != Some(current) {
                chunk = Some(current);
                match class {
                    Class::Entity => progress.outcome.entity_chunks_scanned += 1,
                    Class::Relation => progress.outcome.relation_chunks_scanned += 1,
                }
            }
            progress.slots_scanned += 1;
            self.runtime
                .services
                .instrumentation
                .count(|counts| match class {
                    Class::Entity => counts.retention_entity_slots_scanned += 1,
                    Class::Relation => counts.retention_relation_slots_scanned += 1,
                });
            let base = memory.bytes();
            let step = self.runtime.storage_authority().retention_step::<K>(
                class,
                partition,
                slot,
                fence,
                self.runtime
                    .config
                    .storage
                    .mvcc
                    .auto_reclaim_deleted_records
                    && reclaimed < self.runtime.config.storage.mvcc.reclaim_batch_size,
                Some(&mut |work, bytes| {
                    checkpoint(context, work).map_err(kernel_stop)?;
                    memory.claim(bytes)
                }),
            );
            if let Ok(step) = &step {
                progress.states_refreshed += usize::from(step.refreshed);
                reclaimed += usize::from(step.reclaimed);
                match class {
                    Class::Entity => {
                        progress.outcome.entity_reclaimable += usize::from(step.reclaimable);
                        progress.outcome.entity_reclaimed += usize::from(step.reclaimed);
                    }
                    Class::Relation => {
                        progress.outcome.relation_reclaimable += usize::from(step.reclaimable);
                        progress.outcome.relation_reclaimed += usize::from(step.reclaimed);
                    }
                }
            }
            let temporary = memory
                .bytes()
                .checked_sub(base)
                .expect("maintenance only adds its own step peak");
            memory.release(temporary).map_err(Stop::Request)?;
            step?;
        }
        drop(slots);
        memory.release(held).map_err(Stop::Request)
    }

    fn requested_slots<K: RecordKind>(
        &self,
        partition_id: PartitionId,
        context: &mut MapKernelContext<'_, '_>,
        memory: &mut OriginalResultMemory<'_, '_>,
    ) -> Result<(Vec<usize>, u64), Cause> {
        let edition = self.runtime.storage_access().current_edition();
        // Portable upper bound for this keyed standard-map lookup, matching
        // explicit writer admission without assuming private B-tree geometry.
        checkpoint(context, edition.len() as u64).map_err(kernel_stop)?;
        let Some(partition) = edition.partition(partition_id) else {
            return Ok((Vec::new(), 0));
        };
        let arena = K::arena(partition);
        let mut navigation_bytes = 0;
        let iterator = arena.try_occupied_slots(|work, bytes| {
            checkpoint(context, work).map_err(kernel_stop)?;
            memory.claim(bytes)?;
            navigation_bytes = bytes;
            Ok::<_, Cause>(())
        })?;
        let bytes = (arena.slot_count() as u64)
            .checked_mul(size_of::<usize>() as u64)
            .ok_or(Cause::ChargedBytesOverflow)?;
        memory.claim(bytes)?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(arena.slot_count())
            .map_err(|_| Cause::AllocationFailed)?;
        let actual = (slots.capacity() as u64)
            .checked_mul(size_of::<usize>() as u64)
            .ok_or(Cause::ChargedBytesOverflow)?;
        memory.claim(
            actual
                .checked_sub(bytes)
                .ok_or(Cause::ChargedBytesOverflow)?,
        )?;
        // Initial left descent is charged by try_occupied_slots. Pre-admit the
        // complete remaining navigation: each AVL node is pushed/popped once.
        checkpoint(
            context,
            (arena.slot_count() as u64)
                .checked_mul(2)
                .ok_or(Cause::WorkCounterOverflow)?,
        )
        .map_err(kernel_stop)?;
        for slot in iterator {
            checkpoint(context, 0).map_err(kernel_stop)?;
            slots.push(slot);
        }
        // Iterator and authoritative edition are gone before the writer opens.
        drop(edition);
        memory.release(navigation_bytes)?;
        Ok((slots, actual))
    }
}
