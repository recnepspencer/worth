use super::*;
use crate::data::graph::SignalGraph;
use crate::diagnostics::lineage::InvalidationCause;
use crate::runtime_policy::SignalConditionalEvaluationBudget;
use crate::state::SignalBranchId;

fn ledger(bytes: u64) -> Arc<SignalConditionalRetentionLedger> {
    SignalConditionalRetentionLedger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 2,
            maximum_retained_bytes: bytes,
            maximum_attempt_visits: 100_000,
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    )
}
fn record(node: NodeId, sequence: u64) -> LineageRecord {
    LineageRecord::invalidation(
        sequence,
        SignalBranchId(0),
        node,
        LineageArtifactId(sequence % 3),
        InvalidationCause::SourceAspectChanged { aspect_index: 0 },
    )
}
fn assert_charges(state: &DiagnosticsState) {
    assert_eq!(
        state.lineage_records.prepared_retained_charge().unwrap(),
        state
            .lineage_records
            .retained_heap_charge(&mut Work::new(8_000_000))
            .unwrap()
    );
    assert_eq!(
        state
            .lineage_records_by_node
            .prepared_retained_charge()
            .unwrap(),
        state
            .lineage_records_by_node
            .retained_heap_charge(&mut Work::new(8_000_000))
            .unwrap()
    );
    assert_eq!(
        state
            .lineage_records_by_artifact
            .prepared_retained_charge()
            .unwrap(),
        state
            .lineage_records_by_artifact
            .retained_heap_charge(&mut Work::new(8_000_000))
            .unwrap()
    );
}
#[test]
fn retained_lineage_publication_matches_native_order_and_eviction_across_indexes() {
    let mut graph = SignalGraph::new();
    let nodes = [graph.create_node(), graph.create_node()];
    let ledger = ledger(64 * 1024 * 1024);
    let mut state = DiagnosticsState::default();
    state.installed_retention_budget.history_limit =
        crate::diagnostics::policy::HistoryLimit::new(1);
    let mut ordinary = state.clone();
    for sequence in 0..72 {
        let record = record(nodes[sequence as usize % 2], sequence);
        ordinary.record_lineage_record(record.clone());
        state
            .record_retained_lineage(record, &ledger, &mut Work::new(8_000_000))
            .unwrap();
        assert_eq!(state, ordinary);
        assert_charges(&state);
    }
    assert_eq!(state.lineage_records.len(), 32);
    let fork = state.fork_persistent();
    drop(state);
    assert!(ledger.usage().1 > 0);
    drop(fork);
    assert_eq!(ledger.usage(), (0, 0));
}
#[test]
fn retained_lineage_publication_denies_before_mutating_any_canonical_root() {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let mut state = DiagnosticsState::default();
    let before = state.clone();
    let empty = ledger(0);
    assert!(matches!(
        state.record_retained_lineage(record(node, 0), &empty, &mut Work::new(8_000_000)),
        Err(Denial::Retention(
            SignalConditionalRetentionDenial::CapacityExhausted
        ))
    ));
    assert_eq!(state, before);
    assert_eq!(empty.usage(), (0, 0));
    let ledger = ledger(64 * 1024 * 1024);
    state
        .record_retained_lineage(record(node, 0), &ledger, &mut Work::new(8_000_000))
        .unwrap();
    let before = state.clone();
    let usage = ledger.usage();
    assert!(matches!(
        state.record_retained_lineage(record(node, 1), &ledger, &mut Work::new(0)),
        Err(Denial::Accounting(
            RetainedStoragePreparationDenial::WorkExhausted { .. }
        ))
    ));
    assert_eq!(state, before);
    assert_eq!(ledger.usage(), usage);
    ledger.close();
    assert!(matches!(
        state.record_retained_lineage(record(node, 1), &ledger, &mut Work::new(8_000_000)),
        Err(Denial::Retention(SignalConditionalRetentionDenial::Closed))
    ));
    assert_eq!(state, before);
    assert_eq!(ledger.usage(), usage);
}
#[test]
fn retained_lineage_publication_never_repairs_unprepared_ordinary_history() {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let mut state = DiagnosticsState::default();
    state.record_lineage_record(record(node, 0));
    let before = state.clone();
    let ledger = ledger(64 * 1024 * 1024);
    assert!(matches!(
        state.record_retained_lineage(record(node, 1), &ledger, &mut Work::new(8_000_000)),
        Err(Denial::Unprepared)
    ));
    assert_eq!(state, before);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn heap_bearing_lineage_publication_uses_one_exact_work_allowance() {
    use crate::diagnostics::lineage::LineageRecordKind;
    use crate::logic::transaction::{
        ArtifactMergeAction, BranchMergeConflictKind, BranchMergeDivergence, BranchMergeKind,
        BranchMergeReconciliationPolicy, BranchMergeStrategy, MergeDecisionBasis,
    };
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let record = LineageRecord {
        sequence: 0,
        emitted_on_branch_id: SignalBranchId(0),
        kind: LineageRecordKind::ArtifactMerge {
            source_node: node,
            target_node: Some(node),
            source_branch_id: SignalBranchId(1),
            target_branch_id: SignalBranchId(0),
            source_artifact_id: None,
            target_artifact_id_before: None,
            target_artifact_id_after: Some(LineageArtifactId(0)),
            merge_action: ArtifactMergeAction::Adopted,
            decision_basis: MergeDecisionBasis::SourceAuthorityAdopted,
            merge_kind: BranchMergeKind::Applied,
            divergence: BranchMergeDivergence::None,
            merge_strategy: BranchMergeStrategy::AdoptSourceHead,
            reconciliation_policy: BranchMergeReconciliationPolicy::built_in_default(),
            resolved_conflict_kinds: vec![BranchMergeConflictKind::RuntimeArtifactMismatch; 4096],
        },
    };
    let mut sample = DiagnosticsState::default();
    let mut measured = Work::new(8_000_000);
    sample
        .record_retained_lineage(record.clone(), &ledger(64 * 1024 * 1024), &mut measured)
        .unwrap();
    let exact = measured.visits();
    for maximum in [exact - 1, exact] {
        let mut state = DiagnosticsState::default();
        let before = state.clone();
        let mut work = Work::new(maximum);
        let result =
            state.record_retained_lineage(record.clone(), &ledger(64 * 1024 * 1024), &mut work);
        if maximum == exact {
            result.unwrap();
            assert_eq!(state, sample);
            assert_charges(&state);
        } else {
            assert!(matches!(
                result,
                Err(Denial::Accounting(
                    RetainedStoragePreparationDenial::WorkExhausted { .. }
                ))
            ));
            assert_eq!(state, before);
        }
    }
}
