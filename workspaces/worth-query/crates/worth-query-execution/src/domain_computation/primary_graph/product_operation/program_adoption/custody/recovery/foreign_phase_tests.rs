//! The private recovery continuation retains the native custody refusal.
use super::*;
use crate::domain_computation::primary_graph::{
    tests::fixture::installed_authorization_world, WorthQueryAdvancementDenial,
    WorthQueryManagedComputationResourceDenial,
};
#[test]
fn private_recovery_keeps_the_foreign_phase_cause() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
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
