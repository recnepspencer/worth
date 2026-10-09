use super::*;

#[test]
fn compile_capability_cannot_widen_the_installed_relation_manifest() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = authenticated(&world, "alice", &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &account, &operation, Default::default(), &request)
        .unwrap();
    let projected = world
        .invariant
        .project_admitted_operation(
            &admission,
            |reader, account| reader.decision_relations_to(AccountOwner::reference(), account),
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let denial = projected
        .output()
        .as_ref()
        .expect_err("an uninstalled relation target must be denied");

    assert_eq!(
        denial.kind(),
        WorthQueryInvariantProjectionTraversalDenialKind::UndeclaredDecisionTarget
    );
}
