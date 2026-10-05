//! Canonical preparation and publication of one admitted graph apply epoch.
use super::epoch_nodes::{EpochProducerNodeChange, PreparedEpochNodeEdits};
use super::epoch_snapshots::PreparedEpochSnapshotStore;
use super::snapshot_preparation::PreparedSnapshotCandidate;
use super::{
    AppliedEffectReport, ComparatorPolicyResolver, EvaluationWork, OutputCommitPreparationSeam,
    PreparedParallelApplyCommitPacket, SignalError, SignalGraph,
};
use crate::data::graph::runtime::graph::PreparedEpochCausePublication;
use crate::data::graph::{
    EpochSemanticSeed, PreparedDependencyTopologyEpoch, PreparedDependencyTopologyStorage,
};
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::OutputCommitOrdinal;
use crate::data::proof::invalidation::output_commit::{
    CommittedProducedAspectDelta, ProducedAspectDelta,
};
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};
use crate::logic::evaluation::PendingDependencySnapshot;
use crate::logic::invalidation::causality::EpochCauseHead;
use worth_execution::{ChargedBytes, ExecutionResourceLease, MapKernelContext};

fn with_request_evaluation<R>(
    request_work: Option<&mut MapKernelContext<'_, '_>>,
    operation: impl FnOnce(&mut EvaluationWork<'_, '_>) -> Result<R, SignalError>,
) -> Result<R, SignalError> {
    match request_work {
        Some(request) => {
            let mut checkpoint = |units: usize| {
                let units = u64::try_from(units)
                    .map_err(|_| SignalError::invalid_input("epoch work size overflow"))?;
                request
                    .checkpoint(units)
                    .map_err(|_| SignalError::invalid_input("epoch preparation work stopped"))
            };
            operation(&mut EvaluationWork::RequestCheckpoint(&mut checkpoint))
        }
        None => operation(&mut EvaluationWork::Ordinary),
    }
}

