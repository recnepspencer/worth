//! Each cancellation is raised inside equality; work belongs to the caller.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::consult_observation::{
    Door, ReadConsultObservationForTest,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Session,
};
use crate::facade::primary_graph::{
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
struct Policy(Placement, Option<worth_foundational::ExecutionBudget>);
/// How a stopped comparison left the public commit door.
#[derive(Debug, PartialEq)]
enum Stop {
    /// The fresh read-set owner kept the original request's control outcome.
    Cancelled,
    /// A refusal, with its stage and execution cause.
    Denied(
        WorthQueryApplicationCommitDenialStage,
        Option<Result<Session, Control>>,
    ),
}
impl Drop for Policy {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
#[test]
fn equality_stop_oracle_at_the_public_commit_door() {
    let (count, _) = run(None, None, None);
    assert!(count > 1, "entry-only observation cannot prove the read");
    println!("equality commit door: {count} consults");
    // The fresh read-set comparison runs first and reports the request's own
    // cancellation; once the commit-time recompare is reached, its consults keep
    // the ProviderCommit denial with the same cancellation cause.
    let mut recompared = false;
    for k in 1..=count {
        let (seen, stop) = run(Some(k), None, None);
        assert!(seen >= k);
        match stop {
            Some(Stop::Cancelled) => {
                assert!(
                    !recompared,
                    "cancel at consult {k} returned to the fresh read set"
                );
            }
            Some(Stop::Denied(stage, cause)) => {
                assert!(
                    k > 1,
                    "the first equality consult belongs to the fresh read set"
                );
                assert_eq!(
                    (stage, cause),
                    (
                        WorthQueryApplicationCommitDenialStage::ProviderCommit,
                        Some(Err(Control::Cancelled))
                    ),
                    "cancel at consult {k}"
                );
                recompared = true;
            }
            None => panic!("cancel at consult {k} was not observed"),
        }
        let (seen, stop) = run(None, None, Some(k));
        assert!(seen >= k, "work stop must reach equality consult {k}");
        let Some(Stop::Denied(_, cause)) = stop else {
            panic!("a work stop at equality consult {k} is a denial: {stop:?}");
        };
        assert_eq!(cause, Some(Ok(Session::ExecutionResource {
            denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted,
            partition_identity: None, policy_ancestor: None,
        })), "inner work stop at equality consult {k}");
    }
    // A write cannot publish under a caller's zero-work request.
    let (_, stop) = run(None, Some(0), None);
    assert!(matches!(stop, Some(Stop::Denied(_, Some(Ok(Session::ExecutionResource {
        denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, ..
    }))))));
}
fn run(
    cancel_at: Option<usize>,
    work: Option<u64>,
    work_at: Option<usize>,
) -> (usize, Option<Stop>) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, source.token());
    let program = world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), &world, &request);
            let account = resolved_account(&phase.test_host_request(), &world, "open", &request);
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "oracle-write",
            )
        })
        .unwrap();
    let _policy = Policy(
        place(Placement::Leased(std::num::NonZeroUsize::MIN)),
        bound(work.or_else(|| work_at.map(|_| u64::MAX - 1)).map(|work| {
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(work))
        })),
    );
    let probe = match work_at {
        Some(k) => ReadConsultObservationForTest::install_with_work_on_this_thread_for_test(
            Door::Equality,
            Some(k),
            move |context| {
                // Exhaust the existing caller meter inside this actual read.
                // Free consultations do not acquire a production unit price.
                let context = context.expect("the admitted caller meter");
                let attempted = context
                    .remaining_work()
                    .expect("this adversarial scope selected a finite ceiling")
                    .checked_add(1)
                    .unwrap();
                assert_eq!(
                    context.checkpoint(attempted),
                    Err(worth_execution::MapKernelStop::WorkCeiling)
                );
            },
        ),
        None => ReadConsultObservationForTest::install_on_this_thread_for_test(
            Door::Equality,
            cancel_at.map(|k| (k, source)),
        ),
    };
    let result = world.application.compare_and_commit_application(
        program,
        idempotency(91, 91),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let stop = if cancel_at.is_none() && work.is_none() && work_at.is_none() {
        assert!(matches!(
            result,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        None
    } else {
        let stop = if matches!(result, WorthQueryApplicationCommitOutcome::Cancelled) {
            Stop::Cancelled
        } else {
            let WorthQueryApplicationCommitOutcome::Denied(denial) = result else {
                panic!("a stopped comparison cannot publish: {result:?}");
            };
            Stop::Denied(denial.stage(), denial.execution_denial_cause())
        };
        assert_eq!(
            world
                .application
                .primary_provider
                .published_application_commit_count(),
            0
        );
        Some(stop)
    };
    (probe.consults(), stop)
}
