use crate::durability::data::RelationalRecoveryStop;
mod branch_readmission;
mod checkpoint_restore;
mod envelope_replay;
mod index_catalog;
pub(crate) use envelope_replay::replay_commit_stop;
pub(crate) use index_catalog::RecoveredIndexCatalog;
mod history_parity;
mod journal_residency;
mod merge_plan;
mod original_inputs;
mod original_schemas;
pub(crate) use original_schemas::OriginalSchemaGraph;
mod recovered_counter_capacity;
mod recovered_lineage_artifacts;
mod recovered_schema_basis;
mod root_inventory;
mod transition_replay;
mod world_selection;

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::durability::data::DurabilityError;
use crate::runtime::RelationalRuntime;

use checkpoint_restore::{
    clear_recovery_partition_pins, finalize_restored_runtime, restore_checkpoint_state,
};
use envelope_replay::replay_readmitted_envelope;
use recovered_counter_capacity::{
    refresh_checkpoint_counters, validate_tail_lineage_allocator_capacity,
};
use recovered_lineage_artifacts::{
    reconcile_recovered_lineage_artifacts, validate_recovered_lineage_artifacts,
};
pub(crate) use root_inventory::RecoveredRootInventory;

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
    restored.unshared_state_mut().ok_or_else(|| {
        DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::ReplayFailure,
            "recovery cannot replace the subsystems of a runtime already bound to a service",
        )
    })
}

pub(super) fn rebuild_runtime_from_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admitted: super::recovery::admission::AdmittedRecoveryPlan,
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
            plan.commit_strategy_executors.clone();

        if let Some(source) = &plan.cold_source {
            unshared_state(&mut restored)?
                .durability
                .attach_cold_source(source.clone());
        }
        let mut checkpoint_work = None;
        if let Some(checkpoint) = plan.checkpoint.take() {
            let mut work = crate::durability::data::CheckpointRestoreWork::default();
            restore_checkpoint_state(
                execution,
                &mut restored,
                checkpoint,
                &mut work,
                original_schemas.as_ref(),
            )?;
            checkpoint_work = Some(work);
        }
        if let Some(original) = &plan.retained_transition_prefix {
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

        refresh_checkpoint_counters(&mut restored)?;
        clear_recovery_partition_pins(&mut restored);
        if let Some(graph) = &original_schemas {
            original_inputs::restore_prefix(&restored, graph, execution)?;
            original_inputs::install_if_due(&restored, graph, execution)?;
        }

        let mut pending_tail = plan.tail_log.clone();
        pending_tail.sort_by_key(|commit| commit.position());
        let available_commit_ids = restored
            .history
            .commit_artifacts()
            .into_iter()
            .map(|artifact| artifact.commit_id())
            .chain(
                pending_tail
                    .iter()
                    .map(|commit| commit.envelope().commit.commit_id),
            )
            .collect::<BTreeSet<_>>();
        let mut recovered_roots = RecoveredRootInventory::capture(&restored)?;

        // Branch transitions merge into the commit tail by the position each
        // follows: a transition performed after commit p and before commit p + 1
        // is applied before commit p + 1 replays (D8).
        let positioned_exactly = plan.branch_transitions.replays_positions_exactly();
        let mut pending_transitions = plan
            .branch_transitions
            .transitions()
            .iter()
            .cloned()
            .collect::<std::collections::VecDeque<_>>();
        let mut finalized_tail = Vec::with_capacity(pending_tail.len());
        for pending in pending_tail {
            for transition in crate::durability::data::RecoveryBranchTransitions::take_before(
                &mut pending_transitions,
                pending.position(),
            ) {
                if let Some(selection) = &world_selection {
                    selection.before_transition(
                        execution,
                        &restored,
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
                    original_inputs::install_if_due(&restored, graph, execution)?;
                }
            }
            if let Some(selection) = &world_selection {
                selection.before(execution, &restored, &plan, &mut recovered_roots, &pending)?;
            }
            if let Some(graph) = &original_schemas {
                original_inputs::require_replay_prestate(&restored, graph, pending.envelope())?;
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
            )?;
            if let Some(graph) = &original_schemas {
                original_inputs::bind_replayed_schema(
                    lane,
                    &mut restored,
                    graph,
                    &positioned,
                    execution,
                )?;
            }
            // Register only the finalized source/schema-bound owner before another
            // canonical entry or transition can resolve this commit's root.
            recovered_roots.retain_current(&restored, &positioned.envelope().branch_context)?;
            finalized_tail.push(positioned);
        }
        for transition in pending_transitions {
            if let Some(selection) = &world_selection {
                selection.before_transition(
                    execution,
                    &restored,
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
                original_inputs::install_if_due(&restored, graph, execution)?;
            }
        }
        if let Some(selection) = &world_selection {
            selection.finish(execution, &restored, &plan, &mut recovered_roots)?;
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
            .flat_map(|checkpoint| checkpoint.envelopes.iter())
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
        unshared_state(&mut restored)?
            .history
            .install_recovered_canonical_inventory(
                execution,
                recovered_canonical_commits,
                &recovered_symbols,
            )
            .map_err(RelationalRecoveryStop::from)?;
        reconcile_recovered_lineage_artifacts(lane, &mut restored, &finalized_tail)?;
        refresh_checkpoint_counters(&mut restored)?;
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
        journal_residency::install(execution, &restored, &plan)?;
        if let Some(graph) = &original_schemas {
            original_inputs::restore_catalog(&restored, graph, &plan, execution)?;
        }
        Ok((restored, plan, checkpoint_work, recovered_roots))
    })
}