fn with_epoch_retained_work<R>(
    work: &mut Work<'_>,
    request_work: Option<&mut MapKernelContext<'_, '_>>,
    operation: impl FnOnce(&mut Work<'_>) -> Result<R, SignalError>,
) -> Result<R, SignalError> {
    let Some(request) = request_work else {
        return operation(work);
    };
    let maximum_visits = work.maximum_visits();
    let mut checkpoint = |units: usize| {
        let units = u64::try_from(units)
            .map_err(|_| RetainedStoragePreparationDenial::WorkExhausted { maximum_visits })?;
        request
            .checkpoint(units)
            .map_err(|_| RetainedStoragePreparationDenial::WorkExhausted { maximum_visits })
    };
    let mut observed = work.reborrow_with_checkpoint(&mut checkpoint);
    operation(&mut observed)
}

pub(crate) struct PreparedEpochPublication {
    topology: PreparedDependencyTopologyStorage,
    nodes: PreparedEpochNodeEdits,
    causes: PreparedEpochCausePublication,
    snapshots: Option<PreparedEpochSnapshotStore>,
    diagnostics: crate::diagnostics::state::PreparedEpochDiagnostics,
    suppressed_downstream: u64,
}

impl SignalGraph {
    pub(crate) fn epoch_publication_shape_bound(width: usize) -> Result<u64, SignalError> {
        use crate::data::dependency::{
            DependencySnapshotId, DependencySnapshotShapeStore, DependencySnapshotStore,
            SnapshotDeltaRecord, SnapshotStorageStrategy,
        };
        use crate::data::retained_storage::RetainedStorageCharge as Charge;
        Charge::capacity::<super::OutputCommitHead>(width)
            .and_then(|charge| charge.checked_add(Charge::capacity::<ProducedAspectDelta>(width)?))
            .and_then(|charge| charge.checked_add(Charge::capacity::<EpochCauseHead>(width)?))
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<PreparedSnapshotCandidate>(width)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<Option<EpochProducerNodeChange>>(width)?)
            })
            .and_then(|charge| charge.checked_add(Charge::capacity::<EpochSemanticSeed>(width)?))
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<(
                    NodeId,
                    DependencySnapshotId,
                    SnapshotDeltaRecord,
                    SnapshotStorageStrategy,
                )>(width)?)
            })
            .and_then(|charge| charge.checked_add(Charge::capacity::<DependencySnapshotStore>(1)?))
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<DependencySnapshotShapeStore>(1)?)
            })
            .and_then(|charge| charge.checked_add(Charge::capacity::<PreparedEpochPublication>(1)?))
            .map(Charge::bytes)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    pub(crate) fn prepare_parallel_apply_epoch(
        &mut self,
        topology: PreparedDependencyTopologyEpoch,
        packets: Vec<PreparedParallelApplyCommitPacket>,
        semantic_seeds: Vec<EpochSemanticSeed>,
        comparator: &mut impl ComparatorPolicyResolver,
        lease: Option<&ExecutionResourceLease<'_>>,
        candidates: crate::data::graph::PreparedCandidateEpoch<'_>,
        mut request_work: Option<&mut MapKernelContext<'_, '_>>,
        mut preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedEpochPublication, SignalError> {
        if let Some(budget) = preparation.as_deref_mut() {
            let width = packets.len();
            budget.claim_vec::<super::OutputCommitHead>(width)?;
            budget.claim_vec::<ProducedAspectDelta>(width)?;
            budget.claim_vec::<EpochCauseHead>(width)?;
            budget.claim_vec::<PreparedSnapshotCandidate>(width)?;
            budget.claim_vec::<Option<EpochProducerNodeChange>>(width)?;
        }
        if semantic_seeds.len() != packets.len() {
            return Err(SignalError::internal(
                "epoch semantic seeds do not align with packets",
            ));
        }
        let width = packets.len();
        let mut ordinal = self.cause_sets.reserve_output_commit_ordinal().0;
        let mut heads = Vec::with_capacity(packets.len());
        let mut deltas = Vec::<ProducedAspectDelta>::with_capacity(width);
        let mut cause_heads = Vec::with_capacity(width);
        let mut snapshots = Vec::<PreparedSnapshotCandidate>::with_capacity(width);
        for packet in packets {
            if let Some(budget) = preparation.as_deref_mut() {
                budget.claim(packet.additional_charged_bytes())?;
                self.claim_epoch_output_head_shape(&packet.0, budget)?;
            }
            let head = with_request_evaluation(request_work.as_deref_mut(), |work| {
                self.prepare_output_commit_head_at_ordinal(
                    packet.0,
                    comparator,
                    OutputCommitOrdinal(ordinal),
                    &mut |_: OutputCommitPreparationSeam| Ok(()),
                    work,
                )
            })?;
            #[cfg(test)]
            self.check_epoch_output_preparation_fault()?;
            let delta_ordinal = head
                .produced_delta
                .as_ref()
                .map(|delta| delta.output_commit_ordinal);
            if let Some(delta) = head.produced_delta.as_ref() {
                self.claim_epoch_delta_copy(
                    delta,
                    preparation.as_deref_mut(),
                    request_work.as_deref_mut(),
                )?;
                ordinal = ordinal
                    .checked_add(1)
                    .ok_or_else(|| SignalError::internal("output commit ordinal overflow"))?;
                deltas.push(delta.clone());
            }
            cause_heads.push(EpochCauseHead {
                producer: head.apply.effect.operational.node,
                clean: super::super::vocabulary::verdict_transitions_clean(
                    &head.apply.effect.operational.verdict,
                ),
                delta_ordinal,
            });
            self.claim_epoch_snapshot_candidate_shape(&head.apply, preparation.as_deref_mut())?;
            if let Some(snapshot) = with_request_evaluation(request_work.as_deref_mut(), |work| {
                self.prepare_effect_snapshot_candidate(&head.apply, work)
            })? {
                snapshots.push(snapshot);
            }
            heads.push(head);
        }
        let mut candidate_queries = candidates.run(
            self,
            &deltas,
            request_work
                .as_deref_mut()
                .expect("checked epoch carries request work"),
        )?;
        let admission = self.prepare_epoch_direct_output_causes(
            &deltas,
            comparator,
            &mut EvaluationWork::Ordinary,
            lease,
            Some(&mut candidate_queries),
            request_work.as_deref_mut(),
            preparation.as_deref_mut(),
        )?;
        // A graph epoch is not a selected-definition conditional attempt.
        // Its observed preparation shares the actual request work ceiling.
        // Ordinary execution retains its existing per-node serial contract.
        let mut retained_work = Work::new(usize::MAX);
        let direct =
            with_epoch_retained_work(&mut retained_work, request_work.as_deref_mut(), |work| {
                self.prepare_epoch_direct_cause_publication(
                    admission,
                    &cause_heads,
                    &topology,
                    work,
                    preparation.as_deref_mut(),
                )
            })?;
        let suppressed_downstream = direct.suppressed_downstream_count();
        let (direct_nodes, direct_stores) = direct.split_node_changes();
        let snapshot_store =
            with_epoch_retained_work(&mut retained_work, request_work.as_deref_mut(), |work| {
                self.prepare_epoch_snapshot_store(snapshots, preparation.as_deref_mut(), work)
            })?;
        let mut producers = Vec::with_capacity(heads.len());
        for (order, (head, semantic_seed)) in heads.into_iter().zip(semantic_seeds).enumerate() {
            let write = with_request_evaluation(request_work.as_deref_mut(), |work| {
                self.prepare_effect_artifact_write(head.artifact_write, work)
            })?;
            let state = self.prepare_effect_node_state(&head.apply.effect, &write)?;
            producers.push(Some(EpochProducerNodeChange {
                order,
                semantic_seed,
                snapshot: snapshot_store
                    .as_ref()
                    .and_then(|store| store.snapshot_id(head.apply.effect.operational.node)),
                apply: head.apply,
                state,
                write,
                delta: head.produced_delta,
            }));
        }
        let cause_store =
            with_epoch_retained_work(&mut retained_work, request_work.as_deref_mut(), |work| {
                direct_stores.prepare_store(self, work, preparation.as_deref_mut())
            })?;
        let (topology_storage, topology_nodes, _topology_waiters) =
            topology.into_publication_parts();
        let ledger = self.arena.retained_node_ledger.clone();
        let mut diagnostics =
            with_epoch_retained_work(&mut retained_work, request_work.as_deref_mut(), |work| {
                self.diagnostics_state_mut().prepare_epoch_diagnostics(
                    work,
                    preparation.as_deref_mut(),
                    ledger.as_ref(),
                    producers.len(),
                )
            })?;
        let node_edits = with_epoch_retained_work(&mut retained_work, request_work, |work| {
            self.prepare_epoch_node_edits(
                topology_nodes,
                producers,
                direct_nodes,
                work,
                preparation,
                |payload, producer, semantic_work, mut budget| {
                    let seed = producer
                        .semantic_seed
                        .take()
                        .expect("one semantic seed per producer");
                    let prepared = self.prepare_epoch_semantic_node(
                        payload,
                        seed,
                        diagnostics.state_mut(),
                        semantic_work,
                        budget.as_deref_mut(),
                    )?;
                    if let Some(lineage) = prepared.lineage {
                        diagnostics.push_lineage(lineage, semantic_work, budget.as_deref_mut())?;
                    }
                    diagnostics.record_facts(
                        prepared.explanation,
                        prepared.provenance,
                        semantic_work,
                        budget.as_deref_mut(),
                    )?;
                    producer.semantic_artifacts = Some(prepared.artifacts);
                    Ok(())
                },
            )
        })?;
        if !node_edits.preconditions_hold(self) {
            return Err(SignalError::internal(
                "epoch node roots changed during preparation",
            ));
        }
        Ok(PreparedEpochPublication {
            topology: topology_storage,
            nodes: node_edits,
            causes: cause_store,
            snapshots: snapshot_store,
            diagnostics,
            suppressed_downstream,
        })
    }
}

