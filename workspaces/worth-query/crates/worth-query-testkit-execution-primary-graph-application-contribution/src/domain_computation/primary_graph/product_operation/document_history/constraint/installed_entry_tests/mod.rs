//! History constraints retain real source/model facts and reject altered admission seals.
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial as Denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::document_history::model::HistoryModelIdentity;
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperationConstraint;
use worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind as Kind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as Refusal;
use worth_query_execution::facade::domain_computation::fact_verification_on_this_thread_for_test as fact_work;
use crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use crate::domain_computation::primary_graph::tests::application_attempt::idempotency;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation;
use worth_query_execution::test_read_stops::charged_prefixes_for_test;
use worth_query_execution::test_read_stops::consult_count_for_test;
use worth_query_execution::test_read_stops::observe_consults_for_test;

/// Test doors on a library type, owned by the moved courts.
pub trait InstalledEntryTestsWorthQueryDocumentHistoryOperationConstraintForTest: Sized {
    fn assert_foreign_installed_seal_is_stale_before_history_work(
        access: &WorthQueryProgramPublicationAccess,
    );
}

impl InstalledEntryTestsWorthQueryDocumentHistoryOperationConstraintForTest
    for WorthQueryDocumentHistoryOperationConstraint
{
    fn assert_foreign_installed_seal_is_stale_before_history_work(
        access: &WorthQueryProgramPublicationAccess,
    ) {
        for forged in [false, true] {
            let world = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
            let request = live_scope();
            world
                .application
                .with_host_advancement(|phase| {
                    let principal = authenticated_principal(
                        &phase.test_host_request_for_test(),
                        &world,
                        &request,
                    );
                    let account = resolved_account(
                        &phase.test_host_request_for_test(),
                        &world,
                        "open",
                        &request,
                    );
                    let operation = world
                        .application
                        .installed_schema()
                        .installed_operation(TouchAccountOperation::reference())
                        .unwrap();
                    let mut admission = world
                        .selected_product(&phase.test_host_request_for_test())
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
                    let restriction = WorthQueryDocumentHistoryOperationConstraint::new_for_test(
                        HistoryModelIdentity::from_resolved_for_test(&account),
                        world
                            .application
                            .product_runtime()
                            .admit_product_occurrence_for_test(
                                world.application.current_world().occurrence_for_test(),
                            )
                            .unwrap()
                            .read_lease_for_test(),
                        world.application.current_world(),
                        binding.bind_admission_for_test(&admission),
                        None,
                    );
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
