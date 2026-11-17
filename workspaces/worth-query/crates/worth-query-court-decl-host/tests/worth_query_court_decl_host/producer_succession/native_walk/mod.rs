//! Genuine published walk-only producer reachability; never inject a retained prior.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    gathers: AtomicUsize,
    outcomes: Mutex<Vec<u64>>,
}
#[test]
fn a_published_walk_producer_exposes_its_actual_native_completion_eligibility() {
    let _observe=worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let calls = Arc::new(Calls::default());
    let app = producer_world::install(calls.clone());
    let (scope, principal, _cancel) = world::authenticate_cancellable(&app);
    let request = app.runtime().request(&principal, &scope);
    source_action::perform(&app, &request, 0, 1199);
    for (width, nonce) in [(1, 1200), (0, 1201), (2, 1202)] {
        worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        worth_query_host::facade::primary_graph::partitioned_computation_runs_on_this_thread_for_test();
        let before = calls.gathers.load(Ordering::Relaxed);
        source_action::perform(&app, &request, width, nonce);
        let verification =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        let runs=worth_query_host::facade::primary_graph::partitioned_computation_runs_on_this_thread_for_test();
        eprintln!(
            "walk producer width={width}: runs={runs:?}, gathers={}, native decline={:?}, fresh keys={}, patches={}",
            calls.gathers.load(Ordering::Relaxed) - before,
            verification.native_completion_last_decline,
            verification.completion_fresh_key_visits,
            verification.native_patch_path_visits
        );
        assert!(
            !runs.is_empty(),
            "actual registered producer performed its computation"
        );
    }
    assert_eq!(calls.outcomes.lock().unwrap().last().copied(), Some(2));
}
