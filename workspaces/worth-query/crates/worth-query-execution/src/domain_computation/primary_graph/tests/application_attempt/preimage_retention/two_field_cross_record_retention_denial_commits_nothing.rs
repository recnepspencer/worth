//! two field cross record retention denial commits nothing.
use super::*;

#[test]
fn two_field_cross_record_retention_denial_commits_nothing() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let unrelated = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let before = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            relational_head(&phase, &world)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiFieldRetentionOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        let open = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_owned())
                            .unwrap();
                        let unrelated = reader
                            .resolve_entity(AccountStatus::reference(), "unrelated".to_owned())
                            .unwrap();
                        reader
                            .require_decision_field(&open, AccountStatus::reference())
                            .unwrap();
                        reader
                            .require_decision_field(&unrelated, AccountLabel::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let reads = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            reads
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("the fixture read owns its installed host call");
    let open = effects.existing_entity(&account).unwrap();
    let unrelated = effects.existing_entity(&unrelated).unwrap();
    effects
        .write_field(&open, AccountStatus::reference(), "frozen".to_owned())
        .unwrap();
    effects
        .write_field(
            &unrelated,
            AccountLabel::reference(),
            "must-not-land".to_owned(),
        )
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(73, 74),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );

    assert_retention_denied(&outcome);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(relational_head(&phase, &world), before)
        })
        .expect("the fixture read owns its installed host call");
}
