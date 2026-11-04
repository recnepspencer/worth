//! A restriction's model ownership precedes the admission's authentication lapse.
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial as Denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::document_history::model::HistoryModelIdentity;
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperationConstraint;
use worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind as Kind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as Refusal;
use crate::domain_computation::primary_graph::tests::application_attempt::idempotency;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;

/// Test doors on a library type, owned by the moved courts.
pub trait OwnershipTestsWorthQueryDocumentHistoryOperationConstraintForTest: Sized {
    fn assert_foreign_history_model_precedes_authentication_lapse(
        access: &WorthQueryProgramPublicationAccess,
    );
}

impl OwnershipTestsWorthQueryDocumentHistoryOperationConstraintForTest
    for WorthQueryDocumentHistoryOperationConstraint
{
    fn assert_foreign_history_model_precedes_authentication_lapse(
        access: &WorthQueryProgramPublicationAccess,
    ) {
        for expired in [false, true] {
            let time = worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionTimeController::new().unwrap();
            time.hold();
            let world = installed_authorization_world(true, time.clock());
            let now = std::time::SystemTime::now();
            world.authorization_time.hold(now);
            let request = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(Some(time.clock().now() + std::time::Duration::from_secs(60)), worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new().token(), time.clock());
            let external = world.authenticate("alice", std::time::Duration::from_secs(1), &request);
            world
                .application
                .with_host_advancement(|phase| {
                    let principal =
                        world.selected_product(&phase.test_host_request_for_test()).resolve_authenticated_principal(&phase, &world.binding, &external, &request, worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
                    let account =
                        resolved_account(&phase.test_host_request_for_test(), &world, "open", &request);
                    let other =
                        resolved_account(&phase.test_host_request_for_test(), &world, "unrelated", &request);
                    assert_ne!(account.entity_id_for_test(), other.entity_id_for_test());
                    let operation = world
                        .application
                        .installed_schema()
                        .installed_operation(TouchAccountOperation::reference())
                        .unwrap();
                    let admission = world
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
                    let binding = idempotency(190, 191);
                    let restriction = Self::new_for_test(HistoryModelIdentity::from_resolved_for_test(&other), world
                            .application
                            .product_runtime()
                            .admit_product_occurrence_for_test(
                                world.application.current_world().occurrence_for_test(),
                            )
                            .unwrap()
                            .read_lease_for_test(), world.application.current_world(), binding.bind_admission_for_test(&admission), None);
                    admission
                        .validate_current_authority()
                        .expect("the genuine admission starts fresh");
                    if expired {
                        time.advance(std::time::Duration::from_secs(2)).unwrap();
                        assert_eq!(
                            admission.validate_current_authority().unwrap_err().kind(),
                            Kind::Refused(Refusal::ExpiredAuthentication)
                        );
                    } else {
                        admission.validate_current_authority().unwrap();
                    }
                    let result = world.application.validate_history_operation(
                        &phase,
                        access,
                        &restriction,
                        &admission,
                        binding,
                    );
                    assert!(
                        matches!(result, Err(Denial::ForeignModel)),
                        "model ownership precedes lapse: {result:?}"
                    );
                })
                .unwrap();
        }
    }
}
