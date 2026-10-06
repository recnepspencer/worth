//! Real allocator evidence for the selected diagnostics epoch request bound.
use super::measure;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger,
};
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::lineage::{InvalidationCause, LineageArtifactId, LineageRecord};
use crate::diagnostics::runtime::state::DiagnosticsState;
use crate::runtime_policy::{
    compile_signal_runtime_policy, SignalConditionalEvaluationBudget, SignalRuntimePolicy,
    SignalRuntimePolicyRequest,
};
use crate::state::SignalBranchId;

fn record(node: NodeId, artifact: LineageArtifactId, sequence: u64) -> LineageRecord {
    LineageRecord::invalidation(
        sequence,
        SignalBranchId(0),
        node,
        artifact,
        InvalidationCause::SourceAspectChanged { aspect_index: 0 },
    )
}

fn policy(history_limit: usize) -> crate::runtime_policy::InstalledSignalRuntimePolicy {
    compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(
        SignalRuntimePolicy::forensic().with_history_limit(history_limit),
    ))
    .unwrap()
}

fn seed(
    history: usize,
    distinct_keys: usize,
    history_limit: usize,
) -> (SignalGraph, DiagnosticsState, NodeId, LineageArtifactId) {
    let mut graph = SignalGraph::new();
    let target = graph.create_node();
    let mut state = DiagnosticsState::default();
    state.set_installed_policy(policy(history_limit));
    let target_artifact = state.allocate_lineage_artifact_id();
    let mut keys = vec![(target, target_artifact)];
    for _ in 1..distinct_keys.max(1) {
        keys.push((graph.create_node(), state.allocate_lineage_artifact_id()));
    }
    for ordinal in 0..history {
        let (node, artifact) = keys[ordinal % keys.len()];
        let sequence = state.allocate_lineage_sequence();
        state.record_lineage_record(record(node, artifact, sequence));
    }
    for &(node, _) in keys.iter().take(distinct_keys) {
        let explanation = graph.observe().explain(node).unwrap();
        state.record_explanation_fact(ExplanationFact::from_explanation(&explanation));
        state.record_provenance_fact(ProvenanceFact::from_explanation(&explanation));
    }
    (graph, state, target, target_artifact)
}

fn assert_epoch_fits_forecast(
    graph: &SignalGraph,
    state: &mut DiagnosticsState,
    target: NodeId,
    artifact: LineageArtifactId,
) {
    let explanation = graph.observe().explain(target).unwrap();
    let explanation = ExplanationFact::from_explanation(&explanation);
    let provenance = ProvenanceFact::from_explanation(&graph.observe().explain(target).unwrap());
    let sequence = state.allocate_lineage_sequence();
    let lineage = record(target, artifact, sequence);
    let mut measurement = Work::new(usize::MAX);
    let lineage_payload = lineage
        .retained_heap_charge(&mut measurement)
        .unwrap()
        .checked_add(arc_allocation_charge::<LineageRecord>().unwrap())
        .unwrap()
        .checked_mul(3)
        .unwrap()
        .bytes();
    let fact_replacement = state
        .epoch_fact_replacement_capacity_bound(target, &mut measurement)
        .unwrap();
    let fact_payload = explanation
        .retained_heap_charge(&mut measurement)
        .unwrap()
        .checked_add(provenance.retained_heap_charge(&mut measurement).unwrap())
        .unwrap()
        .checked_mul(2)
        .unwrap()
        .bytes();
    let forecast = state
        .epoch_preparation_capacity_bound(1)
        .unwrap()
        .checked_add(lineage_payload)
        .and_then(|total| total.checked_add(fact_replacement))
        .and_then(|total| total.checked_add(fact_payload))
        .unwrap();
    let ledger = SignalConditionalRetentionLedger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 2,
            maximum_retained_bytes: 512 * 1024 * 1024,
            maximum_attempt_visits: usize::MAX,
        },
        SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    let (_, peak) = measure(|| {
        // This witness isolates allocation; separate tests enforce visit denials.
        let mut work = Work::new(usize::MAX);
        let mut request = SignalPreparationBudget::new(forecast);
        let mut draft = state
            .prepare_epoch_diagnostics(&mut work, Some(&mut request), Some(&ledger), 1)
            .unwrap();
        draft
            .push_lineage(lineage, &mut work, Some(&mut request))
            .unwrap();
        draft
            .record_facts(
                Some(explanation),
                Some(provenance),
                &mut work,
                Some(&mut request),
            )
            .unwrap();
        draft.publish(state);
    });
    assert!(
        peak.unwrap() as u64 <= forecast,
        "diagnostics epoch allocated {} bytes against forecast {forecast}",
        peak.unwrap()
    );
    assert!(
        ledger.usage().1 > 0,
        "funded retained path was not exercised"
    );
    assert_eq!(
        state
            .lineage_records_for_node(target)
            .and_then(|records| records.back())
            .map(|record| record.sequence),
        Some(sequence),
    );
    assert_eq!(
        state.lineage_records().back().map(|record| record.sequence),
        Some(sequence)
    );
    assert_eq!(
        state
            .lineage_records_for_artifact(artifact)
            .and_then(|records| records.back())
            .map(|record| record.sequence),
        Some(sequence),
    );
    assert!(state.explanation_facts().get(&target).is_some());
    assert!(state.provenance_facts().get(&target).is_some());
}

#[test]
fn diagnostics_epoch_selected_growth_covers_real_allocations() {
    // Old payload is shared. Only the selected paths and new records may grow.
    for (history, keys, limit) in [(0, 0, 64), (1_000, 1, 64), (64, 64, 64), (1_000, 1_000, 64)] {
        let (graph, mut state, target, artifact) = seed(history, keys, limit);
        if keys == 1_000 {
            state.set_installed_policy(policy(1));
            assert_eq!(state.lineage_records().len(), 1_000);
        }
        assert_epoch_fits_forecast(&graph, &mut state, target, artifact);
        if keys == 1_000 {
            // The lineage writer enforces the newly installed limit in the measured epoch.
            assert_eq!(state.lineage_records().len(), 32);
        }
    }
}

#[test]
fn diagnostics_epoch_forked_overlay_remains_bounded_after_retirement_and_readmission() {
    let (mut graph, mut source, target, artifact) = seed(96, 64, 64);
    let mut fork = source.fork_persistent();
    assert!(fork.lineage_records_for_node(target).is_some());
    let other = graph.create_node();
    let other_artifact = fork.allocate_lineage_artifact_id();
    fork.set_installed_policy(policy(1));
    for _ in 0..32 {
        let sequence = fork.allocate_lineage_sequence();
        fork.record_lineage_record(record(other, other_artifact, sequence));
    }
    assert!(fork.lineage_records_for_node(target).is_none());
    assert_epoch_fits_forecast(&graph, &mut fork, target, artifact);
    assert!(source.lineage_records_for_node(target).is_some());
    drop(source);
    assert_epoch_fits_forecast(&graph, &mut fork, target, artifact);
}
