use crate::clock::RuntimeInstant;
use crate::data::error::SignalError;
use crate::diagnostics::replay::ReplayEventKind;
use crate::diagnostics::{ExecutionFailureContext, ExecutionFailurePhase, RollbackDiagnostic};

use super::super::transaction_types::{
    CreatedNodeRollbackDelta, GraphPatchRollbackDelta, SignalTransaction,
    SubscriberRepairRollbackDelta, TransactionOutcome, TransactionReplayEntry, TransactionResult,
};
use crate::logic::transaction::runtime::transaction::ObservationBoundaryOutcome;

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn rollback(self) -> Result<TransactionResult, SignalError> {
        self.rollback_with_restoration_status()
            .map(|(result, _)| result)
    }

    pub(in crate::logic::transaction::runtime) fn rollback_for_canonical_owner(
        self,
    ) -> Result<TransactionResult, SignalError> {
        let (result, restoration_error) = self.rollback_with_restoration_status()?;
        match restoration_error {
            Some(error) => Err(error),
            None => Ok(result),
        }
    }

    fn rollback_with_restoration_status(
        mut self,
    ) -> Result<(TransactionResult, Option<SignalError>), SignalError> {
        let commit_start = RuntimeInstant::now();
        if self.finished {
            return Err(SignalError::transaction_finished());
        }
        self.finished = true;
        let rollback_patch_count = self.rollback_patch_count();
        let touched_nodes = self.scratch.graph_patches.touched_nodes(self.graph).len() as u32;
        self.event_bus
            .rollback_with_capture(self.runtime_ctx, self.captures_optional_telemetry());
        self.stage_graph_rollback_packets()?;
        if !self.poisoned {
            self.with_telemetry(|telemetry| telemetry.transaction.transaction_rollback_count += 1);
        }
        let max_touched_nodes = self
            .telemetry_snapshot()
            .transaction
            .max_touched_nodes_in_txn;
        let captures_rollback = self.graph.captures_rollback_diagnostics();
        self.scratch.semantic_delta.rollback = captures_rollback.then(|| {
            RollbackDiagnostic::new(
                true,
                rollback_patch_count,
                max_touched_nodes,
                Some(if self.poisoned {
                    "poisoned transaction rollback".to_string()
                } else {
                    "explicit rollback".to_string()
                }),
                self.scratch.semantic_delta.event_epochs.clone(),
            )
        });
        let (_, observation) = self
            .scratch
            .observations
            .drain_delivery_boundary(ObservationBoundaryOutcome::RollbackSuppressed);
        self.with_telemetry(|telemetry| {
            telemetry.transaction.rollback_suppressed_observation_count +=
                u64::from(observation.rollback_suppressed_event_count);
        });
        self.scratch.semantic_delta.observation = observation;
        if self.poisoned {
            self.with_telemetry(|telemetry| telemetry.transaction.transaction_poison_count += 1);
            if self.graph.captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::ReplayDetail,
            ) {
                self.scratch
                    .semantic_delta
                    .replay_events
                    .push(TransactionReplayEntry {
                        kind: ReplayEventKind::TransactionRolledBack,
                        detail: "poisoned transaction rollback".to_string(),
                        execution_record_id: None,
                        semantic_segment_id: None,
                    });
            }
            return Ok(self.finalize_semantic_delta_with_rollback_status(
                true,
                TransactionOutcome::Poisoned,
                touched_nodes,
                commit_start.elapsed().as_nanos(),
            ));
        }
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::ReplayDetail,
        ) {
            self.scratch
                .semantic_delta
                .replay_events
                .push(TransactionReplayEntry {
                    kind: ReplayEventKind::TransactionRolledBack,
                    detail: "explicit rollback".to_string(),
                    execution_record_id: None,
                    semantic_segment_id: None,
                });
        }
        Ok(self.finalize_semantic_delta_with_rollback_status(
            true,
            TransactionOutcome::RolledBack,
            touched_nodes,
            commit_start.elapsed().as_nanos(),
        ))
    }

    pub(super) fn stage_graph_rollback_packets(&mut self) -> Result<(), SignalError> {
        let graph_patches = std::mem::take(&mut self.scratch.graph_patches);
        let created_nodes = std::mem::take(&mut self.scratch.created_nodes);
        let mut rewired_sources =
            graph_patches.collect_dependency_sources_for_rollback(self.graph)?;
        for &node in &created_nodes {
            if self.graph.is_alive(node) {
                rewired_sources.extend(self.graph.dependency_sources_of(node)?);
            }
        }
        if !graph_patches.is_empty() {
            self.rollback_packets
                .stage_graph_patches(GraphPatchRollbackDelta {
                    patches: graph_patches,
                })?;
        }
        if !created_nodes.is_empty() {
            self.rollback_packets
                .stage_created_nodes(CreatedNodeRollbackDelta { created_nodes })?;
        }
        if !rewired_sources.is_empty() {
            self.rollback_packets
                .stage_subscriber_repair(SubscriberRepairRollbackDelta {
                    sources: rewired_sources,
                })?;
        }
        self.scratch.dirty_targets.clear_all();
        Ok(())
    }

    pub(super) fn rollback_patch_count(&self) -> u64 {
        self.scratch.graph_patches.touched_count() as u64 + self.scratch.created_nodes.len() as u64
    }

    pub(super) fn fail_and_rollback(
        &mut self,
        rollback_reason: &str,
        error: Option<SignalError>,
        fallback_failure_message: Option<&str>,
        failure_phase: ExecutionFailurePhase,
        increment_poison_count: bool,
        outcome: Result<TransactionOutcome, SignalError>,
        commit_start: RuntimeInstant,
    ) -> Result<TransactionResult, SignalError> {
        let rollback_patch_count = self.rollback_patch_count();
        let touched_nodes = self.scratch.graph_patches.touched_nodes(self.graph).len() as u32;
        self.event_bus
            .rollback_with_capture(self.runtime_ctx, self.captures_optional_telemetry());
        self.stage_graph_rollback_packets()?;
        let captures_rollback = self.graph.captures_rollback_diagnostics();
        self.scratch.semantic_delta.rollback = captures_rollback.then(|| {
            RollbackDiagnostic::new(
                true,
                rollback_patch_count,
                self.telemetry_snapshot()
                    .transaction
                    .max_touched_nodes_in_txn,
                Some(rollback_reason.to_string()),
                self.scratch.semantic_delta.event_epochs.clone(),
            )
        });
        let (_, observation) = self
            .scratch
            .observations
            .drain_delivery_boundary(ObservationBoundaryOutcome::RollbackSuppressed);
        self.with_telemetry(|telemetry| {
            telemetry.transaction.rollback_suppressed_observation_count +=
                u64::from(observation.rollback_suppressed_event_count);
        });
        self.scratch.semantic_delta.observation = observation;
        if self.graph.captures_failure_diagnostics() {
            let profile = self.graph.diagnostics_profile();
            self.scratch.semantic_delta.failure_summary = Some(match &error {
                Some(err) => ExecutionFailureContext::from_error(failure_phase, err, None)
                    .summarize(self.scratch.semantic_delta.rollback.as_ref(), profile),
                None => ExecutionFailureContext::new(
                    failure_phase,
                    None,
                    None,
                    None,
                    None,
                    None,
                    fallback_failure_message.unwrap_or(rollback_reason),
                )
                .summarize(self.scratch.semantic_delta.rollback.as_ref(), profile),
            });
        }
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::ReplayDetail,
        ) {
            self.scratch
                .semantic_delta
                .replay_events
                .push(TransactionReplayEntry {
                    kind: ReplayEventKind::TransactionRolledBack,
                    detail: rollback_reason.to_string(),
                    execution_record_id: None,
                    semantic_segment_id: None,
                });
            let detail = error
                .as_ref()
                .map(|err| err.to_string())
                .or_else(|| fallback_failure_message.map(str::to_owned))
                .unwrap_or_else(|| rollback_reason.to_string());
            self.scratch
                .semantic_delta
                .replay_events
                .push(TransactionReplayEntry {
                    kind: ReplayEventKind::FailureRecorded,
                    detail,
                    execution_record_id: None,
                    semantic_segment_id: None,
                });
        }
        if increment_poison_count {
            self.with_telemetry(|telemetry| telemetry.transaction.transaction_poison_count += 1);
        }
        match outcome {
            Ok(outcome) => Ok(self.finalize_semantic_delta(
                true,
                outcome,
                touched_nodes,
                commit_start.elapsed().as_nanos(),
            )),
            Err(err) => {
                let _ = self.finalize_semantic_delta(
                    true,
                    TransactionOutcome::RolledBack,
                    touched_nodes,
                    commit_start.elapsed().as_nanos(),
                );
                Err(err)
            }
        }
    }
}
