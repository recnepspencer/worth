use worth_relational::facade::transactions::{
    DeleteRelationIntent, MutationIntent, RelationMutationIntent, WorkerIntentBatch,
};

use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountOwner, AccountStatus, AuthorizationWorld, MultiTouchOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

#[test]
fn ownership_revocation_after_admission_stales_before_effect_commit() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "revoked-authority-must-not-write",
    );

    revoke_account_ownership(&world, account.entity_id());

    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(31, 31)),
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::ProviderRejected
                && denial.stage() == WorthQueryApplicationCommitDenialStage::DecisionReadSet
    ));
    let _still_open = resolved_account(&world, "open", &live_scope());
}

#[test]
fn distinct_abilities_sharing_one_policy_retain_exact_provider_cardinality() {
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
    assert_eq!(admission.authorization_requirement_count(), 2);
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, root| {
            reader
                .require_decision_field(root, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let mut reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    reads
        .observe_field(&account, AccountStatus::reference())
        .unwrap();
    let mut effects = reads.complete().unwrap().begin_effect_program();
    let target = effects.existing_entity(&account).unwrap();
    effects
        .write_field(
            &target,
            AccountStatus::reference(),
            "two-requirements-committed".to_string(),
        )
        .unwrap();

    let outcome = world
        .application
        .compare_and_commit_application(effects.finish().unwrap(), idempotency(32, 32));
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "complete two-ability decision set should commit: {outcome:?}",
    );
}

fn revoke_account_ownership(
    world: &AuthorizationWorld,
    account: worth_relational::facade::identity::EntityId,
) {
    let graph = world.application.primary_provider.graph.clone();
    let relation_kind = graph
        .layout
        .relation(AccountOwner::reference().name())
        .expect("account ownership is installed")
        .kind;
    let selected = world.selected_product();
    let relation = graph.with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        runtime
            .read_truth()
            .visible_relations_of_kind(relation_kind, snapshot.version_id())
            .into_iter()
            .find(|record| record.target == account)
            .expect("the admitted account has one ownership edge")
            .relation_id
    });
    drop(selected);
    super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("revoke-account-owner").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }),
        )),
    );
}
