use crate::diagnostics::data::{
    DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticsEntry,
};
use crate::runtime::RelationalRuntime;
use crate::storage::data::{RetentionPassOutcome, RetentionPlan};
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind};

use super::diagnostic_fields::retention_plan_inspection_fields;

mod partition_pass;

use partition_pass::{
    inspect_partition_retention, refresh_entity_retention_state, refresh_relation_retention_state,
    run_partition_retention_pass, trim_live_history, PartitionRetentionPass,
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
                &self.runtime.services.instrumentation,
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
        let mut branch_replay_overlap_entities = 0;
        let mut branch_replay_overlap_relations = 0;

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
            branch_replay_overlap_entities += entity_counts.branch_replay_overlap;

            let relation_counts = inspect_partition_retention::<RelationRecordKind>(
                self.runtime,
                &mut partition,
                retention_fence,
            )?;
            branch_pinned_relations += relation_counts.branch_pinned;
            replay_pinned_relations += relation_counts.replay_pinned;
            snapshot_pinned_relations += relation_counts.snapshot_pinned;
            reclaimable_relations += relation_counts.reclaimable;
            branch_replay_overlap_relations += relation_counts.branch_replay_overlap;
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
        self.runtime
            .publication_authority()
            .push_bounded_diagnostic(
                DiagnosticsScope::Retention,
                DiagnosticsArtifactKind::MinimalSummary,
                vec![RelationalDiagnosticsEntry::new(
                    DiagnosticCode::RetentionPlanInspected,
                    "retention plan inspected with raw pin surface counts",
                    retention_plan_inspection_fields(
                        &plan,
                        branch_replay_overlap_entities,
                        branch_replay_overlap_relations,
                    ),
                )],
            );
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
        let mut writer = self
            .runtime
            .partitions
            .demand_resident_edit(
                execution,
                &self.runtime.services.instrumentation,
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

        writer.visit_partitions(|mut partition| {
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
                        .expect("complexity counter lock poisoned");
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
                        .expect("complexity counter lock poisoned");
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
        for slot in reclaimed {
            self.runtime.record_identity.admit_reclaimed(slot);
        }
        Ok(outcome)
    }

    pub(crate) fn trim_live_history_for_records(
        &self,
        changed_records: &[crate::transactions::data::RecordRef],
        published_version: crate::identity::data::VersionId,
    ) {
        let oldest_visibility_retained_version = self
            .runtime
            .visibility
            .historical_reconstruction_fence_version(published_version);
        let oldest_branch_head_version = self
            .runtime
            .history
            .oldest_branch_head_version()
            .unwrap_or(published_version);
        let oldest_pinned_version =
            oldest_visibility_retained_version.min(oldest_branch_head_version);

        let mut entity_slots = std::collections::BTreeMap::new();
        let mut relation_slots = std::collections::BTreeMap::new();
        for record in changed_records {
            match record {
                crate::transactions::data::RecordRef::Entity(entity_id) => {
                    entity_slots
                        .entry(entity_id.partition_id)
                        .or_insert_with(std::collections::BTreeSet::new)
                        .insert(entity_id.slot_index());
                }
                crate::transactions::data::RecordRef::Relation(relation_id) => {
                    relation_slots
                        .entry(relation_id.partition_id)
                        .or_insert_with(std::collections::BTreeSet::new)
                        .insert(relation_id.slot_index());
                }
            }
        }

        trim_live_history::<EntityRecordKind>(
            self.runtime,
            entity_slots,
            oldest_pinned_version,
            |runtime, trimmed| {
                runtime
                    .services
                    .instrumentation
                    .complexity_counters
                    .lock()
                    .expect("complexity counter lock poisoned")
                    .live_entity_history_entries_trimmed += trimmed;
            },
        );

        trim_live_history::<RelationRecordKind>(
            self.runtime,
            relation_slots,
            oldest_pinned_version,
            |runtime, trimmed| {
                runtime
                    .services
                    .instrumentation
                    .complexity_counters
                    .lock()
                    .expect("complexity counter lock poisoned")
                    .live_relation_history_entries_trimmed += trimmed;
            },
        );
    }

    pub(crate) fn reconcile_changed_record_states(
        &self,
        changed_records: &[crate::transactions::data::RecordRef],
        published_version: crate::identity::data::VersionId,
    ) {
        let retention_fence = self
            .runtime
            .visibility
            .retention_fence_version(published_version);
        for record in changed_records {
            match record {
                crate::transactions::data::RecordRef::Entity(entity_id) => {
                    let retired_at = self
                        .runtime
                        .partitions
                        .partition(entity_id.partition_id)
                        .and_then(|partition| {
                            partition
                                .entity_arena
                                .retired_at_for_slot(entity_id.slot_index())
                        });
                    refresh_entity_retention_state(
                        self.runtime,
                        entity_id.partition_id,
                        entity_id.slot_index(),
                        retired_at,
                        retention_fence,
                    );
                }
                crate::transactions::data::RecordRef::Relation(relation_id) => {
                    let retired_at = self
                        .runtime
                        .partitions
                        .partition(relation_id.partition_id)
                        .and_then(|partition| {
                            partition
                                .relation_arena
                                .retired_at_for_slot(relation_id.slot_index())
                        });
                    refresh_relation_retention_state(
                        self.runtime,
                        relation_id.partition_id,
                        relation_id.slot_index(),
                        retired_at,
                        retention_fence,
                    );
                }
            }
        }
    }
}

impl RelationalRuntime {
    pub(crate) fn retention_authority(&self) -> VisibilityRetentionAuthority<'_> {
        VisibilityRetentionAuthority::new(self)
    }
}
