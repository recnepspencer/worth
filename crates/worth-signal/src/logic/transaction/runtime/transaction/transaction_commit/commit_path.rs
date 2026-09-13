use crate::clock::RuntimeInstant;
use crate::data::error::SignalError;
use crate::diagnostics::epochs::{
    EventEpochOutcome, EventEpochSummary, EventSubscriberOutcome, EventSubscriberOutcomeKind,
};
use crate::diagnostics::replay::ReplayEventKind;
use crate::logic::events::EventFlushError;

use super::super::transaction_types::{
    SignalTransaction, StagedEventOperation, TransactionCommitPosture, TransactionOutcome,
    TransactionReplayEntry, TransactionResult,
};
use crate::diagnostics::ExecutionFailurePhase;
use crate::logic::transaction::runtime::transaction::ObservationBoundaryOutcome;
use worth_foundational::FoundationalBranchReferenceGeneration;

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn commit(mut self) -> Result<TransactionResult, SignalError> {
        let commit_start = RuntimeInstant::now();
        if self.finished {
            return Err(SignalError::transaction_finished());
        }
        self.finished = true;

        if self.poisoned {
            return self.fail_and_rollback(
                "poisoned transaction rollback",
                None,
                Some("transaction rolled back because it was poisoned"),
                ExecutionFailurePhase::Rollback,
                true,
                Ok(TransactionOutcome::Poisoned),
                commit_start,
            );
        }

        self.scratch.staged_patch_count = self.scratch.graph_patches.touched_count() as u64;

        if matches!(self.commit_posture, TransactionCommitPosture::BranchLocal)
            && !self.scratch.staged_event_operations.is_empty()
        {
            let err = SignalError::invalid_input(
                "branch-local transactions cannot publish event-bus operations before reconciliation",
            );
            return self.fail_and_rollback(
                "branch-local event publication denied",
                Some(err.clone()),
                None,
                ExecutionFailurePhase::CommitPromotion,
                false,
                Err(err),
                commit_start,
            );
        }

        let next_branch_generation =
            match FoundationalBranchReferenceGeneration::new(*self.branch_head_generation)
                .checked_advance()
                .map(FoundationalBranchReferenceGeneration::get)
            {
                Ok(generation) => generation,
                Err(denial) => {
                    let error = SignalError::internal(format!(
                        "Signal branch reference generation cannot advance: {denial:?}"
                    ));
                    return self.fail_and_rollback(
                        "branch reference generation exhausted",
                        Some(error.clone()),
                        None,
                        ExecutionFailurePhase::CommitPromotion,
                        false,
                        Err(error),
                        commit_start,
                    );
                }
            };

        if matches!(self.commit_posture, TransactionCommitPosture::Visible) {
            if let Err(err) = self
                .event_bus
                .begin(self.runtime_ctx)
                .map_err(|e| SignalError::invalid_input(format!("event bus begin failed: {e:?}")))
            {
                return self.fail_and_rollback(
                    "event bus begin failed",
                    Some(err.clone()),
                    None,
                    ExecutionFailurePhase::CommitPromotion,
                    true,
                    Err(err),
                    commit_start,
                );
            }

            let mut current_epoch_events = 0u32;
            let mut next_epoch_ordinal = 0u32;
            for operation in std::mem::take(&mut self.scratch.staged_event_operations) {
                match operation {
                    StagedEventOperation::Emit(event) => {
                        current_epoch_events += 1;
                        self.event_bus.emit(event)
                    }
                    StagedEventOperation::Flush(barrier) => {
                        let flush_start = RuntimeInstant::now();
                        let completed_subscribers = match self.event_bus.flush_with_capture(
                            barrier,
                            self.runtime_ctx,
                            self.captures_optional_telemetry(),
                        ) {
                            Ok(completed_subscribers) => completed_subscribers,
                            Err(flush_err) => {
                                self.scratch.staged_event_flush_nanos +=
                                    flush_start.elapsed().as_nanos();
                                self.scratch
                                    .semantic_delta
                                    .event_epochs
                                    .push(match &flush_err {
                                        EventFlushError::Registry(source) => EventEpochSummary {
                                            ordinal: next_epoch_ordinal,
                                            barrier,
                                            emitted_event_count: current_epoch_events,
                                            subscriber_count: self.event_bus.resolved_order().len()
                                                as u32,
                                            committed_subscriber_count: 0,
                                            failed_subscriber_position: None,
                                            subscriber_outcomes: Vec::new(),
                                            outcome: EventEpochOutcome::Failed,
                                            failure_subscriber: None,
                                            message: Some(format!(
                                            "subscriber registry invalid during flush: {source:?}"
                                        )),
                                        },
                                        EventFlushError::Subscriber {
                                            subscriber_name,
                                            completed_subscribers,
                                            failed_subscriber_requires,
                                            failed_subscriber_provides,
                                            failed_subscriber_staged,
                                            source,
                                            ..
                                        } => EventEpochSummary {
                                            ordinal: next_epoch_ordinal,
                                            barrier,
                                            emitted_event_count: current_epoch_events,
                                            subscriber_count: self.event_bus.resolved_order().len()
                                                as u32,
                                            committed_subscriber_count: completed_subscribers.len()
                                                as u32,
                                            failed_subscriber_position: Some(
                                                completed_subscribers.len() as u32 + 1,
                                            ),
                                            subscriber_outcomes: completed_subscribers
                                                .iter()
                                                .map(|subscriber| EventSubscriberOutcome {
                                                    subscriber_name: subscriber.name.to_string(),
                                                    outcome: EventSubscriberOutcomeKind::Committed,
                                                    requires_data_ids: subscriber
                                                        .requires_data_ids
                                                        .clone(),
                                                    provides_data_ids: subscriber
                                                        .provides_data_ids
                                                        .clone(),
                                                    staged_data_ids: subscriber
                                                        .staged_data_ids
                                                        .clone(),
                                                })
                                                .chain(std::iter::once(EventSubscriberOutcome {
                                                    subscriber_name: (*subscriber_name).to_string(),
                                                    outcome: EventSubscriberOutcomeKind::Failed,
                                                    requires_data_ids: failed_subscriber_requires
                                                        .clone(),
                                                    provides_data_ids: failed_subscriber_provides
                                                        .clone(),
                                                    staged_data_ids: failed_subscriber_staged
                                                        .clone(),
                                                }))
                                                .collect(),
                                            outcome: EventEpochOutcome::Failed,
                                            failure_subscriber: Some(
                                                (*subscriber_name).to_string(),
                                            ),
                                            message: Some(source.to_string()),
                                        },
                                    });
                                let err = match &flush_err {
                                    EventFlushError::Registry(source) => {
                                        SignalError::event_flush_failed(
                                            "registry",
                                            format!("{source:?}"),
                                        )
                                    }
                                    EventFlushError::Subscriber {
                                        subscriber_name,
                                        source,
                                        ..
                                    } => SignalError::event_flush_failed(
                                        (*subscriber_name).to_string(),
                                        source.to_string(),
                                    ),
                                };
                                return self.fail_and_rollback(
                                    "event bus flush failed",
                                    Some(err.clone()),
                                    None,
                                    ExecutionFailurePhase::CommitPromotion,
                                    true,
                                    Err(err),
                                    commit_start,
                                );
                            }
                        };
                        self.scratch.staged_event_flush_nanos += flush_start.elapsed().as_nanos();
                        self.scratch
                            .semantic_delta
                            .event_epochs
                            .push(EventEpochSummary {
                                ordinal: next_epoch_ordinal,
                                barrier,
                                emitted_event_count: current_epoch_events,
                                subscriber_count: completed_subscribers.len() as u32,
                                committed_subscriber_count: completed_subscribers.len() as u32,
                                failed_subscriber_position: None,
                                subscriber_outcomes: completed_subscribers
                                    .iter()
                                    .map(|subscriber| EventSubscriberOutcome {
                                        subscriber_name: subscriber.name.to_string(),
                                        outcome: EventSubscriberOutcomeKind::Committed,
                                        requires_data_ids: subscriber.requires_data_ids.clone(),
                                        provides_data_ids: subscriber.provides_data_ids.clone(),
                                        staged_data_ids: subscriber.staged_data_ids.clone(),
                                    })
                                    .collect(),
                                outcome: EventEpochOutcome::Committed,
                                failure_subscriber: None,
                                message: None,
                            });
                        current_epoch_events = 0;
                        next_epoch_ordinal += 1;
                    }
                }
            }
        }

        while let Some(domain) = self.scratch.staged_dirty.first_dirty_domain() {
            if let Some(impact) = self.scratch.staged_dirty.take_domain_impact(domain) {
                self.checkpoint
                    .dirty_mut()
                    .merge_domain_impact(domain, impact);
            }
        }
        if self.captures_optional_telemetry() {
            self.checkpoint
                .telemetry_mut()
                .checkpoint
                .checkpoint_flushes += self.scratch.staged_checkpoint_flushes;
            self.checkpoint
                .telemetry_mut()
                .checkpoint
                .checkpoint_flush_nanos += self.scratch.staged_checkpoint_flush_nanos;
        }
        let touched_nodes = self.scratch.graph_patches.touched_nodes(self.graph);
        let touched_node_count = touched_nodes.len() as u32;
        for ((family_id, key_id, memo_key_id), result) in
            std::mem::take(&mut self.scratch.staged_memo_writes)
        {
            let family = self.config.key_registry.family(family_id).clone();
            let key = self.config.key_registry.keys[key_id.index()].clone();
            let memo_key = self.config.key_registry.memo_key(memo_key_id).clone();
            self.config
                .store_memoized_result(&family, &key, &memo_key, result);
        }
        self.scratch.graph_patches.commit_and_clear();
        self.branch_mutation_ledger
            .absorb_records(self.graph.pending_branch_mutation_records());
        *self.branch_head_generation = next_branch_generation;
        *self.branch_restore_snapshot_id = None;
        self.graph.clear_branch_mutation_nodes();
        let staged_patch_count = self.scratch.staged_patch_count;
        self.with_telemetry(|telemetry| {
            telemetry.transaction.transaction_commit_count += 1;
            telemetry.transaction.staged_node_patch_count += staged_patch_count;
            telemetry.transaction.max_touched_nodes_in_txn = telemetry
                .transaction
                .max_touched_nodes_in_txn
                .max(staged_patch_count);
        });
        for node in touched_nodes.as_slice().iter().copied() {
            let _ = self.graph.record_operational_diagnostic_facts(node, None);
        }
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::ReplayDetail,
        ) {
            self.scratch
                .semantic_delta
                .replay_events
                .push(TransactionReplayEntry {
                    kind: ReplayEventKind::TransactionCommitted,
                    detail: "transaction committed".to_string(),
                    execution_record_id: None,
                    semantic_segment_id: None,
                });
        }
        let observation_outcome = match self.commit_posture {
            TransactionCommitPosture::Visible => ObservationBoundaryOutcome::Delivered,
            TransactionCommitPosture::BranchLocal => {
                ObservationBoundaryOutcome::BranchLocalSuppressed
            }
        };
        let (deliveries, observation) = self
            .scratch
            .observations
            .drain_delivery_boundary(observation_outcome);
        let delivered_observation_count =
            if matches!(self.commit_posture, TransactionCommitPosture::Visible) {
                self.observations.deliver_committed(self.graph, &deliveries) as u64
            } else {
                self.with_telemetry(|telemetry| {
                    telemetry
                        .transaction
                        .branch_local_suppressed_observation_count +=
                        u64::from(observation.branch_local_suppressed_event_count);
                });
                0
            };
        self.with_telemetry(|telemetry| {
            telemetry.transaction.delivered_observation_count += delivered_observation_count;
        });
        self.scratch.semantic_delta.observation = observation;
        Ok(self.finalize_semantic_delta(
            false,
            TransactionOutcome::Committed,
            touched_node_count,
            commit_start.elapsed().as_nanos(),
        ))
    }
}
