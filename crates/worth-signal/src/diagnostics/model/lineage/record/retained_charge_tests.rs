use super::*;
use crate::data::handle::NodeId;
use crate::data::retained_storage::{arc_allocation_charge, ordered_index_charge};
use crate::diagnostics::state::{DiagnosticHistory, DiagnosticHistoryEditDenial};
use crate::facade::{EvaluationContext, EvaluationRequestMode, SignalGraph};
use crate::logic::transaction::{
    BranchConflictResolutionPlan, BranchMergeDivergence, BranchMergeKind,
    BranchMergeReconciliationPolicy, BranchMergeResolutionRequirement, BranchMergeStrategy,
    ConflictResolutionRecord, ConflictResolutionStrategy,
};
use crate::runtime_policy::SignalRuntimePolicy;
use crate::state::SignalBranchId;
use crate::tests::support::version_ab;
use std::sync::Arc;

fn emitted_lineage() -> (LineageRecord, NodeId) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(7, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    let record = graph
        .diagnostics_state()
        .lineage_records()
        .iter()
        .find(|record| matches!(record.kind, LineageRecordKind::ArtifactTransition { .. }))
        .unwrap()
        .clone();
    (record, node)
}

// Capacity fixtures describe retained representation, not an executed merge.
fn merge_record(node: NodeId) -> (LineageRecord, u64) {
    let mut required_resolution = Vec::with_capacity(32);
    required_resolution.push(BranchMergeResolutionRequirement::ReconcileComparableState);
    let mut supported_strategies = Vec::with_capacity(64);
    supported_strategies.push(ConflictResolutionStrategy::AdoptSourceComparableState);
    let inner_bytes = required_resolution.capacity()
        * std::mem::size_of::<BranchMergeResolutionRequirement>()
        + supported_strategies.capacity() * std::mem::size_of::<ConflictResolutionStrategy>();
    let mut records = Vec::with_capacity(16);
    records.push(ConflictResolutionRecord {
        source_node: node,
        target_node: node,
        required_resolution,
        supported_strategies,
    });
    let outer_bytes = records.capacity() * std::mem::size_of::<ConflictResolutionRecord>();
    let mut source_name = String::with_capacity(128);
    source_name.push_str("source");
    let mut target_name = String::with_capacity(512);
    target_name.push_str("target");
    let expected =
        (inner_bytes + outer_bytes + source_name.capacity() + target_name.capacity()) as u64;
    let record = LineageRecord::new(
        17,
        SignalBranchId(0),
        LineageRecordKind::BranchMerge {
            source_branch_id: SignalBranchId(1),
            target_branch_id: SignalBranchId(0),
            merge_kind: BranchMergeKind::ConflictResolved,
            divergence: BranchMergeDivergence::SharedStateConflict,
            merge_strategy: BranchMergeStrategy::ReplaySourceDeltaOntoTarget,
            reconciliation_policy: BranchMergeReconciliationPolicy::built_in_default(),
            resolution_plan: Some(BranchConflictResolutionPlan {
                source_branch_id: SignalBranchId(1),
                target_branch_id: SignalBranchId(0),
                divergence: BranchMergeDivergence::SharedStateConflict,
                records,
            }),
            merged_snapshot_id: None,
            source_branch_display_name: source_name,
            target_branch_display_name: target_name,
        },
    );
    (record, expected)
}

#[test]
fn lineage_merge_charge_includes_outer_inner_vector_and_name_capacity() {
    let (_, node) = emitted_lineage();
    let (record, expected) = merge_record(node);
    let mut work = Work::new(1000);
    assert_eq!(
        record.retained_heap_charge(&mut work).unwrap().bytes(),
        expected
    );
    assert_eq!(
        record
            .retained_heap_charge(&mut Work::new(work.visits()))
            .unwrap()
            .bytes(),
        expected
    );
    assert_eq!(
        record.retained_heap_charge(&mut Work::new(work.visits() - 1)),
        Err(Denial::WorkExhausted {
            maximum_visits: work.visits() - 1
        })
    );
    let mut changed = record.clone();
    let before = changed.retained_heap_charge(&mut Work::new(1000)).unwrap();
    let LineageRecordKind::BranchMerge {
        resolution_plan: Some(plan),
        ..
    } = &mut changed.kind
    else {
        panic!("fixture must contain a plan")
    };
    let old_capacity = plan.records[0].supported_strategies.capacity();
    plan.records[0].supported_strategies.reserve_exact(1024);
    let delta = (plan.records[0].supported_strategies.capacity() - old_capacity)
        * std::mem::size_of::<ConflictResolutionStrategy>();
    assert_eq!(
        changed
            .retained_heap_charge(&mut Work::new(1000))
            .unwrap()
            .bytes()
            - before.bytes(),
        delta as u64
    );
    assert_eq!(changed, record);
}

