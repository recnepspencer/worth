//! The private recovery continuation retains the native custody refusal.
use worth_query_execution::facade::domain_computation::WorthQueryBranchAdoptionRecoveryDenial;
use worth_query_execution::domain_computation::primary_graph::product_operation::program_adoption::custody::recovery::readmission::recovery_execution;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial;
#[test]
fn private_recovery_keeps_the_foreign_phase_cause() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let issuer = installed_authorization_world(true);
    let receiver = installed_authorization_world(true);
    issuer
        .application
        .with_host_advancement(|phase| {
            let denied =
                recovery_execution(&phase, receiver.application.product_runtime()).unwrap_err();
            match denied {
                WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied(cause) => assert_eq!(
                    cause,
                    WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase
                    ),
                ),
                other => panic!("foreign phase changed its cause: {other:?}"),
            }
            assert!(recovery_execution(&phase, issuer.application.product_runtime()).is_ok());
        })
        .unwrap();
}
