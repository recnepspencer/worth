//! Actual installed invocation aliases preserve the original Query service.
use super::*;
use crate::domain_computation::primary_graph::{
    tests::fixture::{self, AccountStatus},
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn erased_slot_and_active_handler_aliases_refuse_while_the_same_query_serves() {
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut world = fixture::installed_authorization_world(true);
    let scope = fixture::live_scope();
    let original =
        Arc::as_ptr(&world.application.mutation_handlers.entries[Binding::IDENTITY].value);
    let alias = world.application.mutation_handlers.entries[Binding::IDENTITY]
        .value
        .clone();
    assert!(matches!(
        world
            .application
            .mutation_handlers
            .preflight_home_retirement(&execution),
        Err(Failure::Handler {
            binding: Binding::IDENTITY,
            cause: Denial::Busy
        })
    ));
    let resolve = |world: &fixture::AuthorizationWorld| {
        world
            .application
            .with_host_advancement(|phase| {
                world
                    .selected_product(&phase.test_host_request())
                    .resolve_entity(
                        &phase,
                        AccountStatus::reference(),
                        "open".to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .expect("the same original Query resolves its account after refusal")
            })
            .unwrap()
    };
    let before = resolve(&world);
    drop(alias);
    let alias = world
        .application
        .mutation_handler_for_attempt::<Binding>()
        .unwrap();
    assert!(matches!(
        world
            .application
            .mutation_handlers
            .preflight_home_retirement(&execution),
        Err(Failure::Handler {
            binding: Binding::IDENTITY,
            cause: Denial::Busy
        })
    ));
    let after = resolve(&world);
    assert_eq!(before.entity_id(), after.entity_id());
    assert!(std::ptr::addr_eq(
        original,
        Arc::as_ptr(&world.application.mutation_handlers.entries[Binding::IDENTITY].value)
    ));
    drop(alias);
    world
        .application
        .mutation_handlers
        .preflight_home_retirement(&execution)
        .unwrap();
    let final_observation = resolve(&world);
    assert_eq!(before.entity_id(), final_observation.entity_id());
}
