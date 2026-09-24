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
    let actor = authenticated(&world, "alice", &request);
    let principal = resolved_principal(&world, 1, &request);
    let open = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &principal, &operation, Default::default(), &request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, principal| {
            reader
                .decision_relations_from(AccountOwner::reference(), principal)
                .unwrap();
            let account = reader
                .resolve_entity(AccountStatus::reference(), "open".to_owned())
                .unwrap();
            reader
                .require_decision_field(&account, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap();
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
    let world = installed_authorization_world(false);
    let request = live_scope();
    let actor = authenticated(&world, "alice", &request);

    for (status, key, caller) in [("open", "owner-open", 90), ("unrelated", "owner-other", 91)] {
        let principal = resolved_principal(&world, 1, &request);
        let account = resolved_account(&world, status, &request);
        let program = link_program(&world, &actor, &principal, &account, &request, status, key);
        assert!(matches!(
            world
                .application
                .compare_and_commit_application(program, idempotency(caller, caller)),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }

    let principal = resolved_principal(&world, 1, &request);
    let open = resolved_account(&world, "open", &request);
    let other = resolved_account(&world, "unrelated", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &principal, &operation, Default::default(), &request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, principal| {
            reader
                .decision_relations_from(AccountOwner::reference(), principal)
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
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

    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(program, idempotency(92, 92))
    else {
        panic!("two observed relation deletions must reach one committed attempt");
    };
    let work = receipt.mutation_work().unwrap();
    assert_eq!(work.expected_step_key_lookups(), 2);
    assert_eq!(work.expected_step_duplicate_equalities(), 1);
    assert_eq!(work.proposed_fact_count(), 1);
    assert_eq!(work.touched_record_count(), 3);
    assert_membership_absent(
        &world,
        &actor,
        &principal,
        &resolved_account(&world, "open", &request),
        &request,
        "open",
    );
    assert_membership_absent(
        &world,
        &actor,
        &principal,
        &resolved_account(&world, "unrelated", &request),
        &request,
        "unrelated",
    );
}
