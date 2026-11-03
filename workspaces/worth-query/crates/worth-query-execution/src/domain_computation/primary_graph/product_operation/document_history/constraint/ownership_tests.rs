//! A restriction's model ownership precedes the admission's authentication lapse.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenialKind as Kind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::domain_computation::primary_graph::tests::{
    application_attempt::{idempotency, resolved_account},
    fixture::{installed_authorization_world, TouchAccountOperation},
};

impl WorthQueryDocumentHistoryOperationConstraint {
    pub(crate) fn assert_foreign_history_model_precedes_authentication_lapse(
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
                        world.selected_product(&phase.test_host_request()).resolve_authenticated_principal(&phase, &world.binding, &external, &request, crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
                    let account =
                        resolved_account(&phase.test_host_request(), &world, "open", &request);
                    let other =
                        resolved_account(&phase.test_host_request(), &world, "unrelated", &request);
                    assert_ne!(account.entity_id(), other.entity_id());
                    let operation = world
                        .application
                        .installed_schema()
                        .installed_operation(TouchAccountOperation::reference())
                        .unwrap();
                    let admission = world
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
                    let binding = idempotency(190, 191);
                    let restriction = Self {
                        operation_scope: HistoryOperationScope::from_resolved(&other),
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
