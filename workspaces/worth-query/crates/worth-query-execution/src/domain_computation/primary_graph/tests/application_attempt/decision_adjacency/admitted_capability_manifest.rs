use super::*;

#[test]
fn compile_capability_cannot_widen_the_installed_relation_manifest() {
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
                    &actor,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let projected = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, account| {
                        reader.decision_relations_to(AccountOwner::reference(), account)
                    },
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let denial = projected
        .output()
        .as_ref()
        .expect_err("an uninstalled relation target must be denied");

    assert_eq!(
        denial.kind(),
        WorthQueryInvariantProjectionTraversalDenialKind::UndeclaredDecisionTarget
    );
}
