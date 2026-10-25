//! Domain evidence reaches the caller that attempted the installed commit.
mod ledger_rule;
mod producer_requirement;
mod program;
mod rule;
mod schema;
mod world;

use crate::domain_computation::primary_graph::*;
use crate::domain_computation::WorthQueryProviderSessionControlStopKind;
use rule::{OtherBinding, Refusal, RefusalBinding, Rule};
use std::sync::{Arc, Mutex};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

struct Case {
    denial: WorthQueryApplicationCommitDenial,
    before: Vec<u8>,
    after: Vec<u8>,
    visits: Vec<u64>,
}

struct RestorePlacement(WorthQueryExecutionPlacementForTest);
impl Drop for RestorePlacement {
    fn drop(&mut self) {
        place_managed_computations_on_this_thread_for_test(self.0);
    }
}

fn checkpoint(world: &world::World) -> Vec<u8> {
    world.application.with_host_advancement(|phase| {
        world.application.capture_application_checkpoint(
            &phase, crate::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        ).unwrap()
    }).unwrap().decode().unwrap().native.bytes().to_vec()
}

fn run(status: String, cancel: bool) -> Case {
    run_cancelled_by(status, cancel.then_some(1))
}

/// `canceller` names the rule that cancels the request on finding its
/// violation; the other rule reports its violation as usual.
fn run_cancelled_by(status: String, canceller: Option<u64>) -> Case {
    let _isolation = super::super::fixture::isolated_request_owner();
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let cancellation = WorthQueryCancellationSource::new();
    let visits = Arc::new(Mutex::new(Vec::new()));
    let rule = |ordinal| Rule {
        ordinal,
        cancellation: (canceller == Some(ordinal)).then(|| cancellation.clone()),
        visits: visits.clone(),
    };
    let world = world::install(rule(1), rule(2));
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let program = program::prepare(&world, &scope, status);
    let before = checkpoint(&world);
    let _restore = RestorePlacement(place_managed_computations_on_this_thread_for_test(
        WorthQueryExecutionPlacementForTest::Leased(std::num::NonZeroUsize::new(2).unwrap()),
    ));
    let mut original = None;
    let _opening = world
        .application
        .with_application_advancement(&scope, |phase| {
            original = Some(
                world
                    .application
                    .compare_and_commit_application_in_advancement(
                        &phase,
                        program,
                        super::idempotency(199, 199),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ),
            );
        });
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        original.expect("the caller received its original outcome")
    else {
        panic!("the candidate must not publish");
    };
    let after = checkpoint(&world);
    let visits = visits.lock().unwrap().clone();
    Case {
        denial,
        before,
        after,
        visits,
    }
}

#[test]
fn typed_refusal_round_trips_to_the_original_publication_caller() {
    let case = run("closed".into(), false);
    assert_eq!(
        case.denial
            .decode_invariant_refusal::<RefusalBinding>()
            .unwrap(),
        Refusal {
            rule: 1,
            observed: "closed".into()
        }
    );
    assert_eq!(
        case.denial.invariant_refusal_bytes(),
        Some(br#"{"rule":1,"observed":"closed"}"#.as_slice())
    );
}

#[test]
fn refused_candidate_keeps_native_checkpoint_bytes_equal() {
    let case = run("closed".into(), false);
    assert_eq!(case.before, case.after);
}

/// Rules run on parallel workers, so the other rule may or may not have
/// found its violation when `canceller` cancels; either way the outcome is
/// the stop, never the violation.
fn assert_cancelled_without_domain_evidence(case: &Case, canceller: u64) {
    assert_eq!(
        case.denial.execution_denial_cause(),
        Some(Err(WorthQueryProviderSessionControlStopKind::Cancelled))
    );
    assert!(case.denial.invariant_refusal_bytes().is_none());
    assert!(case.denial.custom_invariant_denial().is_none());
    assert_eq!(
        case.visits
            .iter()
            .filter(|&&rule| rule == canceller)
            .count(),
        1
    );
    assert!(case.visits.iter().all(|&rule| rule == 1 || rule == 2));
    assert_eq!(case.before, case.after);
}

#[test]
fn cancellation_during_evaluation_is_not_a_domain_violation() {
    assert_cancelled_without_domain_evidence(&run("closed".into(), true), 1);
}

#[test]
fn a_violation_found_before_another_rule_cancels_does_not_surface() {
    let case = run_cancelled_by("closed".into(), Some(2));
    assert_cancelled_without_domain_evidence(&case, 2);
}

#[test]
fn first_installed_violation_wins_over_identity_and_group_sort_order() {
    let mut case = run("closed".into(), false);
    // Parallel workers record visits in either order; both rules ran.
    case.visits.sort_unstable();
    assert_eq!(case.visits, [1, 2]);
    assert_eq!(
        case.denial
            .custom_invariant_violation_identity()
            .unwrap()
            .rule_id
            .as_str(),
        "ZFirstInstalled"
    );
    assert_eq!(
        case.denial
            .decode_invariant_refusal::<RefusalBinding>()
            .unwrap()
            .rule,
        1
    );
}

#[test]
fn oversized_refusal_is_an_evaluation_error() {
    let case = run("x".repeat(120), false);
    let (_, _, failure) = case
        .denial
        .custom_invariant_failure()
        .expect("the codec cannot evaluate a bounded refusal");
    assert_eq!(
        failure,
        worth_relational::facade::transactions::ResultCustomInvariantFailureKind::ExecutionError
    );
    assert!(case.denial.invariant_refusal_bytes().is_none());
    assert_eq!(case.before, case.after);
}

#[test]
fn refusal_decode_type_mismatch_is_a_typed_denial() {
    let case = run("closed".into(), false);
    assert!(
        matches!(case.denial.decode_invariant_refusal::<OtherBinding>(),
        Err(crate::facade::application_invariants::WorthQueryInvariantRefusalDecodeDenial::TypeMismatch { expected, observed })
        if expected == "tests.other-refusal.v1" && observed == "tests.account-refusal.v1")
    );
}

#[test]
fn identical_candidates_return_identical_refusal_bytes_on_two_runs() {
    let first = run("closed".into(), false);
    let second = run("closed".into(), false);
    assert_eq!(
        first.denial.invariant_refusal_bytes(),
        second.denial.invariant_refusal_bytes()
    );
    assert!(first.denial.invariant_refusal_bytes().is_some());
}
