//! Presence and absence from the actual admitted snapshot remain decision facts.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_operation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_with_expected_status;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_changed_decision;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountNote;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::PatchAccountDraftInput;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PatchAccountDraftOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use std::cell::Cell;

#[test]
fn present_predecode_field_stales_after_a_competing_actual_change() {
    let world = installed_authorization_world(true);
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
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_operation(&phase, &world, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (_, snapshot, work) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        let value = reader
                            .decision_field_with_predecode_admission_for_test(
                                root,
                                AccountStatus::reference(),
                                |_, _| Ok::<_, ()>(()),
                                &|| Ok(()),
                            )
                            .unwrap()
                            .unwrap();
                        assert_eq!(value.as_deref(), Some("open"));
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(work.field_reads(), 1);
    let mut effects = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    snapshot,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("the fixture read owns its installed host call");
    let target = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&target, AccountStatus::reference(), "loser".to_owned())
        .unwrap();
    let loser = effects.finish().unwrap();
    let winner = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_expected_status(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                ("open", "winner"),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            winner,
            idempotency(201, 201),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_changed_decision(
        world.application.compare_and_commit_application(
            loser,
            idempotency(202, 202),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        "the admitted predecode source field changed",
    );
}

#[test]
fn absence_skips_admission_and_stales_after_a_competing_presence_change() {
    let world = installed_authorization_world(true);
    let called = Cell::new(false);
    let loser = world
        .application
        .with_host_advancement(|phase| optional_program(&phase, &world, "loser", &called))
        .expect("the fixture read owns its installed host call");
    assert!(!called.get(), "absence never invokes scalar admission");
    let winner = world
        .application
        .with_host_advancement(|phase| optional_program(&phase, &world, "winner", &called))
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            winner,
            idempotency(203, 203),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_changed_decision(
        world.application.compare_and_commit_application(
            loser,
            idempotency(204, 204),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        "the predecode-observed absent field became present",
    );
}

pub fn optional_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    note: &str,
    called: &Cell<bool>,
) -> WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    PatchAccountDraftOperation,
    PatchAccountDraftInput,
    Account,
> {
    let request = live_scope();
    let principal = authenticated_principal(
        &(phase_custody).test_host_request_for_test(),
        world,
        &request,
    );
    let account = resolved_account(
        &(phase_custody).test_host_request_for_test(),
        world,
        "unrelated",
        &request,
    );
    let operation = world
        .application
        .installed_schema()
        .installed_operation(PatchAccountDraftOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .authorize_operation(
            phase_custody,
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, snapshot, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, root| {
                let value = reader
                    .decision_field_with_predecode_admission_for_test(
                        root,
                        AccountNote::reference(),
                        |_, _| {
                            called.set(true);
                            Ok::<_, ()>(())
                        },
                        &|| Ok(()),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(value, None);
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            snapshot,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            phase_custody,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let target = effects.existing_entity(&account).unwrap();
    effects
        .write_optional_field(&target, AccountNote::reference(), Some(note.to_owned()))
        .unwrap();
    effects.finish().unwrap()
}
