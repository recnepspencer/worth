use crate::domain_computation::primary_graph::tests::application_attempt::touched_graph_closure::*;

#[test]
fn one_field_family_instance_cannot_satisfy_two_planned_entity_dependencies() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = authenticated_principal(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                "open",
                &request,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(MultiTouchOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                )
                .authorize_operation(
                    &host_custody.phase_for_application_for_test(&world.application),
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
                    &host_custody.phase_for_application_for_test(&world.application),
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
                    &host_custody.phase_for_application_for_test(&world.application),
                    admission,
                    projection,
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap();
            reads
                .observe_field(execution, &account, AccountStatus::reference())
                .unwrap();

            let denial = reads
                .complete(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                .err()
                .expect("one target-family instance cannot satisfy two exact planned facts");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch
            );
        })
        .expect("the fixture block owns its installed host call")
}
