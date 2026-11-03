//! History constraints retain real source/model facts and reject altered admission seals.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenialKind as Kind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::domain_computation::primary_graph::fact_verification_observation::fact_verification_on_this_thread_for_test as fact_work;
use crate::domain_computation::primary_graph::tests::{
    application_attempt::{authenticated_principal, idempotency, resolved_account},
    fixture::{
        installed_authorization_world, live_scope, ExactStatusRetentionOperation,
        TouchAccountOperation,
    },
};
use crate::facade::primary_graph::WorthQueryExecutionObservation;
use crate::test_read_stops::{
    charged_prefixes_for_test, consult_count_for_test, observe_consults_for_test,
};

impl WorthQueryDocumentHistoryOperationConstraint {
    pub(crate) fn assert_foreign_installed_seal_is_stale_before_history_work(
        access: &WorthQueryProgramPublicationAccess,
    ) {
        for forged in [false, true] {
            let world = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
            let request = live_scope();
            world
                .application
                .with_host_advancement(|phase| {
                    let principal =
                        authenticated_principal(&phase.test_host_request(), &world, &request);
                    let account =
                        resolved_account(&phase.test_host_request(), &world, "open", &request);
                    let operation = world
                        .application
                        .installed_schema()
                        .installed_operation(TouchAccountOperation::reference())
                        .unwrap();
                    let mut admission = world
                        .selected_product(&phase.test_host_request())
                        .authorize_operation(
                            &phase,
                            &principal,
                            &account,
                            &operation,
                            Default::default(),
                            &request,
                        )
                        .unwrap();
                    let binding = idempotency(185, 186);
                    let restriction = WorthQueryDocumentHistoryOperationConstraint {
                        operation_scope: HistoryOperationScope::from_resolved(&account),
                        source: world
                            .application
                            .product_runtime()
                            .admit_product_occurrence(
                                world.application.current_world().occurrence(),
                            )
                            .unwrap()
                            .read_lease(),
                        candidate: world.application.current_world(),
                        meaning: binding.bind_admission(&admission),
                        candidate_predecessor: None,
                    };
                    if forged {
                        let donor = world
                            .application
                            .installed_schema()
                            .installed_operation(ExactStatusRetentionOperation::reference())
                            .unwrap();
                        admission.forge_installed_seal_for_test(donor.installed_view());
                    }
                    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
                    fact_work();
                    let consults = observe_consults_for_test(|_| {});
                    let result = world.application.validate_history_operation(
                        &phase,
                        access,
                        &restriction,
                        &admission,
                        binding,
                    );
                    if forged {
                        let Err(Denial::OperationAdmission(denial)) = result else {
                            panic!("a stale installed seal must precede history work: {result:?}");
                        };
                        assert_eq!(
                            denial.kind(),
                            Kind::Refused(Refusal::StaleInstalledOperation)
                        );
                        assert_eq!(fact_work().source_admission_visits, 0);
                        assert_eq!(consult_count_for_test(&consults), 0);
                        assert!(charged_prefixes_for_test(&consults).is_empty());
                    } else {
                        result.expect("a genuinely matching model, source and admission validate");
                    }
                })
                .unwrap();
        }
    }
}
