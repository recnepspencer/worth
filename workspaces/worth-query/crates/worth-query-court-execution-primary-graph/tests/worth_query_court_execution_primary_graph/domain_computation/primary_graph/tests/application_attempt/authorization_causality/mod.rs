use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::MultiTouchOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

#[test]
fn ownership_revocation_after_admission_stales_before_effect_commit() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
            let host_custody = phase.test_host_request_for_test();
            revoke_account_ownership(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id_for_test(),
            )
        })
        .expect("the fixture read owns its installed host call");

    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, idempotency(31, 31), worth_execution::ExecutionAllocationPolicy::SystemAllocation),
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::ProviderRejected
                && denial.stage() == WorthQueryApplicationCommitDenialStage::DecisionReadSet
    ));
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn distinct_abilities_sharing_one_policy_retain_exact_provider_cardinality() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiTouchOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
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
                            .require_decision_field(root, AccountStatus::at("AccountStatus"))
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let completed = world
        .application
        .with_host_advancement(|phase| {
            let mut reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let host_custody = phase.test_host_request_for_test();
            reads
                .observe_field(
                    &host_custody.borrowed_request_for_test(),
                    &account,
                    AccountStatus::at("AccountStatus"),
                )
                .unwrap();
            reads
                .complete(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = completed.begin_effect_program();
    let target = effects.existing_entity(&account).unwrap();
    effects
        .write_field(
            &target,
            AccountStatus::at("AccountStatus"),
            "two-requirements-committed".to_string(),
        )
        .unwrap();

    let outcome = world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(32, 32),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "complete two-ability decision set should commit: {outcome:?}",
    );
}

pub fn revoke_account_ownership(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    account: worth_relational::facade::identity::EntityId,
) {
    let graph = (*(*(*world.application.runtime_for_test()).primary_provider_for_test())
        .graph_for_test())
    .clone();
    let relation_kind = *(*graph.layout_for_test())
        .relation(AccountOwner::reference().name())
        .expect("account ownership is installed")
        .kind_for_test();
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let relation = graph.with_runtime_mut_for_test(execution, |runtime| {
        let snapshot = selected.application_basis_for_test().snapshot_handle();
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
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("revoke-account-owner").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }),
        )),
    );
}
