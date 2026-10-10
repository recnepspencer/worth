use super::{
    assert_membership_absent, authenticated, idempotency, installed_authorization_world,
    link_program, live_scope, resolved_account, resolved_principal, AccountOwner, AccountStatus,
    ChangeOwnershipOperation, WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

#[test]
fn reused_relation_key_is_denied_during_public_authoring() {
    let world = installed_authorization_world(false);
    let request = live_scope();
    let actor = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
    let open = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let world = installed_authorization_world(false);
        let request = live_scope();
        let actor = authenticated(
            &host_custody.phase_for_application(&world.application),
            &(host_custody.phase_for_application(&world.application)).borrowed_request_for_test(),
            &world,
            "alice",
            &request,
        );

        for (status, key, caller) in [("open", "owner-open", 90), ("unrelated", "owner-other", 91)]
        {
            let principal = resolved_principal(
                &host_custody.phase_for_application(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                1,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                status,
                &request,
            );
            let program = link_program(
                &host_custody.phase_for_application(&world.application),
                &world,
                &actor,
                &principal,
                &account,
                &request,
                status,
                key,
            );
            assert!(matches!(
                world.application.compare_and_commit_application(
                    program,
                    idempotency(caller, caller),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
                ),
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }

        let principal = resolved_principal(
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            1,
            &request,
        );
        let open = resolved_account(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            "open",
            &request,
        );
        let other = resolved_account(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            "unrelated",
            &request,
        );
        let operation = world
            .application
            .installed_schema()
            .installed_operation(ChangeOwnershipOperation::reference())
            .unwrap();
        let admission = world
            .selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            )
            .authorize_operation(
                &host_custody.phase_for_application(&world.application),
                &actor,
                &principal,
                &operation,
                Default::default(),
                &request,
            )
            .unwrap();
        let (_, projection, _) = world
            .invariant
            .project_admitted_operation(
                &host_custody.phase_for_application(&world.application),
                &admission,
                |reader, principal| {
                    reader
                        .decision_relations_from(AccountOwner::reference(), principal)
                        .unwrap();
                },
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .into_parts();
        let reads = world
            .application
            .begin_projected_application_read_attempt(
                &host_custody.phase_for_application(&world.application),
                admission,
                projection,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .complete_projected_dependencies(
                &host_custody.phase_for_application(&world.application),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let mut effects = reads.begin_effect_program();
        let from = effects.existing_entity(&principal).unwrap();
        let open = effects.existing_entity(&open).unwrap();
        let other = effects.existing_entity(&other).unwrap();
        effects
            .unlink_observed(AccountOwner::reference(), &from, &open)
            .unwrap();
        effects
            .unlink_observed(AccountOwner::reference(), &from, &other)
            .unwrap();
        let program = effects.finish().unwrap();

        let WorthQueryApplicationCommitOutcome::Committed(receipt) =
            world.application.compare_and_commit_application(
                program,
                idempotency(92, 92),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        else {
            panic!("two observed relation deletions must reach one committed attempt");
        };
        let work = receipt.mutation_work().unwrap();
        assert_eq!(work.expected_step_key_lookups(), 2);
        assert_eq!(work.expected_step_duplicate_equalities(), 1);
        assert_eq!(work.proposed_fact_count(), 1);
        assert_eq!(work.touched_record_count(), 3);
        assert_membership_absent(
            &host_custody.phase_for_application(&world.application),
            &world,
            &actor,
            &principal,
            &resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "open",
                &request,
            ),
            &request,
            "open",
        );
        assert_membership_absent(
            &host_custody.phase_for_application(&world.application),
            &world,
            &actor,
            &principal,
            &resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "unrelated",
                &request,
            ),
            &request,
            "unrelated",
        );
    })
}
