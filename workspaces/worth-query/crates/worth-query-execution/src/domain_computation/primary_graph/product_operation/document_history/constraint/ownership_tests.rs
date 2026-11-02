//! A restriction's model ownership precedes the admission's authentication lapse.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenialKind as Kind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::domain_computation::primary_graph::tests::{
    application_attempt::{authenticated_principal, idempotency, resolved_account},
    fixture::{installed_authorization_world, live_scope, TouchAccountOperation},
};

impl WorthQueryDocumentHistoryOperationConstraint {
    pub(crate) fn assert_foreign_history_model_precedes_authentication_lapse(
        access: &WorthQueryProgramPublicationAccess,
    ) {
        for expired in [false, true] {
            let world = installed_authorization_world(true);
            let now = std::time::SystemTime::now();
            world.authorization_time.hold(now);
            let request = live_scope();
            world
                .application
                .with_host_advancement(|phase| {
                    let principal =
                        authenticated_principal(&phase.test_host_request(), &world, &request);
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
                    let binding = idempotency(190, 191);
                    let restriction = Self {
                        model: HistoryModelIdentity::from_resolved(&other),
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
                        admission.expire_authentication_for_test();
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
