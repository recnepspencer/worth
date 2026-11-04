//! Root input work and cancellation remain on the original installed request.
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::Probe;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::*;
use worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test as reports;
use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test as bound;
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_relational::facade::runtime::CustomInvariantInterruption;
use worth_foundational::facade::RelationDirection;
use worth_query_execution::facade::application_invariants::WorthQueryInvariantAccessDenialKind;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
pub struct Case {
    pub outcome: WorthQueryApplicationCommitOutcome,
    pub read: Result<usize, WorthQueryInvariantAccessDenialKind>,
    pub charged: u64,
}
pub fn run(leased: bool, roots: usize, ceiling: Option<u64>, cancel: bool) -> Case {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(Placement::Serial), bound(None));
    let source = WorthQueryCancellationSource::new();
    let cancellation = source.clone();
    let observations = Arc::new(Mutex::new(Vec::new()));
    let observed = observations.clone();
    let probe = Probe::new(move |context, scope, rule| {
        let before = context.reads();
        if cancel {
            cancellation.cancel();
        }
        // This vector is the test's input, made from an actual local identity.
        let result = if roots == 0 {
            Ok(0)
        } else {
            let inputs = vec![scope.node(0).clone(); roots];
            context
                .proposed()
                .closure(&rule.next, RelationDirection::Forward, &inputs)
                .map(|walk| walk.len())
                .map_err(|error| error.kind())
        };
        if result.is_err() {
            assert_eq!(
                context.reads(),
                before,
                "input refusal precedes graph acquisition"
            );
        }
        observed.lock().unwrap().push(result);
    });
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::install_with_probe(worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::Graph::three_hops(false, vec![]), probe.clone());
    let request = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        source.token(),
    );
    let program = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::activation::set_active(&world, &request, true);
    let before = world.checkpoint();
    probe.arm();
    reports();
    place(if leased {
        Placement::Leased(NonZeroUsize::MIN)
    } else {
        Placement::Serial
    });
    bound(
        ceiling.map(|work| {
            worth_foundational::ExecutionBudget::new(NonZeroUsize::MIN, None, Some(work))
        }),
    );
    let outcome = commit(&world, &request, program, 221);
    let requests = reports();
    assert_eq!(requests.len(), 1, "one original funded application attempt");
    let charged = requests[0].as_ref().unwrap().charged_work();
    if matches!(outcome, WorthQueryApplicationCommitOutcome::Denied(_)) {
        assert_eq!(world.checkpoint(), before);
    }
    let read = *observations
        .lock()
        .unwrap()
        .last()
        .expect("the actual installed probe reached the public read");
    Case {
        outcome,
        read,
        charged,
    }
}

#[test]
fn duplicate_root_work_refuses_on_the_original_serial_and_leased_budget_with_funded_twins() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    for leased in [false, true] {
        let control = run(leased, 0, None, false);
        assert!(matches!(
            control.outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        assert_eq!(control.read, Ok(0));
        // Ordinary owners fit the no-read control. The raw input alone exceeds
        // the entire chosen ceiling, independently of callback suffix work.
        let ceiling = control.charged.checked_add(64).unwrap();
        let roots = usize::try_from(ceiling.checked_add(1).unwrap()).unwrap();
        assert!(
            roots <= 1_000_000,
            "unexpected control cost exceeds this bounded input fixture"
        );
        let refused = run(leased, roots, Some(ceiling), false);
        assert_eq!(
            refused.read,
            Err(WorthQueryInvariantAccessDenialKind::WorkBudgetExceeded)
        );
        let WorthQueryApplicationCommitOutcome::Denied(denial) = refused.outcome else {
            panic!("original work exhaustion must deny the real commit");
        };
        assert!(matches!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: Resource::WorkExhausted,
                ..
            }
        ));
        let funded = run(leased, roots, None, false);
        assert_eq!(funded.read, Ok(4));
        assert!(matches!(
            funded.outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
}

#[test]
fn cancellation_inside_the_installed_callback_is_a_typed_read_refusal_with_a_live_twin() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    for leased in [false, true] {
        let refused = run(leased, 1, None, true);
        assert_eq!(
            refused.read,
            Err(WorthQueryInvariantAccessDenialKind::Interrupted(
                CustomInvariantInterruption::Cancelled
            ))
        );
        let WorthQueryApplicationCommitOutcome::Denied(denial) = refused.outcome else {
            panic!("original cancellation must deny the real commit");
        };
        assert_eq!(
            denial.execution_denial_cause(),
            Some(Err(Control::Cancelled))
        );
        let live = run(leased, 1, None, false);
        assert_eq!(live.read, Ok(4));
        assert!(matches!(
            live.outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
}
