//! Real committed-source producer carriage, route and body custody.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_query_host::facade::application_contribution::{
    published_partitioned_computations_on_this_thread_for_test as publications,
    WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRunForTest as Run,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_host::facade::primary_graph::*;
mod handler;
mod model;
mod producer_world;
mod provider;
mod source_action;
#[derive(Default)]
pub struct Calls {
    // Fixed before installation; the field-only world protects sparse completion.
    field_only: bool,
    gathers: AtomicUsize,
    outcomes: Mutex<Vec<u64>>,
}
fn fresh_full(width: u64) -> u64 {
    let calls = Arc::new(Calls::default());
    let app = producer_world::install(calls.clone(), true);
    let (scope, principal, _cancel) = world::authenticate_cancellable(&app);
    let request = app.runtime().request(&principal, &scope);
    // Every reference reaches its first preserve through real authored effects.
    source_action::perform(&app, &request, if width == 0 { 2 } else { 0 }, 1300);
    partitioned_computation_runs_on_this_thread_for_test();
    source_action::perform(&app, &request, width, 1301);
    let runs = partitioned_computation_runs_on_this_thread_for_test();
    assert!(
        matches!(runs.as_slice(), [(Run::Full(Cause::FirstRun), _)]),
        "fresh reference really ran Full: {runs:?}"
    );
    assert_eq!(
        calls.gathers.load(Ordering::Relaxed),
        if width == 2 { 2 } else { 1 }
    );

    *calls.outcomes.lock().unwrap().last().unwrap()
}
fn worker_only_successor(include_native_carry: bool) {
    let _observe = WorthQueryExecutionObservation::install_on_this_thread();
    let calls = Arc::new(Calls::default());
    let app = producer_world::install(calls.clone(), true);
    let (scope, principal, _cancel) = world::authenticate_cancellable(&app);
    let request = app.runtime().request(&principal, &scope);
    source_action::perform(&app, &request, 0, 1199);
    publications();
    let mut prior = None;
    let mut outcomes = Vec::new();
    let changes: &[(u64, u64)] = if include_native_carry {
        &[(1, 1200), (0, 1201), (2, 1202)]
    } else {
        &[(1, 1200), (2, 1202)]
    };
    for &(width, nonce) in changes {
        fact_verification_on_this_thread_for_test();
        partitioned_computation_runs_on_this_thread_for_test();
        let before = calls.gathers.load(Ordering::Relaxed);
        source_action::perform(&app, &request, width, nonce);
        let verification = fact_verification_on_this_thread_for_test();
        let runs = partitioned_computation_runs_on_this_thread_for_test();
        // `None` is an incremental run.
        let expected = if width == 1 {
            Some(Cause::FirstRun)
        } else {
            None
        };
        assert!(
            matches!(runs.as_slice(), [(actual, _)] if actual.full_cause() == expected),
            "width={width}: {runs:?}"
        );
        assert_eq!(
            calls.gathers.load(Ordering::Relaxed) - before,
            usize::from(width != 0),
            "only the new partition gathers at width2; the unchanged partition carries"
        );
        let published = publications();
        assert_eq!(
            published.len(),
            1,
            "one actual parent producer published its retained computation"
        );
        let current = published.into_iter().next().unwrap();
        if width == 1 {
            assert!(
                runs[0]
                    .1
                    .expect("the Full run's real report")
                    .accounted_cost()
                    .is_none(),
                "WorkerOnly is the actual request policy"
            );
        }
        if width == 0 {
            assert_eq!(
                verification.native_completion_last_decline, None,
                "{verification:?}"
            );
            assert!(verification.completion_fresh_key_visits > 0 && verification.native_patch_path_visits > 0, "actual native completion must execute, not just successful projected sealing: {verification:?}");
        } else if width == 2 {
            assert_eq!(
                verification.native_completion_last_decline,
                Some("call-roster"),
                "the new owner call must cause the actual named native decline: {verification:?}"
            );
        }
        if let Some(prior) = &prior {
            assert_eq!(
                current.structural_walk_body_sharing_with_for_test(prior),
                Some(vec![(false, false)]),
                "both completion routes publish a new current body/source, never the retained Arc"
            );
        }
        outcomes.push(*calls.outcomes.lock().unwrap().last().unwrap());
        prior = Some(current);
    }
    if include_native_carry {
        assert_eq!(outcomes, [1, 1, 2]);
        assert_eq!(
            outcomes[1],
            fresh_full(0),
            "independent first-preserve Full reference"
        );
    } else {
        assert_eq!(outcomes, [1, 2]);
    }
    assert_eq!(
        *outcomes.last().unwrap(),
        fresh_full(2),
        "changed membership matches independent Full reference"
    );
}

