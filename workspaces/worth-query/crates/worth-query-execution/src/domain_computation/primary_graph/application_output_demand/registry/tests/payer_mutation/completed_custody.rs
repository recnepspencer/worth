//! Request cleanup must pay for non-empty completed custody before editing.
use super::super::super::{BoundOutputSource, PreparedOutputRootKind, SourceCustody};
use super::*;

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
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
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
            occurrence(),
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
                root_kind: PreparedOutputRootKind::Discovered(std::any::TypeId::of::<()>()),
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
                completed: true,
            },
        );
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
