use super::*;
use crate::data::aspect::Aspect;
use crate::data::retained_storage::{
    SignalConditionalRetentionLedger as Ledger,
    SignalConditionalRetentionReservation as Reservation,
};
use crate::facade::SignalGraph;
use crate::runtime_policy::SignalConditionalEvaluationBudget;
use crate::state::{SignalBranchHandle, SignalBranchId};

fn fixture(population: usize, payload: usize) -> DiagnosticsState {
    let mut graph = SignalGraph::new();
    let mut state = DiagnosticsState::default();
    for index in 0..population {
        let node = graph.node().build();
        state.note_change_input(node, Aspect::new(3), &[], Some("cause".repeat(payload)));
        let id = SignalBranchId(index as u64 + 1);
        state.branch_catalog.insert(
            id,
            SignalBranchHandle {
                id,
                name: "branch".repeat(payload),
                parent_branch_id: Some(SignalBranchId(0)),
                head_snapshot_id: None,
            },
        );
    }
    state
}
fn ledger(bytes: u64) -> std::sync::Arc<Ledger> {
    Ledger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: bytes,
            maximum_attempt_visits: 1_000_000,
        },
        crate::runtime_policy::SignalConditionalTemporalBudget {
            maximum_live_partitions: 1,
            maximum_reserved_active_wakes: 1,
        },
    )
}
fn cold(state: &mut DiagnosticsState) {
    state
        .prepare_retained_heap_charge(&mut Work::new(1_000_000))
        .unwrap();
}

#[test]
fn diagnostic_fork_growth_requires_cold_facts_and_denies_without_conversion() {
    let mut state = fixture(8, 128);
    let wire = serde_json::to_value(&state).unwrap();
    assert_eq!(
        state.prepare_fork_growth(&mut Work::new(100_000)),
        Err(DiagnosticForkGrowthDenial::PreparationRequired)
    );
    cold(&mut state);
    let mut measured = Work::new(100_000);
    let growth = state.prepare_fork_growth(&mut measured).unwrap();
    assert!(matches!(
        state.prepare_fork_growth(&mut Work::new(measured.visits() - 1)),
        Err(DiagnosticForkGrowthDenial::Accounting(
            RetainedStoragePreparationDenial::WorkExhausted { .. }
        ))
    ));
    let denied = ledger(growth.bytes() + std::mem::size_of::<Reservation>() as u64 - 1);
    assert!(denied.reserve(0, growth).is_err());
    assert_eq!(denied.usage(), (0, 0));
    assert_eq!(
        state.prepare_fork_growth(&mut Work::new(100_000)).unwrap(),
        growth
    );
    assert_eq!(serde_json::to_value(&state).unwrap(), wire);
}

#[test]
fn diagnostic_fork_conversion_custody_survives_either_root_drop_order() {
    for drop_source_first in [false, true] {
        let mut state = fixture(8, 128);
        cold(&mut state);
        let wire = serde_json::to_value(&state).unwrap();
        let growth = state.prepare_fork_growth(&mut Work::new(100_000)).unwrap();
        let ledger = ledger(growth.bytes() + std::mem::size_of::<Reservation>() as u64);
        let mut resources = ledger.reserve(0, growth).unwrap();
        let mut candidate = state.fork_reserved(&mut resources);
        drop(resources);
        assert_eq!(ledger.usage().1, growth.bytes());
        assert_eq!(serde_json::to_value(&candidate).unwrap(), wire);
        assert!(state.branch_catalog.ptr_eq(&candidate.branch_catalog));
        assert_eq!(
            candidate
                .prepare_fork_growth(&mut Work::new(100_000))
                .unwrap(),
            Charge::ZERO
        );
        let survivor = if drop_source_first {
            drop(state);
            candidate
        } else {
            drop(candidate);
            state
        };
        assert_eq!(ledger.usage().1, growth.bytes());
        assert_eq!(serde_json::to_value(&survivor).unwrap(), wire);
        drop(survivor);
        assert_eq!(ledger.usage(), (0, 0));
    }
}

#[test]
fn prepared_diagnostic_fork_work_does_not_walk_catalog_or_payloads() {
    let mut observed = Vec::new();
    for (population, payload) in [(1, 1), (8, 128), (64, 1024)] {
        let mut state = fixture(population, payload);
        assert_eq!(state.branch_catalog.len(), population + 1);
        assert_eq!(
            state.pending_input.as_ref().unwrap().changed_nodes.len(),
            population
        );
        cold(&mut state);
        let mut work = Work::new(100_000);
        let growth = state.prepare_fork_growth(&mut work).unwrap();
        observed.push((growth, work.visits()));
    }
    assert!(observed.windows(2).all(|pair| pair[0] == pair[1]));
}