#[test]
fn worker_only_committed_walk_producer_accepts_native_carry_and_declines_changed_call_roster() {
    worker_only_successor(true);
}
#[test]
fn worker_only_call_roster_decline_seals_carried_walk_without_fresh_owner_reread() {
    worker_only_successor(false);
}

#[test]
fn worker_only_nonstructural_native_carry_does_not_visit_fallback_metadata() {
    let _observe = WorthQueryExecutionObservation::install_on_this_thread();
    let calls = Arc::new(Calls {
        field_only: true,
        ..Calls::default()
    });
    let app = producer_world::install(calls.clone(), true);
    let (scope, principal, _cancel) = world::authenticate_cancellable(&app);
    let request = app.runtime().request(&principal, &scope);
    source_action::perform(&app, &request, 0, 1500);
    publications();
    let mut prior = None;
    for (width, nonce, expected, gathers) in
        [(1, 1501, Some(Cause::FirstRun), 1), (0, 1502, None, 0)]
    {
        fact_verification_on_this_thread_for_test();
        partitioned_computation_runs_on_this_thread_for_test();
        let before = calls.gathers.load(Ordering::Relaxed);
        source_action::perform(&app, &request, width, nonce);
        let verification = fact_verification_on_this_thread_for_test();
        let runs = partitioned_computation_runs_on_this_thread_for_test();
        assert!(
            matches!(runs.as_slice(), [(actual, _)] if actual.full_cause() == expected),
            "{runs:?}"
        );
        assert_eq!(calls.gathers.load(Ordering::Relaxed) - before, gathers);
        assert_eq!(
            calls.outcomes.lock().unwrap().last(),
            Some(&1),
            "the seeded Value survives actual carriage"
        );
        let published = publications();
        assert_eq!(published.len(), 1);
        let current = published.into_iter().next().unwrap();
        if let Some(prior) = &prior {
            assert_eq!(
                current.structural_walk_body_sharing_with_for_test(prior),
                Some(vec![]),
                "this is the field-only native population"
            );
            assert_eq!(
                verification.native_completion_last_decline, None,
                "{verification:?}"
            );
            assert!(
                verification.completion_fresh_key_visits > 0
                    && verification.native_patch_path_visits > 0,
                "actual native completion must execute: {verification:?}"
            );
            assert_eq!(verification.fallback_carried_metadata_visits, 0, "nonstructural native completion must not enumerate retained metadata: {verification:?}");
            assert_eq!(
                verification.carried_key_expansion_visits, 0,
                "{verification:?}"
            );
        } else {
            assert!(runs[0]
                .1
                .expect("the real Full report")
                .accounted_cost()
                .is_none());
        }
        prior = Some(current);
    }
}

#[test]
fn default_accounted_committed_walk_producer_runs_full_observation_over_budget() {
    let _observe = WorthQueryExecutionObservation::install_on_this_thread();
    let calls = Arc::new(Calls::default());
    let app = producer_world::install(calls.clone(), false);
    let (scope, principal, _cancel) = world::authenticate_cancellable(&app);
    let request = app.runtime().request(&principal, &scope);
    source_action::perform(&app, &request, 0, 1400);
    for (width, nonce, expected) in [
        (1, 1401, Cause::FirstRun),
        (0, 1402, Cause::ObservationOverBudget),
    ] {
        partitioned_computation_runs_on_this_thread_for_test();
        let before = calls.gathers.load(Ordering::Relaxed);
        source_action::perform(&app, &request, width, nonce);
        let runs = partitioned_computation_runs_on_this_thread_for_test();
        assert!(
            matches!(runs.as_slice(), [(Run::Full(actual), Some(report))] if *actual == expected && report.accounted_cost().is_some()),
            "the existing Accounted policy correctly runs Full: {runs:?}"
        );
        assert_eq!(calls.gathers.load(Ordering::Relaxed) - before, 1);
        assert_eq!(calls.outcomes.lock().unwrap().last(), Some(&1));
    }
}
