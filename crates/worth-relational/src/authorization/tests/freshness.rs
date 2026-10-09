use crate::facade::history::BranchId;
use crate::identity::data::{KindId, PartitionId};
use crate::tests::support::{create_entity, create_relation, delete_relation_on_branch};
use crate::transactions::data::{
    AspectFieldPatch, CreateIntent, EntityReference, MutationIntent, RelationSpec,
    WorkerIntentBatch,
};

use super::{allow_plan, authorization_fixture, role_and_direct_path_plan};
use crate::authorization::RelationalAuthorizationObservationFreshness;

#[test]
fn dependency_collection_mechanism_does_not_change_authorization_meaning() {
    use crate::authorization::evidence::RelationalAuthorizationPathDependencies;
    use crate::authorization::{
        RelationalAuthorizationPathObservation, RelationalAuthorizationPathWitness,
    };

    let fixture = authorization_fixture();
    let witness = RelationalAuthorizationPathWitness::new(vec![fixture.principal, fixture.scope]);
    let reconstructive = RelationalAuthorizationPathObservation::new(
        true,
        Some(witness.clone()),
        RelationalAuthorizationPathDependencies {
            entities: vec![fixture.principal, fixture.scope],
            relations: vec![fixture.role_scope_relation],
            adjacency_lists: Vec::new(),
            fields: Vec::new(),
        },
        true,
    );
    let indexed = RelationalAuthorizationPathObservation::new(
        true,
        Some(witness),
        RelationalAuthorizationPathDependencies {
            entities: vec![fixture.scope],
            relations: Vec::new(),
            adjacency_lists: Vec::new(),
            fields: Vec::new(),
        },
        true,
    );

    assert!(reconstructive.has_same_decision_and_witness(&indexed));
}

#[test]
fn exact_authorization_observation_stales_after_membership_revocation() {
    let fixture = authorization_fixture();
    let admitted_snapshot = fixture.runtime.visibility_authority().snapshot();
    let admitted = fixture
        .runtime
        .observe_authorization(allow_plan(
            admitted_snapshot,
            fixture.principal,
            fixture.scope,
            [],
        ))
        .unwrap();
    let unchanged = fixture.runtime.visibility_authority().snapshot();
    assert_eq!(
        fixture
            .runtime
            .compare_authorization_observation(&admitted, unchanged),
        RelationalAuthorizationObservationFreshness::Fresh
    );

    let revoked = delete_relation_on_branch(
        &fixture.runtime,
        fixture.role_scope_relation,
        BranchId("main".to_string()),
    );
    assert_eq!(
        fixture
            .runtime
            .compare_authorization_observation(&admitted, revoked.snapshot.clone()),
        RelationalAuthorizationObservationFreshness::Stale
    );
}

#[test]
fn unrelated_relation_kind_does_not_widen_authorization_causality() {
    let fixture = authorization_fixture();
    let admitted_snapshot = fixture.runtime.visibility_authority().snapshot();
    let admitted = fixture
        .runtime
        .observe_authorization(allow_plan(
            admitted_snapshot,
            fixture.principal,
            fixture.scope,
            [],
        ))
        .unwrap();
    let unrelated = create_entity(&fixture.runtime, "unrelated-kind-target");
    create_relation_of_kind(&fixture.runtime, fixture.principal, unrelated, KindId(99));
    let current = fixture.runtime.visibility_authority().snapshot();

    assert_eq!(
        fixture
            .runtime
            .compare_authorization_observation(&admitted, current),
        RelationalAuthorizationObservationFreshness::Fresh
    );
}

#[test]
fn newly_matching_parallel_path_stales_the_exact_observation() {
    let fixture = authorization_fixture();
    let admitted_snapshot = fixture.runtime.visibility_authority().snapshot();
    let admitted = fixture
        .runtime
        .observe_authorization(role_and_direct_path_plan(
            admitted_snapshot,
            fixture.principal,
            fixture.scope,
        ))
        .unwrap();
    assert!(!admitted.paths()[1].matched());

    create_relation(
        &fixture.runtime,
        fixture.principal,
        fixture.scope,
        "newly-matching-deny",
    );
    let current = fixture.runtime.visibility_authority().snapshot();
    assert_eq!(
        fixture
            .runtime
            .compare_authorization_observation(&admitted, current),
        RelationalAuthorizationObservationFreshness::Stale
    );
}

fn create_relation_of_kind(
    runtime: &crate::runtime::RelationalRuntime,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
    kind_id: KindId,
) {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("unrelated-kind").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id,
                    client_key: crate::symbols::data::ClientKey::raw("unrelated-kind"),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
}
