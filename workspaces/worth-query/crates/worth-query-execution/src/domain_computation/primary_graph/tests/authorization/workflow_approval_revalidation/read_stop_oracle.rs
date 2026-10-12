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
    let (count, _) = run(None, None);
    assert!(count > 1);
    println!("approval readmission door: {count} consults");
    for k in 1..=count {
        let (seen, stop) = run(Some(k), None);
        assert!(seen >= k);
        assert_eq!(
            stop,
            Some(WorthQueryOperationAuthorizationDenialKind::Cancelled)
        );
        let (_, stop) = run(None, Some((k - 1) as u64));
        use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
        use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
        use WorthQueryOperationAuthorizationDenialKind as Kind;
        match stop {
            Some(Kind::CanonicalWorkDenied)
            | Some(Kind::RelationalExecution(Native::WorkExhausted))
            | Some(Kind::ExecutionRequest(Session::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            })) => {}
            other => panic!("work stop at consult {k}: {other:?}"),
        }
    }
}
fn run(
    cancel_at: Option<usize>,
    work: Option<u64>,
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
    let basis = durable_approval_basis(&admission);
    let _policy = Policy(
        place(Placement::Leased(std::num::NonZeroUsize::MIN)),
        bound(work.map(|work| {
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, 64 << 20, work)
        })),
    );
    let probe = match work {
        Some(work) => ReadConsultObservationForTest::install_with_work_on_this_thread_for_test(
            Door::Approval,
            Some(work as usize + 1),
            move |context| {
                // Metadata consultations are free. At consult k the caller
                // attempts k actual work units on its ceiling of k - 1.
                assert_eq!(
                    context
                        .expect("the caller's admitted scope")
                        .checkpoint(work + 1),
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
            world
                .application
                .readmit_workflow_approval_authority(&execution, &basis, &admission)
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
        Err(denial) => {
            assert_eq!(denial, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted));
            Some(WorthQueryOperationAuthorizationDenialKind::CanonicalWorkDenied)
        }
    };
    (probe.consults(), stop)
}
