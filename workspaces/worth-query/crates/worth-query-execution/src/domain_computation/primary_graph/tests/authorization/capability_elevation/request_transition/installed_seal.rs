//! Consuming lifecycle readmission must run its own installed-seal gate.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use crate::test_read_stops::{
    charged_prefixes_for_test, consult_count_for_test, observe_consults_for_test,
};

#[test]
fn foreign_installed_seal_is_stale_at_elevation_request_readmission_before_work() {
    for forged in [false, true] {
        let world = request_world();
        let request = live_scope();
        let mut program = world
            .application
            .with_host_advancement(|phase| {
                let principal =
                    authenticated_principal(&phase.test_host_request(), &world, &request);
                request_reads(
                    &phase.test_host_request(),
                    &world,
                    &principal,
                    &request,
                    honest_input(),
                )
                .materialize_elevation_request_program()
                .unwrap()
            })
            .unwrap();
        if forged {
            let donor = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            program.forge_installed_seal_for_test(donor.installed_view());
        }
        let _observation =
            crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        let consults = observe_consults_for_test(|_| {});
        let result = world
            .application
            .readmit_elevation_request_installation(None, program);
        if forged {
            let Err(WorthQueryElevationRequestOutcome::Denied(denial)) = result else {
                panic!("a stale installed seal refuses the consuming lifecycle entry");
            };
            assert_eq!(
                denial.authorization_denial_kind(),
                Some(WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::StaleInstalledOperation
                ))
            );
            assert_eq!(consult_count_for_test(&consults), 0);
            assert!(charged_prefixes_for_test(&consults).is_empty());
        } else {
            assert!(
                result.is_ok(),
                "the real lifecycle binding survives fresh readmission"
            );
        }
    }
}
