use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::assert_membership_absent;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::authenticated;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::link_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::resolved_principal;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::AccountOwner;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::AccountStatus;
use crate::domain_computation::primary_graph::tests::application_attempt::decision_adjacency::ChangeOwnershipOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;

#[test]
fn reused_relation_key_is_denied_during_public_authoring() {
    let world = installed_authorization_world(false);
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
            let host_custody = phase.test_host_request_for_test();
            resolved_principal(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                1,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let open = world
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
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
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
                            .decision_relations_from(AccountOwner::reference(), principal)
                            .unwrap();
                        let account = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_owned())
                            .unwrap();
                        reader
                            .require_decision_field(&account, AccountStatus::reference())
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
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = reads.begin_effect_program();
    let from = effects.existing_entity(&principal).unwrap();
    let open = effects.existing_entity(&open).unwrap();
    effects
        .link(AccountOwner::reference(), "same-key", &from, &open)
        .unwrap();
    let denial = effects
        .link(AccountOwner::reference(), "same-key", &from, &open)
        .expect_err("public authoring must reject a repeated relation key");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey
    );
}

#[test]
fn two_real_relation_deletes_share_one_ordered_provisional_step_through_commit() {
    {
        let world = installed_authorization_world(false);
        let request = live_scope();

        let actor = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .borrowed_request_for_test(),
                    &world,
                    "alice",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");

        for (status, key, caller) in [("open", "owner-open", 90), ("unrelated", "owner-other", 91)]
        {
            let principal = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    resolved_principal(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &host_custody.borrowed_request_for_test(),
                        &world,
                        1,
                        &request,
                    )
                })
                .expect("the fixture read owns its installed host call");
            let account = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    resolved_account(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                        &world,
                        status,
                        &request,
                    )
                })
                .expect("the fixture read owns its installed host call");
            let program = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    link_program(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &world,
                        &actor,
                        &principal,
                        &account,
                        &request,
                        status,
                        key,
                    )
                })
                .expect("the fixture read owns its installed host call");
            assert!(matches!(
                world.application.compare_and_commit_application(
                    program,
                    idempotency(caller, caller),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation
                ),
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }

        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                resolved_principal(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    1,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let open = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                resolved_account(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    "open",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let other = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                resolved_account(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    "unrelated",
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
                let host_custody = phase.test_host_request_for_test();
                world
                    .selected_product(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                    )
                    .authorize_operation(
                        &host_custody.phase_for_application_for_test(&world.application),
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
                let host_custody = phase.test_host_request_for_test();
                world
                    .invariant
                    .project_admitted_operation(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &admission,
                        |reader, principal| {
                            reader
                                .decision_relations_from(AccountOwner::reference(), principal)
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
                let host_custody = phase.test_host_request_for_test();
                world
                    .application
                    .begin_projected_application_read_attempt(
                        &host_custody.phase_for_application_for_test(&world.application),
                        admission,
                        projection,
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .complete_projected_dependencies(
                        &host_custody.phase_for_application_for_test(&world.application),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let mut effects = reads.begin_effect_program();
        let from = effects.existing_entity(&principal).unwrap();
        let open = effects.existing_entity(&open).unwrap();
        let other = effects.existing_entity(&other).unwrap();
        effects
            .unlink_observed_for_test(AccountOwner::reference(), &from, &open)
            .unwrap();
        effects
            .unlink_observed_for_test(AccountOwner::reference(), &from, &other)
            .unwrap();
        let program = effects.finish().unwrap();

        let WorthQueryApplicationCommitOutcome::Committed(receipt) =
            world.application.compare_and_commit_application(
                program,
                idempotency(92, 92),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        else {
            panic!("two observed relation deletions must reach one committed attempt");
        };
        let work = receipt.mutation_work().unwrap();
        assert_eq!(work.expected_step_key_lookups(), 2);
        assert_eq!(work.expected_step_duplicate_equalities(), 1);
        assert_eq!(work.proposed_fact_count(), 1);
        assert_eq!(work.touched_record_count(), 3);
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                assert_membership_absent(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    &actor,
                    &principal,
                    &resolved_account(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                        &world,
                        "open",
                        &request,
                    ),
                    &request,
                    "open",
                )
            })
            .expect("the fixture read owns its installed host call");
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                assert_membership_absent(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    &actor,
                    &principal,
                    &resolved_account(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                        &world,
                        "unrelated",
                        &request,
                    ),
                    &request,
                    "unrelated",
                )
            })
            .expect("the fixture read owns its installed host call");
    }
}