impl PreparedEpochPublication {
    pub(crate) fn publish(
        self,
        graph: &mut SignalGraph,
    ) -> Vec<(
        AppliedEffectReport,
        Option<PendingDependencySnapshot>,
        crate::data::graph::PreparedSemanticArtifactImage,
    )> {
        assert!(
            self.nodes.preconditions_hold(graph),
            "prepared epoch roots remain current"
        );
        self.topology.publish(graph);
        let mut published = self.nodes.publish(graph);
        published.sort_by_key(|producer| producer.order);
        self.causes.publish(graph);
        if let Some(snapshots) = self.snapshots {
            snapshots.publish(graph);
        }
        self.diagnostics.publish(graph.diagnostics_state_mut());
        let mut outcomes = Vec::with_capacity(published.len());
        let receipt = super::OutputCommitPublicationReceipt::after_atomic_publication();
        for producer in published {
            let super::ApplyCommitPacket {
                effect,
                comparison,
                pending_snapshot,
                ..
            } = producer.apply;
            graph.record_output_node_publication(
                effect.operational.node,
                producer.mutation,
                producer.causality_changed,
                producer.runtime_write,
            );
            let performed = producer
                .delta
                .map(|delta| CommittedProducedAspectDelta::after_publication(delta, &receipt));
            graph.record_effect_telemetry(
                performed.as_ref(),
                &effect,
                &comparison,
                self.suppressed_downstream,
            );
            outcomes.push((
                AppliedEffectReport {
                    verdict: effect.operational.verdict,
                    comparison,
                    suppressed_downstream: self.suppressed_downstream,
                    temporal_eligibility: None,
                },
                pending_snapshot,
                producer
                    .semantic_artifacts
                    .expect("prepared epoch semantic artifacts"),
            ));
        }
        outcomes
    }
}
