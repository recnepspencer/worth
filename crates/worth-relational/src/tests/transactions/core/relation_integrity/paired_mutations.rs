//! A balanced proposed new pair must not hide an abandoned old paired edge.
use crate::identity::data::EntityId;
use crate::schema::data::{
    RelationIntegrityDeclarations, SymmetryContractDeclaration, SymmetryMode,
};
use crate::tests::support::*;
use crate::transactions::data::{ConflictClass, EntityReference, RelationSpec};
use crate::validation::data::InvariantViolationFields;
use std::collections::BTreeMap;

fn relation(source: EntityId, target: EntityId, key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw(key),
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    }))
}

fn update(relation_id: RelationId, source: EntityId, target: EntityId) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target),
        },
    ))
}

fn commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    intents: Vec<MutationIntent>,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    let mut batch = WorkerIntentBatch::new("paired-mutation");
    for intent in intents {
        batch = batch.push(intent);
    }
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime)
}

fn committed(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> BTreeMap<RelationId, (EntityId, EntityId)> {
    let snapshot = snapshot_for_owner_branch(runtime, &BranchId("main".into()));
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let edges = read
        .relations()
        .iter()
        .map(|edge| (edge.relation_id, (edge.source, edge.target)))
        .collect();
    drop(read);
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    edges
}

fn paired_runtime(mode: SymmetryMode) -> RelationalRuntime {
    RelationIntegritySchemaFixture {
        relation_integrity: RelationIntegrityDeclarations::new(
            vec![],
            vec![],
            vec![],
            vec![SymmetryContractDeclaration {
                contract_id: "paired-update".into(),
                mode,
            }],
            vec![],
        ),
        ..RelationIntegritySchemaFixture::default()
    }
    .build_runtime()
}

#[test]
fn endpoint_updates_judge_the_abandoned_pair_and_balanced_twin_for_both_paired_modes() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    for mode in [
        SymmetryMode::PairedInverseRequired,
        SymmetryMode::PairedTwinRequired,
    ] {
        let runtime = paired_runtime(mode);
        let a = create_entity(execution, &runtime, "a");
        let b = create_entity(execution, &runtime, "b");
        let c = create_entity(execution, &runtime, "c");
        let seed = commit(
            execution,
            &runtime,
            vec![relation(a, b, "ab"), relation(b, a, "ba")],
        )
        .unwrap();
        release_test_commit_snapshot(&runtime, &seed);
        let before = committed(execution, &runtime);
        let forward = *before.iter().find(|(_, pair)| **pair == (a, b)).unwrap().0;
        let reverse = *before.iter().find(|(_, pair)| **pair == (b, a)).unwrap().0;

        // Both proposed new edges have a reverse. Only the abandoned B->A
        // edge exposes an evaluator that still judges planned edges alone.
        let failure = commit(
            execution,
            &runtime,
            vec![update(forward, a, c), relation(c, a, "new-reverse")],
        )
        .unwrap_err();
        let TransactionCommitError::Conflict { error, .. } = failure else {
            panic!("{failure:?}")
        };
        let ConflictClass::InvariantViolation {
            fields:
                InvariantViolationFields::RelationSymmetry {
                    source,
                    target,
                    mode: actual_mode,
                    ..
                },
            ..
        } = error.class
        else {
            panic!(
                "original native symmetry fields required: {:?}",
                error.class
            )
        };
        assert_eq!(
            (source, target),
            (EntityReference::Existing(b), EntityReference::Existing(a))
        );
        assert_eq!(actual_mode, mode);
        assert_eq!(committed(execution, &runtime), before);

        let balanced = commit(
            execution,
            &runtime,
            vec![update(forward, a, c), update(reverse, c, a)],
        )
        .unwrap();
        release_test_commit_snapshot(&runtime, &balanced);
        let after = committed(execution, &runtime);
        assert_eq!(after.len(), before.len());
        assert_eq!(after[&forward], (a, c));
        assert_eq!(after[&reverse], (c, a));
    }
}

#[test]
fn explicit_entity_and_relation_revalidation_reaches_native_paired_judgment() {
    use crate::transactions::data::{
        EntityMutationIntent, RevalidateEntityIntent, RevalidateRelationIntent,
    };
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = paired_runtime(SymmetryMode::PairedTwinRequired);
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let seed = commit(
        execution,
        &runtime,
        vec![relation(a, b, "ab"), relation(b, a, "ba")],
    )
    .unwrap();
    release_test_commit_snapshot(&runtime, &seed);
    let before = committed(execution, &runtime);
    let forward = *before.iter().find(|(_, pair)| **pair == (a, b)).unwrap().0;
    for intent in [
        MutationIntent::Relation(RelationMutationIntent::Revalidate(
            RevalidateRelationIntent {
                relation_id: forward,
            },
        )),
        MutationIntent::Entity(EntityMutationIntent::Revalidate(RevalidateEntityIntent {
            entity_id: a,
        })),
    ] {
        let outcome = commit(execution, &runtime, vec![intent]).unwrap();
        // This is reach evidence from actual checks, not an exact cost or work allowance.
        assert!(outcome.complexity_delta().relation_symmetry_checks > 0);
        release_test_commit_snapshot(&runtime, &outcome);
        assert_eq!(committed(execution, &runtime), before);
    }
}
