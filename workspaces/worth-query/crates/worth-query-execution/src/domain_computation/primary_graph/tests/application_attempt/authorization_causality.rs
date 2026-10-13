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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "revoked-authority-must-not-write",
            )
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            revoke_account_ownership(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");

    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(31, 31), crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation),
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::ProviderRejected
                && denial.stage() == WorthQueryApplicationCommitDenialStage::DecisionReadSet
    ));
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &live_scope())
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn distinct_abilities_sharing_one_policy_retain_exact_provider_cardinality() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
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
        .installed_operation(MultiTouchOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(admission.authorization_requirement_count(), 2);
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        reader
                            .require_decision_field(root, AccountStatus::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let mut reads = world
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
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            reads
                .observe_field(
                    &host_custody.borrowed_request_for_test(),
                    &account,
                    AccountStatus::reference(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = reads
        .complete(crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
        .begin_effect_program();
    let target = effects.existing_entity(&account).unwrap();
    effects
        .write_field(
            &target,
            AccountStatus::reference(),
            "two-requirements-committed".to_string(),
        )
        .unwrap();

    let outcome = world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(32, 32),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "complete two-ability decision set should commit: {outcome:?}",
    );
}

fn revoke_account_ownership(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    account: worth_relational::facade::identity::EntityId,
) {
    let graph = world.application.primary_provider.graph.clone();
    let relation_kind = graph
        .layout
        .relation(AccountOwner::reference().name())
        .expect("account ownership is installed")
        .kind;
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let relation = graph.with_runtime_mut(execution, |runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        runtime
            .read_truth()
            .visible_relations_of_kind(execution, relation_kind, snapshot.version_id())
            .expect("fixture read request remains live")
            .into_iter()
            .find(|record| record.target == account)
            .expect("the admitted account has one ownership edge")
            .relation_id
    });
    drop(selected);
    super::super::fixture::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("revoke-account-owner").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }),
        )),
    );
}
