use super::super::durable_publication::branch_history_publications::{
    changed_relations, create_relation_outcome, delete_relation_on_branch,
};
use super::super::proposal_records::create_relation;
use super::{admit_main, collect, selected};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::{
        BranchId, CommitId, RelationalAuthoredWriteDenial, RelationalAuthoredWriteSelector,
        RelationalDescriptiveTouch, RelationalTouchAdjacencyDirection as Direction,
    },
    identity::{EntityId, KindId, RelationId},
    runtime::RelationalRuntime,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key,
        definition::default_relation_aspects,
        publications::{begin_main_transaction, release_test_commit_snapshot},
    },
    durable_publication::memory_runtime as runtime_with_test_schema,
    ordinary_records::{create_entity, delete_entity},
    shared_relational_certification_test_execution_policy::serial_request,
};

mod audit;
mod paired;
mod recovery;

fn key(anchor: EntityId, direction: Direction) -> RelationalAuthoredWriteSelector {
    RelationalAuthoredWriteSelector::Adjacency {
        anchor,
        kind: KindId(2),
        direction,
    }
}

fn scan(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    source: &AdmittedRelationalBranchBasis,
    target: &AdmittedRelationalBranchBasis,
    anchor: EntityId,
    direction: Direction,
) -> Vec<CommitId> {
    let cursor = runtime
        .authored_write_history_cursor(
            execution,
            source,
            selected(runtime, target),
            &[key(anchor, direction)],
        )
        .unwrap();
    collect(runtime, execution, cursor).0
}

fn endpoints(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    relation: RelationId,
    source: EntityId,
    target: EntityId,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("authored-endpoints").push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(&execution, runtime).unwrap()
}

#[test]
fn entrants_moves_noops_and_removals_keep_both_full_generation_roles() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let a = create_entity(&execution, &runtime, "source");
    let b = create_entity(&execution, &runtime, "target");
    let c = create_entity(&execution, &runtime, "moved-target");
    let original = admit_main(&runtime).unwrap();
    let entrant = create_relation_outcome(&execution, &runtime, a, b, "entrant");
    let relation = changed_relations(&entrant)[0];
    let moved = endpoints(&runtime, execution, relation, a, c);
    let noop = endpoints(&runtime, execution, relation, a, c);
    let removed =
        delete_relation_on_branch(&execution, &runtime, relation, BranchId("main".into()));
    let target = admit_main(&runtime).unwrap();
    assert_eq!(
        scan(
            &runtime,
            execution,
            &original,
            &target,
            a,
            Direction::Outgoing
        ),
        [
            entrant.commit.commit_id,
            moved.commit.commit_id,
            noop.commit.commit_id,
            removed.commit.commit_id
        ]
    );
    assert_eq!(
        scan(
            &runtime,
            execution,
            &original,
            &target,
            b,
            Direction::Incoming
        ),
        [entrant.commit.commit_id, moved.commit.commit_id]
    );
    assert_eq!(
        scan(
            &runtime,
            execution,
            &original,
            &target,
            c,
            Direction::Incoming
        ),
        [
            moved.commit.commit_id,
            noop.commit.commit_id,
            removed.commit.commit_id
        ]
    );
    let wrong_generation = EntityId::new(
        a.partition_id,
        a.local_slot_value(),
        a.generation_value() + 1,
    );
    assert!(scan(
        &runtime,
        execution,
        &original,
        &target,
        wrong_generation,
        Direction::Outgoing
    )
    .is_empty());
    assert!(matches!(
        runtime.authored_write_history_cursor(
            execution,
            &original,
            selected(&runtime, &target),
            &[key(
                EntityId::new(a.partition_id, a.local_slot_value(), 0),
                Direction::Outgoing
            )]
        ),
        Err(RelationalAuthoredWriteDenial::UnsupportedAdjacencySelector)
    ));
    for outcome in [&entrant, &moved, &noop, &removed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn relation_fields_are_disjoint_but_cascade_retirement_is_an_occurrence() {
    use worth_foundational::facade::{
        AspectValue, ContractValidationInput, PortableAspectContractBasis,
        PortableAspectPatchOperation, PortableRecordAspectPatch,
    };
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let a = create_entity(&execution, &runtime, "source");
    let b = create_entity(&execution, &runtime, "target");
    let relation = create_relation(&execution, &runtime, a, b, "edge");
    let source = admit_main(&runtime).unwrap();
    let contract = default_relation_aspects()
        .into_iter()
        .map(|(_, contract)| contract)
        .find(|contract| contract.key() == &aspect_key("label"))
        .unwrap();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("disjoint-label").push(MutationIntent::Relation(
                RelationMutationIntent::ApplyAspectPatch(ApplyRelationAspectPatchIntent {
                    relation_id: relation,
                    aspect_patch: PortableRecordAspectPatch::new([
                        PortableAspectPatchOperation::SetWhole {
                            basis: PortableAspectContractBasis::from_contract(&contract),
                            value: ContractValidationInput::Scalar(AspectValue::String(
                                "new-label".into(),
                            )),
                        },
                    ]),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let edited = transaction.commit(&execution, &runtime).unwrap();
    assert!(runtime.replay().canonical_commit_envelope(&execution, edited.commit.commit_id).unwrap().unwrap()
        .descriptive_touches().exact_touches().unwrap().iter().any(|touch|
            matches!(touch, RelationalDescriptiveTouch::RelationMembership { relation: actual, .. } if *actual == relation)),
        "the actual field edit emits the descriptive membership effect that must not become an authored endpoint write");
    let after_field = admit_main(&runtime).unwrap();
    assert!(scan(
        &runtime,
        execution,
        &source,
        &after_field,
        a,
        Direction::Outgoing
    )
    .is_empty());
    let retired = delete_entity(&execution, &runtime, b);
    let target = admit_main(&runtime).unwrap();
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            a,
            Direction::Outgoing
        ),
        [retired.commit.commit_id]
    );
    for outcome in [&edited, &retired] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
