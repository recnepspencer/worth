//! Domain evidence reaches the caller that attempted the installed commit.

use worth_relational::facade::durability::CheckpointAllocatorInspection;
use worth_relational::facade::durability::CheckpointRecordIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest;
use worth_query_execution::facade::application_invariants::WorthQueryInvariantProjectionWork;
use worth_relational::facade::inspection::RecordReservationReport as WorthQueryRecordReservationReport;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::denial;
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::OtherBinding;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::Refusal;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::RefusalBinding;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::Rule;
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::durability::RelationalNativeCheckpoint;

pub struct Case {
    pub denial: WorthQueryApplicationCommitDenial,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
    pub reservations: WorthQueryRecordReservationReport,
    pub visits: Vec<u64>,
    pub reads: Vec<crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::ReadSample>,
}

impl Case {
    pub fn assert_model_unchanged(&self) {
        let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        assert!(
            self.reservations.records().len() > 0,
            "this refused commit really reserved ids"
        );
        let before = RelationalNativeCheckpoint::inspect_bytes(&execution, &self.before).unwrap();
        let after = RelationalNativeCheckpoint::inspect_bytes(&execution, &self.after).unwrap();
        assert_eq!(before.model, after.model);
        assert!(
            after.symbol_identity_frontier >= before.symbol_identity_frontier,
            "abandoned symbol reservations are monotonic; published meanings remain exact"
        );
        assert_eq!(before.live_records, after.live_records);
        CheckpointAllocatorInspection {
            frontiers: &before.frontiers,
            generations: &before.generations,
        }
        .verify_reported_advance(
            &CheckpointAllocatorInspection {
                frontiers: &after.frontiers,
                generations: &after.generations,
            },
            self.reservations
                .records()
                .map(CheckpointRecordIdentity::from),
        )
        .unwrap();
    }
}

pub struct RestorePlacement(WorthQueryExecutionPlacementForTest);
impl Drop for RestorePlacement {
    fn drop(&mut self) {
        place_managed_computations_on_this_thread_for_test(self.0);
    }
}

pub fn checkpoint(
    world: &crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::world::World,
) -> Vec<u8> {
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            world
                .application
                .primary_provider
                .graph
                .with_runtime(&execution, |runtime| {
                    runtime
                        .durability_authority()
                        .native_checkpoint(&execution, execution.allocation_policy())
                        .unwrap()
                        .bytes()
                        .to_vec()
                })
        })
        .unwrap()
}

pub fn run(status: String, cancel: bool) -> Case {
    run_cancelled_by(status, cancel.then_some(1))
}

/// `canceller` names the rule that cancels the request on finding its
/// violation; the other rule reports its violation as usual.
pub fn run_cancelled_by(status: String, canceller: Option<u64>) -> Case {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let cancellation = WorthQueryCancellationSource::new();
    let visits = Arc::new(Mutex::new(Vec::new()));
    let reads = Arc::new(Mutex::new(Vec::new()));
    let rule = |ordinal| Rule {
        ordinal,
        cancellation: (canceller == Some(ordinal)).then(|| cancellation.clone()),
        visits: visits.clone(),
        reads: reads.clone(),
    };
    let world = crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::world::install(rule(1), rule(2));
    reads.lock().unwrap().clear();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let program = crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::program::prepare(&world, &scope, status);
    let before = checkpoint(&world);
    let reservations = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_record_reservations(
                    &phase,
                    16,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
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
                        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(199, 199),
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
        reservations: reservations
            .report(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
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
fn refused_candidate_changes_no_native_model_bytes() {
    let case = run("closed".into(), false);
    assert!(case
        .denial
        .decode_invariant_refusal::<RefusalBinding>()
        .is_ok());
    case.assert_model_unchanged();
}

/// Rules run on parallel workers, so the other rule may or may not have
/// found its violation when `canceller` cancels; either way the outcome is
/// the stop, never the violation.
pub fn assert_cancelled_without_domain_evidence(case: &Case, canceller: u64) {
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
    case.assert_model_unchanged();
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
    case.assert_model_unchanged();
}

#[test]
fn refusal_decode_type_mismatch_is_a_typed_denial() {
    let case = run("closed".into(), false);
    assert!(
        matches!(case.denial.decode_invariant_refusal::<OtherBinding>(),
        Err(worth_query_execution::facade::application_invariants::WorthQueryInvariantRefusalDecodeDenial::TypeMismatch { expected, observed })
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

/// Both rules record on parallel workers, so their samples interleave; rule 1
/// reads both views before it refuses or cancels, so its own sequence is
/// asserted.
pub fn assert_read_observations(
    samples: &[crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::ReadSample],
) {
    use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::ReadStage;
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

#[path = "ledger_rule/mod.rs"]
pub mod ledger_rule;
#[path = "producer_requirement/mod.rs"]
pub mod producer_requirement;
#[path = "program/mod.rs"]
pub mod program;
#[path = "rule/mod.rs"]
pub mod rule;
#[path = "schema/mod.rs"]
pub mod schema;
#[path = "world/mod.rs"]
pub mod world;
