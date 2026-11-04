//! A reserved publication slot that already names its binding answers guarded
//! custody before Relational: the exact binding is pending, a reused key with
//! another intent is drift.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountOperation;
use worth_query_execution::publication_boundary::workflow_advance::WorthQueryGuardedWorkflowOperationCustody as Custody;

#[test]
fn reserved_slot_naming_the_exact_binding_is_publication_pending() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let transition = [87; 32];
    let binding = idempotency(88, 89).bind_guarded_workflow_effect(&transition);
    // The owner binds the caller's key to the admitted operation before it
    // reads custody; the slot records that bound identity.
    let bound = binding
        .bind_operation(admission.operation_definition_identity_for_test())
        .bind_operation_scope(admission.operation_scope_binding())
        .bind_preconditions(admission.mutation_preconditions().identity_for_test())
        .bind_governed_input(admission.governed_input_identity_for_test())
        .bind_governed_proposal_for_test(admission.governed_proposal_identity_for_test());
    let reservation = world
        .application
        .with_host_advancement(|phase| {
            (*(*world.application.runtime_for_test()).primary_provider_for_test())
                .reserve_application_publication_recovery_for_test(
                    world
                        .selected_product(&(phase).test_host_request_for_test())
                        .product()
                        .observation_for_test(),
                    Some(bound),
                )
                .expect("the owner reserves the exact product incarnation")
        })
        .expect("the fixture read owns its installed host call");
    let resolve = |binding| {
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .application
                    .resolve_admitted_guarded_workflow_operation_custody(
                        &host_custody.borrowed_request_for_test(),
                        &admission,
                        binding,
                        &transition,
                        None,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call")
    };
    assert!(matches!(resolve(binding), Custody::PublicationPending));
    let drift = idempotency(88, 90).bind_guarded_workflow_effect(&transition);
    assert!(matches!(resolve(drift), Custody::IntentDrift));
    // A guarded effect is keyed by its transition, so another client key for
    // the same transition reuses the reserved key with another intent.
    let other = idempotency(91, 89).bind_guarded_workflow_effect(&transition);
    let other = resolve(other);
    assert!(matches!(other, Custody::IntentDrift), "{other:?}");
    drop(reservation);
    assert!(matches!(resolve(binding), Custody::Unseen));
}
