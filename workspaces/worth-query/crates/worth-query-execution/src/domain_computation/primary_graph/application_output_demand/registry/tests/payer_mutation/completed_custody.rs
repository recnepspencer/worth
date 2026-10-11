//! Request cleanup must pay for non-empty completed custody before editing.
use super::super::super::refreshed_rejoin::infallible;
use super::super::super::{BoundOutputSource, PreparedOutputRootKind, SourceCustody};
use super::*;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support;

#[test]
fn registry_requests_price_completed_custody_joined() {
    check(0);
}
#[test]
fn registry_requests_price_completed_custody_cache() {
    check(1);
}
#[test]
fn registry_requests_price_completed_custody_unheld() {
    check(2);
}

fn check(entry: u8) {
    let receipt = recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = super::super::source_custody::real_scope_and_occurrence();
    let DemandState::Output(output) = closed_retirement::ready() else {
        panic!("Ready fixture")
    };
    let Some(WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else {
        panic!("Ready fixture")
    };
    let make = |entry: u8, seed| {
        let (registry, interest) = fixture(seed, &completion, entry == 0);
        let mut state = registry.state.lock().unwrap();
        let source = key("completed", seed + 1, 3);
        let mut row = record(
            receipt.product_branch().occurrence(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::NoEffect,
                "completed",
            )),
            0,
        );
        row.source_commits.push(commit.clone());
        state.records.insert(source.clone(), row);
        state.source_custody.insert(
            commit.clone(),
            SourceCustody {
                occurrence: receipt.product_branch().occurrence(),
                root_kind: PreparedOutputRootKind::Discovered(
                    crate::equality_type_id::EqualityTypeId::of::<()>(),
                ),
                source: None,
                discovery: None,
                bound_sources: Some(vec![BoundOutputSource {
                    scope,
                    identity: source.source.clone(),
                }]),
                consumed_sources: vec![source.source],
                retired_sources: Vec::new(),
                retired: None,
                token_count: 0,
                cleanup: Default::default(),
                completed: true,
            },
        );
        state.note_custody(&commit);
        drop(state);
        (registry, interest)
    };
    {
        for seed in 0..16 {
            let (completed, interest) = make(entry, seed);
            let mut measured = admission(1_000_000);
            run(entry, &completed, &interest, &mut measured).unwrap();
            let expected = value(&completed.state.lock().unwrap());
            assert!(expected.custodians.is_empty());
            // An additional seek/row charge cannot be free merely because
            // earlier registry edits completed inside the same lock.
            if seed == 0 {
                // +17: two custody-entry visits, source quote (2+1),
                // stale-owner quote and walk 2*(2+1), two source walks 2*(2+1).
                let expected = match entry {
                    0 => 36,
                    1 => 46,
                    2 => 44,
                    _ => panic!("unknown cleanup entry"),
                };
                assert_eq!(measured.charged_work(), expected);
            }
            for k in 0..measured.charged_work() {
                let (retry, interest) = make(entry, seed);
                assert_eq!(
                    run(entry, &retry, &interest, &mut admission(k)).unwrap_err(),
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                );
                run(entry, &retry, &interest, &mut admission(1_000_000)).unwrap();
                assert_eq!(
                    value(&retry.state.lock().unwrap()),
                    expected,
                    "entry {entry}, seed {seed}, k {k}"
                );
            }
        }
    }
}

#[test]
fn registry_completed_custody_is_not_swept_after_commit() {
    let receipt = recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = super::super::source_custody::real_scope_and_occurrence();
    for seed in 0..16 {
        let make = || {
            let registry = WorthQueryOutputDemandRegistry::default();
            let mut state = registry.state.lock().unwrap();
            let selected = key("queued-custody", seed + 1, 7);
            for generation in 1..=1 + seed % 3 {
                let mut row = record(
                    receipt.product_branch().occurrence(),
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::NoEffect,
                        "completed",
                    )),
                    0,
                );
                row.source_commits.push(commit.clone());
                state
                    .records
                    .insert(key("queued-custody", generation, 7), row);
            }
            state.source_custody.insert(
                commit.clone(),
                SourceCustody {
                    occurrence: receipt.product_branch().occurrence(),
                    root_kind: PreparedOutputRootKind::Discovered(
                        crate::equality_type_id::EqualityTypeId::of::<()>(),
                    ),
                    source: None,
                    discovery: None,
                    bound_sources: Some(vec![BoundOutputSource {
                        scope,
                        identity: selected.source.clone(),
                    }]),
                    consumed_sources: vec![selected.source],
                    retired_sources: Vec::new(),
                    retired: None,
                    token_count: 0,
                    cleanup: Default::default(),
                    completed: true,
                },
            );
            state.note_custody(&commit);
            drop(state);
            registry
        };
        let eager = make();
        infallible(
            eager
                .state
                .lock()
                .unwrap()
                .prune_completed_custody(&mut super::super::OracleWalk::after_effect()),
        );
        let expected = value(&eager.state.lock().unwrap());
        let deferred = make();
        let before = super::super::super::record_map::occurrence_rows_visited();
        deferred.complete_prepared_source(&receipt);
        assert!(
            !deferred.state.lock().unwrap().source_custody.is_empty(),
            "a committed completion must leave an O(1) note, not sweep custody"
        );
        assert_eq!(
            super::super::super::record_map::occurrence_rows_visited(),
            before
        );
        assert!(deferred.state.lock().unwrap().pending_custody_prune());
        let mut full = admission(1_000_000);
        deferred
            .state
            .lock()
            .unwrap()
            .drain_terminal_cleanup(&mut full)
            .unwrap();
        assert_eq!(value(&deferred.state.lock().unwrap()), expected);
        for k in 0..full.charged_work() {
            let retry = make();
            retry.complete_prepared_source(&receipt);
            let snapshot = value(&retry.state.lock().unwrap());
            let denial = retry
                .state
                .lock()
                .unwrap()
                .drain_terminal_cleanup(&mut admission(k))
                .unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            );
            assert_eq!(value(&retry.state.lock().unwrap()), snapshot);
            assert!(retry.state.lock().unwrap().pending_custody_prune());
            retry
                .state
                .lock()
                .unwrap()
                .drain_terminal_cleanup(&mut admission(1_000_000))
                .unwrap();
            assert_eq!(
                value(&retry.state.lock().unwrap()),
                expected,
                "seed {seed}, k {k}"
            );
        }
    }
}
