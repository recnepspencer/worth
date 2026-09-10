use super::*;
use crate::data::retained_storage::{
    arc_allocation_charge, btree_structure_charge, ordered_index_charge,
};
use crate::data::reuse::{ArtifactFamilyId, ReuseOrigin};
use crate::diagnostics::state::{DiagnosticHistory, DiagnosticHistoryEditDenial};
use crate::facade::{EvaluationContext, EvaluationRequestMode, SignalGraph};
use crate::runtime_policy::SignalRuntimePolicy;
use crate::tests::support::version_ab;
use std::sync::Arc;

fn recorded_summary() -> ExecutionHistorySummary {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(7, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    let summary = graph
        .diagnostics_state()
        .recent_history()
        .back()
        .unwrap()
        .clone();
    assert_eq!(summary.nodes.len(), 1);
    assert_eq!(summary.execution_record_count, 1);
    summary
}

#[test]
fn recorded_history_charge_tracks_capacity_without_double_charging_inline_summary() {
    let mut small = recorded_summary();
    // Retention representation fixture; these annotations do not claim execution.
    small.nodes[0].causality_kind = Some(String::from("cause"));
    let small_family = String::from("family");
    let small_family_capacity = small_family.capacity();
    small.nodes[0]
        .reuse_basis
        .as_mut()
        .unwrap()
        .artifact_family_basis = Some(ArtifactFamilyId::new(small_family));
    let mut large = small.clone();
    let original_vector_capacity = small.nodes.capacity();
    large.nodes.reserve_exact(127);
    let vector_delta = (large.nodes.capacity() - original_vector_capacity)
        * std::mem::size_of::<ExecutionHistoryNodeSummary>();
    let cause = large.nodes[0].causality_kind.as_mut().unwrap();
    let original_cause_capacity = small.nodes[0].causality_kind.as_ref().unwrap().capacity();
    cause.reserve_exact(8192);
    let cause_delta = cause.capacity() - original_cause_capacity;
    let mut family = String::from("family");
    let original_family_capacity = small_family_capacity;
    family.reserve_exact(4096);
    let family_delta = family.capacity() - original_family_capacity;
    large.nodes[0]
        .reuse_basis
        .as_mut()
        .unwrap()
        .artifact_family_basis = Some(ArtifactFamilyId::new(family));
    assert_eq!(small, large);
    let small_charge = small.retained_heap_charge(&mut Work::new(1000)).unwrap();
    let large_charge = large.retained_heap_charge(&mut Work::new(1000)).unwrap();
    assert_eq!(
        large_charge.bytes() - small_charge.bytes(),
        (vector_delta + cause_delta + family_delta) as u64
    );
    let expected = btree_structure_charge::<ReuseOrigin, u32>(small.reuse_origin_counts.len())
        .unwrap()
        .bytes()
        + (small.nodes.capacity() * std::mem::size_of::<ExecutionHistoryNodeSummary>()) as u64
        + small.nodes[0].causality_kind.as_ref().unwrap().capacity() as u64
        + small_family_capacity as u64;
    assert_eq!(small_charge.bytes(), expected);
}

#[test]
fn recorded_summary_append_admits_exact_bytes_and_work_then_evicts_with_live_custody() {
    let summary = recorded_summary();
    let original: DiagnosticHistory<_> = [summary.clone()].into_iter().collect();
    let old_charge = original
        .prepare_retained_charge(&mut Work::new(1000))
        .unwrap();
    let incoming = summary.clone();
    let incoming_heap = incoming.retained_heap_charge(&mut Work::new(1000)).unwrap();
    let exact = two_frame_charge(old_charge, &incoming);
    let mut admitted = original.clone();
    let mut work = Work::new(1000);
    admitted
        .append_with_retained_capacity(incoming, exact, &mut work)
        .unwrap();
    let visits = work.visits();
    let mut exact_work = original.clone();
    let incoming = summary.clone();
    let exact_candidate = two_frame_charge(old_charge, &incoming);
    exact_work
        .append_with_retained_capacity(incoming, exact_candidate, &mut Work::new(visits))
        .unwrap();
    assert_eq!(admitted, exact_work);
    assert_eq!(admitted.prepared_retained_charge().unwrap(), exact);
    for limit in [1000, visits - 1] {
        let mut rejected = original.clone();
        let incoming = summary.clone();
        let required = two_frame_charge(old_charge, &incoming);
        let maximum = if limit == 1000 {
            required
                .checked_sub(Charge::capacity::<u8>(1).unwrap())
                .unwrap()
        } else {
            required
        };
        let node_allocation = incoming.nodes.as_ptr();
        let (returned, denial) = rejected
            .append_with_retained_capacity(incoming, maximum, &mut Work::new(limit))
            .unwrap_err();
        assert_eq!(returned, summary);
        assert_eq!(returned.nodes.as_ptr(), node_allocation);
        assert_eq!(rejected, original);
        assert!(std::ptr::eq(
            rejected.front().unwrap(),
            original.front().unwrap()
        ));
        assert_eq!(rejected.prepared_retained_charge().unwrap(), old_charge);
        let expected_denial = if limit == 1000 {
            DiagnosticHistoryEditDenial::CapacityExhausted { maximum, required }
        } else {
            DiagnosticHistoryEditDenial::Accounting(Denial::WorkExhausted {
                maximum_visits: limit,
            })
        };
        assert_eq!(denial, expected_denial);
    }
    let removed = admitted
        .evict_front_with_retained_charge(&mut Work::new(1000))
        .unwrap()
        .unwrap();
    assert!(std::ptr::eq(
        Arc::as_ptr(&removed),
        original.front().unwrap()
    ));
    assert_eq!(*removed, summary);
    let remaining_charge = ordered_index_charge::<u64, Arc<ExecutionHistorySummary>>(1)
        .unwrap()
        .checked_add(arc_allocation_charge::<ExecutionHistorySummary>().unwrap())
        .unwrap()
        .checked_add(incoming_heap)
        .unwrap();
    assert_eq!(
        admitted.prepared_retained_charge().unwrap(),
        remaining_charge
    );
    assert_eq!(original.len(), 1);
}

fn two_frame_charge(old_charge: Charge, incoming: &ExecutionHistorySummary) -> Charge {
    let charge = old_charge
        .checked_sub(ordered_index_charge::<u64, Arc<ExecutionHistorySummary>>(1).unwrap())
        .unwrap()
        .checked_add(ordered_index_charge::<u64, Arc<ExecutionHistorySummary>>(2).unwrap())
        .unwrap()
        .checked_add(arc_allocation_charge::<ExecutionHistorySummary>().unwrap())
        .unwrap()
        .checked_add(incoming.retained_heap_charge(&mut Work::new(1000)).unwrap())
        .unwrap();
    charge
}
