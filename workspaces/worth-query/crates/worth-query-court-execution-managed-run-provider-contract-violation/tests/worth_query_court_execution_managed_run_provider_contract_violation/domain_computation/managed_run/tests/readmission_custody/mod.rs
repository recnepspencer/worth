//! A bounded standalone provider fixture gates its real checkpoint restore at both placements.
//! The fixture has no product World, so it does not borrow an installed runtime's phase.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_custody::Restore;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_custody::restores;

use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test as bound;
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::advancement_host::AdvancementHost;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
#[test]
fn bounded_provider_fixture_opener_refuses_before_direct_and_workflow_readmission() {
    let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    use std::num::NonZeroUsize;
    bound(None);
    let host = AdvancementHost::install();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let (yielded, bridge, runtime) = host
            .run_provider_fixture(|phase| {
                let custody = phase.test_host_request_for_test();
                let bootstrap = custody.bootstrap_for_test();
                worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_direct::yielded_direct(&custody, bootstrap.execution_request())
            })
            .unwrap();
        let mut direct = Some(yielded);
        let (yielded, workflow_bridge, workflow_runtime, _) = host
            .run_provider_fixture(|phase| {
                let fixture_custody = phase.test_host_request_for_test();
                let bootstrap = fixture_custody.bootstrap_for_test();
                worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_workflow::yielded_workflow(
                    &phase.test_host_request_for_test(),
                    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::yield_fixture::YieldProvider::installed(5),
                    bootstrap.execution_request(),
                )
            })
            .unwrap();
        let mut workflow = Some(yielded);
        for memory in [false, true] {
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if memory { Some(0) } else { None },
                Some(if memory { 8_000_000 } else { 0 }),
            )));
            let before = restores();
            let direct_stop = host
                .run_provider_fixture(|phase| {
                    let yielded = direct.take().unwrap();
                    let installed =
                        yielded.fixture_phase_for_test(&phase.test_host_request_for_test());
                    yielded.readmit_same_runtime(&installed, &runtime, &bridge)
                })
                .err()
                .expect("zero request refuses before direct restore");
            let workflow_stop = host
                .run_provider_fixture(|phase| {
                    let yielded = workflow.take().unwrap();
                    let installed =
                        yielded.fixture_phase_for_test(&phase.test_host_request_for_test());
                    yielded.readmit_same_runtime(&installed, &workflow_runtime, &workflow_bridge)
                })
                .err()
                .expect("zero request refuses before workflow restore");
            assert_eq!(direct_stop, workflow_stop);
            if !memory {
                assert_eq!(direct_stop, Denial::Resource(Resource::WorkExhausted));
            } else {
                let Denial::Resource(Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                }) = direct_stop
                else {
                    panic!("lease bytes")
                };
                assert_eq!(
                    level,
                    worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy
                );
                assert!(requested > 0);
                assert_eq!(admitted, 0);
            }
            assert_eq!(restores(), before);
        }
        bound(None);
        let before = restores();
        let direct = host
            .run_provider_fixture(|phase| {
                let yielded = direct.take().unwrap();
                let installed = yielded.fixture_phase_for_test(&phase.test_host_request_for_test());
                yielded.readmit_same_runtime(&installed, &runtime, &bridge)
            })
            .unwrap();
        assert!(
            restores() > before,
            "direct control reaches its actual checkpoint reader"
        );
        let worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(direct) =
            direct
        else {
            panic!("admitted direct restore")
        };
        let _ = direct.into_active().abandon();
        let before = restores();
        let workflow = host
            .run_provider_fixture(|phase| {
                let yielded = workflow.take().unwrap();
                let installed = yielded.fixture_phase_for_test(&phase.test_host_request_for_test());
                yielded.readmit_same_runtime(&installed, &workflow_runtime, &workflow_bridge)
            })
            .unwrap();
        assert!(
            restores() > before,
            "workflow control reaches its actual checkpoint reader"
        );
        let worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionOutcome::Readmitted(workflow) =
            workflow
        else {
            panic!("admitted workflow restore")
        };
        let _ = workflow.into_active().abandon();
    }
}
