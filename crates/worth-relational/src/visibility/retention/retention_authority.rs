// Diagnostic counters keep their valid numeric prefix on unwind; they grant no authority.
use crate::identity::data::VersionId;
use crate::runtime::RelationalRuntime;
use crate::storage::data::{RetentionPassOutcome, RetentionPlan};
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind};


mod maintenance;
mod partition_pass;
pub use crate::storage::authority::RelationalRetentionPassStop;
pub use maintenance::{RelationalRetentionPassProgress, RelationalStoppedRetentionPass};

use partition_pass::{
    inspect_partition_retention, run_partition_retention_pass, PartitionRetentionPass,
};

pub struct VisibilityRetentionAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> VisibilityRetentionAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    /// Count owned snapshot handles without inspecting record storage.
    pub fn active_snapshot_count(&self) -> usize {
        self.runtime.visibility.active_snapshot_count()
    }

    /// Demand Full custody for the entire mirror before inspecting its retention state.
    pub fn inspect_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RetentionPlan, crate::execution::RelationalExecutionDenialCause> {
        let retention_fence = self
            .runtime
            .visibility
            .retention_fence_version(self.runtime.current_version_id());
        let mut writer = self
            .runtime
            .partitions
            .demand_resident_edit(
                execution,
                &self.runtime.services.instrumentation.complexity_counters,
                &self.runtime.services.symbols,
            )
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        let mut branch_pinned_entities = 0;
        let mut replay_pinned_entities = 0;
        let mut snapshot_pinned_entities = 0;
        let mut branch_pinned_relations = 0;
        let mut replay_pinned_relations = 0;
        let mut snapshot_pinned_relations = 0;
        let mut reclaimable_entities = 0;
        let mut reclaimable_relations = 0;

        writer.visit_partitions(|mut partition| {
            let entity_counts = inspect_partition_retention::<EntityRecordKind>(
                self.runtime,
                &mut partition,
                retention_fence,
            )?;
            branch_pinned_entities += entity_counts.branch_pinned;
            replay_pinned_entities += entity_counts.replay_pinned;
            snapshot_pinned_entities += entity_counts.snapshot_pinned;
            reclaimable_entities += entity_counts.reclaimable;

            let relation_counts = inspect_partition_retention::<RelationRecordKind>(
                self.runtime,
                &mut partition,
                retention_fence,
            )?;
            branch_pinned_relations += relation_counts.branch_pinned;
            replay_pinned_relations += relation_counts.replay_pinned;
            snapshot_pinned_relations += relation_counts.snapshot_pinned;
            reclaimable_relations += relation_counts.reclaimable;
            Ok(())
        })?;

        let plan = RetentionPlan {
            retention_fence_version: retention_fence,
            active_snapshot_count: self.runtime.visibility.active_snapshot_count(),
            branch_pinned_entities,
            replay_pinned_entities,
            snapshot_pinned_entities,
            branch_pinned_relations,
            replay_pinned_relations,
            snapshot_pinned_relations,
            reclaimable_entities,
            reclaimable_relations,
        };
        Ok(plan)
    }

    /// Demand Full custody for the entire mirror before reclaiming any record.
    pub fn run_pass(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RetentionPassOutcome, crate::execution::RelationalExecutionDenialCause> {
        let retention_fence = self
            .runtime
            .visibility
            .retention_fence_version(self.runtime.current_version_id());
        let history_fence = self.history_retention_fence();
        let mut writer = self
            .runtime
            .partitions
            .demand_resident_edit(
                execution,
                &self.runtime.services.instrumentation.complexity_counters,
                &self.runtime.services.symbols,
            )
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        let mut reclaimed = Vec::new();
        let mut outcome = RetentionPassOutcome {
            entity_reclaimable: 0,
            entity_reclaimed: 0,
            entity_chunks_scanned: 0,
            relation_reclaimable: 0,
            relation_reclaimed: 0,
            relation_chunks_scanned: 0,
        };

        let entity_chunk_size = self.runtime.storage_access().entity_chunk_size();
        let relation_chunk_size = self.runtime.storage_access().relation_chunk_size();

        let mut entity_history_trimmed = 0;
        let mut relation_history_trimmed = 0;
        writer.visit_partitions(|mut partition| {
            entity_history_trimmed += self
                .runtime
                .storage_authority()
                .trim_full_live_history::<EntityRecordKind>(&mut partition, history_fence)?;
            relation_history_trimmed += self
                .runtime
                .storage_authority()
                .trim_full_live_history::<RelationRecordKind>(&mut partition, history_fence)?;
            let entity_counts = run_partition_retention_pass::<EntityRecordKind>(
                self.runtime,
                &mut partition,
                PartitionRetentionPass {
                    class: crate::history::data::RecordAllocationClass::Entity,
                    chunk_size: entity_chunk_size,
                    retention_fence,
                },
                |runtime| {
                    let mut counters = runtime
                        .services
                        .instrumentation
                        .complexity_counters
                        .lock()
                        .unwrap_or_else(|error| error.into_inner());
                    counters.retention_entity_slots_scanned += 1;
                },
                &mut reclaimed,
            )?;
            outcome.entity_chunks_scanned += entity_counts.chunks_scanned;
            outcome.entity_reclaimable += entity_counts.reclaimable;
            outcome.entity_reclaimed += entity_counts.reclaimed;

            let relation_counts = run_partition_retention_pass::<RelationRecordKind>(
                self.runtime,
                &mut partition,
                PartitionRetentionPass {
                    class: crate::history::data::RecordAllocationClass::Relation,
                    chunk_size: relation_chunk_size,
                    retention_fence,
                },
                |runtime| {
                    let mut counters = runtime
                        .services
                        .instrumentation
                        .complexity_counters
                        .lock()
                        .unwrap_or_else(|error| error.into_inner());
                    counters.retention_relation_slots_scanned += 1;
                },
                &mut reclaimed,
            )?;
            outcome.relation_chunks_scanned += relation_counts.chunks_scanned;
            outcome.relation_reclaimable += relation_counts.reclaimable;
            outcome.relation_reclaimed += relation_counts.reclaimed;
            Ok(())
        })?;

        drop(writer);
        self.runtime.services.instrumentation.count(|counters| {
            counters.live_entity_history_entries_trimmed += entity_history_trimmed;
            counters.live_relation_history_entries_trimmed += relation_history_trimmed;
        });
        for slot in reclaimed {
            self.runtime.record_identity.admit_reclaimed(slot);
        }
        Ok(outcome)
    }

    fn history_retention_fence(&self) -> VersionId {
        let current = self.runtime.current_version_id();
        let visibility = self
            .runtime
            .visibility
            .historical_reconstruction_fence_version(current);
        let branch = self
            .runtime
            .history
            .oldest_branch_head_version()
            .unwrap_or(current);
        visibility.min(branch)
    }
}

impl RelationalRuntime {
    pub(crate) fn retention_authority(&self) -> VisibilityRetentionAuthority<'_> {
        VisibilityRetentionAuthority::new(self)
    }
}