#[test]
fn lineage_history_denial_preserves_roots_and_incoming_nested_allocations() {
    let (emitted, node) = emitted_lineage();
    let original: DiagnosticHistory<_> = [emitted].into_iter().collect();
    let old_charge = original
        .prepare_retained_charge(&mut Work::new(1000))
        .unwrap();
    let (candidate, heap) = merge_record(node);
    let exact = two_frame_charge(old_charge, heap);
    let mut admitted = original.clone();
    let mut work = Work::new(1000);
    admitted
        .append_with_retained_capacity(candidate, exact, &mut work)
        .unwrap();
    let visits = work.visits();
    for (limit, short_bytes) in [(visits, false), (visits, true), (visits - 1, false)] {
        let (candidate, heap) = merge_record(node);
        let required = two_frame_charge(old_charge, heap);
        let maximum = if short_bytes {
            required
                .checked_sub(Charge::capacity::<u8>(1).unwrap())
                .unwrap()
        } else {
            required
        };
        let incoming_records = merge_records_ptr(&candidate);
        let mut draft = original.clone();
        let result = draft.append_with_retained_capacity(candidate, maximum, &mut Work::new(limit));
        if limit == visits && !short_bytes {
            result.unwrap();
            assert_eq!(draft, admitted);
            assert_eq!(draft.prepared_retained_charge().unwrap(), required);
            continue;
        }
        let (returned, denial) = result.unwrap_err();
        assert_eq!(merge_records_ptr(&returned), incoming_records);
        assert_eq!(&returned, admitted.back().unwrap());
        assert_eq!(
            returned
                .retained_heap_charge(&mut Work::new(1000))
                .unwrap()
                .bytes(),
            heap
        );
        assert_eq!(draft, original);
        assert!(std::ptr::eq(
            draft.front().unwrap(),
            original.front().unwrap()
        ));
        assert_eq!(draft.prepared_retained_charge().unwrap(), old_charge);
        let expected = if short_bytes {
            DiagnosticHistoryEditDenial::CapacityExhausted { maximum, required }
        } else {
            DiagnosticHistoryEditDenial::Accounting(Denial::WorkExhausted {
                maximum_visits: limit,
            })
        };
        assert_eq!(denial, expected);
    }
    let removed = admitted
        .evict_front_with_retained_charge(&mut Work::new(1000))
        .unwrap()
        .unwrap();
    assert!(std::ptr::eq(
        Arc::as_ptr(&removed),
        original.front().unwrap()
    ));
    assert_eq!(original.len(), 1);
    assert_eq!(admitted.len(), 1);
    let survivor_charge = ordered_index_charge::<u64, Arc<LineageRecord>>(1)
        .unwrap()
        .checked_add(arc_allocation_charge::<LineageRecord>().unwrap())
        .unwrap()
        .checked_add(Charge::capacity::<u8>(usize::try_from(heap).unwrap()).unwrap())
        .unwrap();
    assert_eq!(
        admitted.prepared_retained_charge().unwrap(),
        survivor_charge
    );
}

fn merge_records_ptr(record: &LineageRecord) -> *const ConflictResolutionRecord {
    let LineageRecordKind::BranchMerge {
        resolution_plan: Some(plan),
        ..
    } = &record.kind
    else {
        panic!("fixture must contain a plan")
    };
    plan.records.as_ptr()
}
fn two_frame_charge(old: Charge, heap: u64) -> Charge {
    old.checked_sub(ordered_index_charge::<u64, Arc<LineageRecord>>(1).unwrap())
        .unwrap()
        .checked_add(ordered_index_charge::<u64, Arc<LineageRecord>>(2).unwrap())
        .unwrap()
        .checked_add(arc_allocation_charge::<LineageRecord>().unwrap())
        .unwrap()
        .checked_add(Charge::capacity::<u8>(usize::try_from(heap).unwrap()).unwrap())
        .unwrap()
}

#[path = "retained_variant_tests.rs"]
mod variants;
