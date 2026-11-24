use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;
use worth_relational_durability::internal_port::durability::data::recovery_plan::RecoveryPlanOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::original_inputs::CanonicalCommitEnvelopeOwnerPort as _;
pub(crate) use worth_relational_durability::internal_port::durability::authority::runtime_rebuild::{branch_readmission, history_parity, journal_residency, merge_plan, original_inputs, original_schemas, recorded_result_schema, recovered_counter_capacity, recovered_lineage_artifacts, recovered_schema_basis, replay_schedule, root_inventory, world_selection};
mod checkpoint_restore;
pub(super) mod envelope_replay;
pub(super) mod index_catalog;
use original_schemas::OriginalSchemaGraph;

mod transition_replay;

use crate::durability::data::DurabilityError;
use crate::runtime::RelationalRuntime;
use root_inventory::RecoveredRootInventory;

use std::sync::Arc;

use crate::durability::data::RelationalRecoveryStop;

use checkpoint_restore::installation::{
    clear_recovery_partition_pins, finalize_restored_runtime, restore_checkpoint_state,
};
use envelope_replay::replay_readmitted_envelope;
use recovered_counter_capacity::{
    refresh_checkpoint_counters, validate_tail_lineage_allocator_capacity,
};
use recovered_lineage_artifacts::{
    reconcile_recovered_lineage_artifacts, validate_recovered_lineage_artifacts,
};

/// The exclusive state of a runtime this rebuild constructed and has not yet
/// handed to anyone.
///
/// Recovery replaces whole subsystems, which is only honest while the runtime
/// under construction is unshared. Asking for that exclusivity, rather than
/// assuming it, keeps a would-be violation a typed recovery failure instead of
/// a panic.
fn unshared_state(
    restored: &mut RelationalRuntime,
) -> Result<&mut crate::runtime::RelationalRuntimeState, DurabilityError> {
    restored
        .unshared_state_mut()
        .ok_or_else(checkpoint_restore::shared_rebuild_refusal)
}

pub(super) fn rebuild_runtime_from_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admitted: crate::durability::authority::recovery::admission::AdmittedRecoveryPlan,
) -> Result<
    (
        RelationalRuntime,
        crate::durability::data::RecoveryPlan,
        Option<crate::durability::data::CheckpointRestoreWork>,
        RecoveredRootInventory,
    ),
    RelationalRecoveryStop,
