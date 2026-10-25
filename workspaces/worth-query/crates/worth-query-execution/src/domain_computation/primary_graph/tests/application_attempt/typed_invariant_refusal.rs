//! Domain evidence reaches the caller that attempted the installed commit.
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
    reads: Vec<rule::ReadSample>,
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
    let _isolation = super::super::fixture::isolated_request_owner();
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let cancellation = WorthQueryCancellationSource::new();
    let visits = Arc::new(Mutex::new(Vec::new()));
    let reads = Arc::new(Mutex::new(Vec::new()));
    let world = world::install(
        Rule {
            ordinal: 1,
            cancellation: cancel.then(|| cancellation.clone()),
            visits: visits.clone(),
            reads: reads.clone(),
        },
        Rule {
            ordinal: 2,
            cancellation: None,
            visits: visits.clone(),
            reads: reads.clone(),
        },
    );
    reads.lock().unwrap().clear();
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
    let reads = reads.lock().unwrap().clone();
    Case {
        denial,
        before,
        after,
        visits,
        reads,
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

#[test]
fn cancellation_during_evaluation_is_not_a_domain_violation() {
    let case = run("closed".into(), true);
    assert_eq!(
        case.denial.execution_denial_cause(),
        Some(Err(WorthQueryProviderSessionControlStopKind::Cancelled))
    );
    assert!(case.denial.invariant_refusal_bytes().is_none());
    assert!(case.denial.custom_invariant_denial().is_none());
    assert_eq!(case.visits, [1]);
    assert_eq!(case.before, case.after);
}

#[test]
fn first_installed_violation_wins_over_identity_and_group_sort_order() {
    let case = run("closed".into(), false);
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

#[test]
fn installed_rule_reads_survive_repeated_views_and_domain_refusal() {
    let case = run("closed".into(), false);
    assert!(case
        .denial
        .decode_invariant_refusal::<RefusalBinding>()
        .is_ok());
    assert_read_observations(&case.reads);
}

#[test]
fn cancellation_preserves_the_reads_done_before_the_stop() {
    let case = run("closed".into(), true);
    assert_eq!(
        case.denial.execution_denial_cause(),
        Some(Err(WorthQueryProviderSessionControlStopKind::Cancelled))
    );
    assert_read_observations(&case.reads);
}

/// Both rules evaluate concurrently into one log, so only rule 1's samples are
/// in order; it reaches both views before it refuses or cancels.
fn assert_read_observations(samples: &[rule::ReadSample]) {
    use rule::ReadStage;
    let samples: Vec<_> = samples.iter().filter(|s| s.rule == 1).collect();
    let prepared = samples
        .iter()
        .find(|s| matches!(s.stage, ReadStage::Prepared))
        .unwrap()
        .reads;
    let evaluation: Vec<_> = samples
        .iter()
        .filter(|s| !matches!(s.stage, ReadStage::Prepared))
        .take(3)
        .collect();
    assert!(matches!(evaluation[0].stage, ReadStage::EvaluationStart));
    assert!(matches!(evaluation[1].stage, ReadStage::FirstView));
    assert!(matches!(evaluation[2].stage, ReadStage::SecondView));
    assert_eq!(evaluation[0].reads, prepared);
    for read_kind in [
        WorthQueryInvariantProjectionWork::field_reads,
        WorthQueryInvariantProjectionWork::adjacency_lists_read,
        WorthQueryInvariantProjectionWork::adjacency_edges_inspected,
    ] {
        let first = read_kind(evaluation[1].reads) - read_kind(evaluation[0].reads);
        let second = read_kind(evaluation[2].reads) - read_kind(evaluation[1].reads);
        assert!(first > 0, "the real installed read was observed");
        assert_eq!(
            first, second,
            "equivalent reads from fresh views contribute equally"
        );
    }
}
