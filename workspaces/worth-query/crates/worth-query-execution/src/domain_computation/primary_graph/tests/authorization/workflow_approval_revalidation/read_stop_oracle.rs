//! The readmission door preserves stops raised inside its freshness reads.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::consult_observation::{
    Door, ReadConsultObservationForTest,
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
impl Drop for Policy {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
#[test]
fn read_stop_oracle_at_the_approval_readmission_door() {
    use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    use WorthQueryOperationAuthorizationDenialKind as Kind;
    let (count, _) = run(None, None);
    assert!(count > 1);
    for k in 1..=count {
        let (seen, stop) = run(Some(k), None);
        assert!(seen >= k);
        assert_eq!(
            stop,
            Some(Kind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
            )))
        );
        let (seen, stop) = run(None, Some(k));
        assert!(seen >= k, "work stop must reach approval consult {k}");
        // Principal, grant selection and durable dependency owners use their
        // existing typed work vocabularies. Opening cannot counterfeit a row.
        assert_eq!(
            stop,
            Some(Kind::Stopped(Ok(Session::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            })))
        );
    }
}
fn run(
    cancel_at: Option<usize>,
    work: Option<usize>,
) -> (usize, Option<WorthQueryOperationAuthorizationDenialKind>) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_delegated_capability_world();
    world.authorization_time.hold(time(100));
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, source.token());
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), &world, &request);
            admitted_capability_operation(&phase.test_host_request(), &world, &principal, &request)
        })
        .unwrap();
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            durable_approval_basis(&host.borrowed_request_for_test(), &admission)
        })
        .expect("the fixture envelope owns its installed host call");
    let _policy = Policy(
        place(Placement::Leased(std::num::NonZeroUsize::MIN)),
        // Admission stays outside the observed read.
        bound(None),
    );
    let probe = match work {
        Some(work) => ReadConsultObservationForTest::install_with_work_on_this_thread_for_test(
            Door::Approval,
            Some(work),
            move |context| {
                // Exceed the live caller ceiling at the selected read consultation.
                assert_eq!(
                    {
                        let context = context.expect("the caller's admitted scope");
                        context.checkpoint(
                            context
                                .remaining_work()
                                .expect("this adversarial scope selected a finite ceiling")
                                + 1,
                        )
                    },
                    Err(worth_execution::MapKernelStop::WorkCeiling)
                );
            },
        ),
        None => ReadConsultObservationForTest::install_on_this_thread_for_test(
            Door::Approval,
            cancel_at.map(|k| (k, source)),
        ),
    };
    let result = world
        .application
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            let read = || {
                world
                    .application
                    .readmit_workflow_approval_authority(&execution, &basis, &admission)
            };
            match work {
                Some(_) => {
                    execution
                        .run(
                            worth_execution::ExecutionWorkCeiling::new(
                                execution
                                    .work_ceiling()
                                    .into_iter()
                                    .chain([u64::MAX - 1])
                                    .min(),
                            ),
                            |_| read(),
                        )
                        .expect("the read's own admitted scope")
                        .0
                }
                None => read(),
            }
        });
    let stop = match result {
        Ok(Ok(_)) => {
            assert!(
                cancel_at.is_none() && work.is_none(),
                "admitted: cancel_at={cancel_at:?}, work={work:?}, consults={}",
                probe.consults()
            );
            None
        }
        Ok(Err(denial)) => Some(denial.kind()),
        Err(denial) => panic!("opening is not the approval read: {denial:?}"),
    };
    (probe.consults(), stop)
}
