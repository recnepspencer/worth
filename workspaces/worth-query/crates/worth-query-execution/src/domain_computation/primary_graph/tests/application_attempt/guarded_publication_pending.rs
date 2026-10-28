//! A reserved publication slot that already names its binding answers guarded
//! custody before Relational: the exact binding is pending, a reused key with
//! another intent is drift.
use super::{
    authenticated_principal, idempotency, installed_authorization_world, live_scope,
    resolved_account, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody as Custody;

#[test]
fn reserved_slot_naming_the_exact_binding_is_publication_pending() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
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
                .selected_product(&(phase).test_host_request())
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
        .bind_operation(admission.operation_definition_identity())
        .bind_operation_scope(admission.operation_scope_binding())
        .bind_preconditions(admission.mutation_preconditions().identity())
        .bind_governed_input(admission.governed_input_identity())
        .bind_governed_proposal(admission.governed_proposal_identity());
    let reservation = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .primary_provider
                .reserve_application_publication_recovery(
                    world
                        .selected_product(&(phase).test_host_request())
                        .product()
                        .observation(),
                    Some(bound),
                )
                .expect("the owner reserves the exact product incarnation")
        })
        .expect("the fixture read owns its installed host call");
    let resolve = |binding| {
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
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
