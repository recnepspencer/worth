use super::*;

#[test]
fn one_field_family_instance_cannot_satisfy_two_planned_entity_dependencies() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiTouchOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &admission,
            |reader, _| {
                let open = reader
                    .resolve_entity(AccountStatus::reference(), "open".to_string())
                    .unwrap();
                let unrelated = reader
                    .resolve_entity(AccountStatus::reference(), "unrelated".to_string())
                    .unwrap();
                reader
                    .require_decision_field(&open, AccountStatus::reference())
                    .unwrap();
                reader
                    .require_decision_field(&unrelated, AccountStatus::reference())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let mut reads = world
        .application
        .begin_projected_application_read_attempt(
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    reads
        .observe_field(&account, AccountStatus::reference())
        .unwrap();

    let denial = reads
        .complete(crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
        .err()
        .expect("one target-family instance cannot satisfy two exact planned facts");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch
    );
}