> {
    crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let (mut plan, prepared_schemas) = admitted.into_parts();
        let world_selection = world_selection::WorldSelectionReplay::prepare(&plan, execution)?;
        let original_schemas = match prepared_schemas {
            Some(graph) => Some(graph),
            None => OriginalSchemaGraph::admit(&mut plan, execution)?,
        };
        let tail_envelopes = plan.tail_envelopes_in_stream_order();
        validate_tail_lineage_allocator_capacity(tail_envelopes.iter().copied())?;
        validate_recovered_lineage_artifacts(
            plan.checkpoint.as_ref(),
            tail_envelopes.iter().copied(),
        )?;
        drop(tail_envelopes);
        // The rebuilt runtime has no durable medium until recovery binds one after
        // replay, so replaying history never republishes it.
        let mut restored = RelationalRuntime::new(plan.config.clone()).map_err(|denial| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::ReplayFailure,
                format!("rebuild configuration refused: {denial}"),
            )
        })?;
        restored
            .history
            .discard_unobserved_construction_schema_inputs_for_recovery();
        unshared_state(&mut restored)?.commit_strategies.executors =
            plan.commit_strategy_executors().clone();

        if let Some(source) = plan.cold_source() {
            unshared_state(&mut restored)?
                .durability
                .attach_cold_source(source.clone());
        }
        let mut checkpoint_work = None;
        if let Some(checkpoint) = plan.checkpoint.take() {
            let mut work = crate::durability::data::CheckpointRestoreWork::default();
            restore_checkpoint_state(
                lane,
                execution,
                &mut restored,
                checkpoint,
                &mut work,
                original_schemas.as_ref(),
            )?;
            checkpoint_work = Some(work);
        }
        if let Some(original) = plan.retained_transition_prefix() {
            restored
                .history
                .canonical_checkpoint_gate()
                .restore_retained_transition_prefix(original, execution)
                .map_err(|cause| match cause {
                    crate::runtime::OriginalBranchSchemaInputsDenial::Request(cause) => {
                        RelationalRecoveryStop::Request(cause.into())
                    }
                    crate::runtime::OriginalBranchSchemaInputsDenial::SizeOverflow => {
                        RelationalRecoveryStop::Request(
                            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                        )
                    }
                    crate::runtime::OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
                        crate::durability::checkpoints::region_directory::corrupt(
                            "retained Memory transition prefix mismatch",
                        )
                        .into()
                    }
                })?;
        }
        // Tail replay performs ordinary compare-and-publish cutovers. Seed its
        // fixed-depth current-head index from the exact restored branch cells
        // before the first replayed movement tries to retire a prior head.
        restored.history.rebuild_branch_head_version_index(lane);

        refresh_checkpoint_counters(
            &restored.history,
            &restored.lineage,
            restored.durability.checkpoint_selection(),
        )?;
        clear_recovery_partition_pins(&mut restored);
        if let Some(graph) = &original_schemas {
            original_inputs::restore_prefix(
                &restored.history,
                &*restored.history.canonical_checkpoint_gate(),
                &restored.config.history.main_branch,
                restored.durability.checkpoint_selection(),
                graph,
                execution,
            )?;
            original_inputs::install_if_due(
                &restored.history,
                &*restored.history.canonical_checkpoint_gate(),
                graph,
                execution,
            )?;
        }

        let schedule = replay_schedule::RecoveryReplaySchedule::prepare(&plan, &restored.history);
        let mut recovered_roots = RecoveredRootInventory::capture(
            &restored.history,
            &restored.config.schema.registry,
            restored.durability.checkpoint_selection(),
        )?;
        let replay_schedule::RecoveryReplaySchedule {
            commits: pending_tail,
            mut transitions,
            available_commit_ids,
            positioned_exactly,
        } = schedule;
        let mut finalized_tail = Vec::with_capacity(pending_tail.len());
        for pending in pending_tail {
            for transition in crate::durability::data::RecoveryBranchTransitions::take_before(
                &mut transitions,
                pending.position(),
            ) {
                if let Some(selection) = &world_selection {
                    selection.before_transition(
                        execution,
                        &world_replay_inputs(&restored),
                        &plan,
                        &mut recovered_roots,
                        &transition,
                    )?;
                }
                transition_replay::replay_branch_transition(
                    execution,
                    &mut restored,
                    &transition,
                    positioned_exactly,
                    &mut recovered_roots,
                )?;
                if let Some(graph) = &original_schemas {
                    original_inputs::install_if_due(
                        &restored.history,
                        &*restored.history.canonical_checkpoint_gate(),
                        graph,
                        execution,
                    )?;
                }
            }
            if let Some(selection) = &world_selection {
                selection.before(
                    execution,
                    &world_replay_inputs(&restored),
                    &plan,
                    &mut recovered_roots,
                    &pending,
                )?;
            }
            if let Some(graph) = &original_schemas {
                original_inputs::require_replay_prestate(
                    &restored.history,
                    graph,
                    pending.envelope(),
                )?;
                let inputs = pending
                    .envelope()
                    .recorded_original_inputs()
                    .ok_or_else(|| {
                        crate::durability::checkpoints::region_directory::corrupt(
                            "Home tail original inputs missing",
                        )
                    })?;
                restored.restore_source_replay_configuration(
                    &inputs.mutation,
                    graph.get(inputs.result_schema)?.registry(),
                );
            }
            let recorded_result_schema = match &original_schemas {
                Some(graph) => original_inputs::recorded_result_schema(graph, pending.envelope())?,
                None => None,
            };
            let commit_id = pending.envelope().commit.commit_id;
            let restore_authoritative_envelope =
                plan.should_restore_authoritative_envelope(commit_id);
            let positioned = replay_readmitted_envelope(
                execution,
                &mut restored,
                pending,
                &available_commit_ids,
                restore_authoritative_envelope,
                &mut recovered_roots,
                recorded_result_schema,
            )?;
            if let Some(graph) = &original_schemas {
                original_inputs::bind_replayed_schema(
                    lane,
                    &restored.history,
                    &restored.services.symbols,
                    graph,
                    &positioned,
                    execution,
                )?;
            }
            // Register only the finalized source/schema-bound owner before another
            // canonical entry or transition can resolve this commit's root.
            recovered_roots.retain_current(
                &restored.history,
                &restored.config.schema.registry,
                &positioned.envelope().branch_context,
            )?;
            finalized_tail.push(positioned);
        }
        for transition in transitions {
            if let Some(selection) = &world_selection {
                selection.before_transition(
                    execution,
                    &world_replay_inputs(&restored),
                    &plan,
                    &mut recovered_roots,
                    &transition,
                )?;
            }
            transition_replay::replay_branch_transition(
                execution,
                &mut restored,
                &transition,
                positioned_exactly,
                &mut recovered_roots,
            )?;
            if let Some(graph) = &original_schemas {
                original_inputs::install_if_due(
                    &restored.history,
                    &*restored.history.canonical_checkpoint_gate(),
                    graph,
                    execution,
                )?;
            }
        }
        if let Some(selection) = &world_selection {
            let world = selection.finish(
                execution,
                &world_replay_inputs(&restored),
                &plan,
                &mut recovered_roots,
            )?;
            restored
                .history
                .canonical_checkpoint_gate()
                .retain_home_world_selection(world);
        }
        if original_schemas.is_some() {
            restored.restore_source_replay_configuration(
                &crate::config::data::MutationConfig {
                    cascade_delete_policy: plan.config.storage.cascade_delete_policy,
                    adjacency_policy: plan.config.storage.adjacency_policy.clone(),
                    cross_context_policy: plan.config.storage.cross_context_policy,
                },
                &plan.config.schema.registry,
            );
        }
        restored.record_identity.release_unowned_reservations();
        let recovered_checkpoint = restored.durability.latest_checkpoint();
        let mut recovered_canonical_commits = recovered_checkpoint
            .iter()
            .flat_map(|checkpoint| checkpoint.envelopes().iter())
            .cloned()
            .map(Arc::new)
            .collect::<Vec<_>>();
        for durable in &finalized_tail {
            let selected = if plan.should_restore_authoritative_envelope(durable.commit.commit_id) {
                Arc::new(durable.clone())
            } else {
                restored
                    .history
                    .positioned_canonical_commit(durable.commit.commit_id)
                    .ok_or_else(|| {
                        DurabilityError::new(
                            crate::durability::data::RecoveryFailureClass::ReplayFailure,
                            "replayed commit has no positioned canonical route",
                        )
                    })?
            };
            recovered_canonical_commits.push(selected);
        }
        let recovered_symbols = restored.services.symbols.interner_snapshot();
        let history = &mut unshared_state(&mut restored)?.history;
        crate::runtime::state::subsystems::history::history_catalog_recovery::inventory::install_recovered_canonical_inventory(
            history, execution, &recovered_canonical_commits, &recovered_symbols,
        ).map_err(RelationalRecoveryStop::from)?;
        history.rebuild_recovered_canonical_routes(recovered_canonical_commits)
            .map_err(|detail| crate::runtime::state::subsystems::history::history_recovery_denial::RelationalHistoryRecoveryDenial::Invalid(detail.to_owned()))
            .map_err(RelationalRecoveryStop::from)?;
        reconcile_recovered_lineage_artifacts(lane, &restored.lineage, &finalized_tail)?;
        refresh_checkpoint_counters(
            &restored.history,
            &restored.lineage,
            restored.durability.checkpoint_selection(),
        )?;
        let log = finalized_tail
            .iter()
            .map(|commit| {
                let id = commit.commit.commit_id;
                restored
                    .history
                    .commit_artifact(id)
                    .map(|artifact| (id, commit.position(), artifact.envelope_residency()))
                    .ok_or_else(|| {
                        DurabilityError::new(
                            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                            "recovered durable log has no catalog entry",
                        )
                    })
            })
            .collect::<Result<_, _>>()?;
        restored.durability.set_log(lane, log);

        finalize_restored_runtime(lane, execution, &mut restored)?;
        journal_residency::install(execution, &restored.history, &plan)?;
        if let Some(graph) = &original_schemas {
            original_inputs::restore_catalog(&restored.history, graph, &plan, execution)?;
            restored
                .history
                .canonical_checkpoint_gate()
                .install_checkpoint_catalog(restored.history.commit_artifacts());
        }
        Ok((restored, plan, checkpoint_work, recovered_roots))
    })
}

fn world_replay_inputs(runtime: &RelationalRuntime) -> world_selection::WorldReplayInputs<'_> {
    world_selection::WorldReplayInputs {
        history: &runtime.history,
        registry: &runtime.config.schema.registry,
        symbols: &runtime.services.symbols,
        checkpoint: runtime.durability.checkpoint_selection(),
        runtime_instance_id: runtime.runtime_instance_id(),
        visibility: runtime.into(),
    }
}
