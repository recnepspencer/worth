//! capability relation traversal does not enter sealed decision dependencies.
use super::*;

#[test]
fn capability_relation_traversal_does_not_enter_sealed_decision_dependencies() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = world
        .application
        .with_host_advancement(|phase| {
            authenticated(
                &phase,
                &(phase).borrowed_request_for_test(),
                &world,
                "alice",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_principal(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                1,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &actor,
                    &principal,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, principal| {
                        reader
                            .relations_from(AccountOwner::reference(), principal)
                            .unwrap();
                        let account = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_string())
                            .unwrap();
                        reader
                            .require_decision_field(&account, AccountStatus::reference())
                            .unwrap();
                    },
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let reads = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            reads
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("completion must re-observe only the explicitly sealed status dependency")
        })
        .expect("the fixture read owns its installed host call");
}
