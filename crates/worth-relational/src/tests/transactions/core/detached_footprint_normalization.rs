use crate::facade::mvcc::{RelationalTransactionReadLocus, RelationalTransactionWriteLocus};
use crate::facade::transactions::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityReference, MutationIntent,
    RelationSpec, WorkerIntentBatch,
};
use crate::tests::support::*;

#[test]
fn require_interned_normalizes_plan_and_created_footprint_to_identical_symbols() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    let raw_source = created_entity("normalized-source");
    let raw_target = created_entity("normalized-target");
    let raw_relation = CreatedRelationRef {
        partition_id: crate::facade::identity::PartitionId::main(),
        kind_id: crate::facade::identity::KindId(2),
        client_key: crate::facade::symbols::ClientKey::raw("normalized-relation"),
        source: EntityReference::Created(raw_source.clone()),
        target: EntityReference::Created(raw_target.clone()),
    };
    transaction
        .push_batch(
            batch_create("normalized-source"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            batch_create("normalized-target"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            WorkerIntentBatch::new("normalized-relation-batch").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: raw_relation.partition_id,
                    kind_id: raw_relation.kind_id,
                    client_key: raw_relation.client_key.clone(),
                    source: raw_relation.source.clone(),
                    target: raw_relation.target.clone(),
                    fields: Default::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    assert!(transaction
        .read_created_entity(&raw_source)
        .unwrap()
        .is_some());
    assert!(transaction
        .read_created_entity(&raw_target)
        .unwrap()
        .is_some());
    assert!(transaction
        .read_created_relation(&raw_relation)
        .unwrap()
        .is_some());

    let merged_intents = transaction
        .merged_plan(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("raw created graph normalizes")
        .merged_intents
        .clone();
    let normalized_source = normalized_entity(
        &merged_intents,
        &runtime.services.symbols.interner_snapshot(),
        "normalized-source",
    );
    let normalized_target = normalized_entity(
        &merged_intents,
        &runtime.services.symbols.interner_snapshot(),
        "normalized-target",
    );
    let normalized_relation = normalized_relation(&merged_intents);
    assert!(transaction
        .read_created_entity(&raw_source)
        .unwrap()
        .is_some());
    assert!(transaction
        .read_created_entity(&raw_target)
        .unwrap()
        .is_some());
    assert!(transaction
        .read_created_relation(&raw_relation)
        .unwrap()
        .is_some());

    assert_eq!(
        normalized_relation.source,
        EntityReference::Created(normalized_source.clone())
    );
    assert_eq!(
        normalized_relation.target,
        EntityReference::Created(normalized_target.clone())
    );
    for (key, expected) in [
        (&normalized_source.client_key, "normalized-source"),
        (&normalized_target.client_key, "normalized-target"),
        (&normalized_relation.client_key, "normalized-relation"),
    ] {
        let symbol = key.as_symbol().expect("RequireInterned emits symbols");
        assert_eq!(
            runtime.services.symbols.resolve(symbol).as_deref(),
            Some(expected)
        );
        assert!(runtime
            .config()
            .identity
            .symbol_table
            .entries
            .contains(&(symbol, expected.to_owned())));
    }

    let expected_reads = std::collections::BTreeSet::from([
        RelationalTransactionReadLocus::CreatedEntity(normalized_source.clone()),
        RelationalTransactionReadLocus::CreatedEntity(normalized_target.clone()),
        RelationalTransactionReadLocus::CreatedRelation(normalized_relation.clone()),
    ]);
    let expected_writes = std::collections::BTreeSet::from([
        RelationalTransactionWriteLocus::CreatedEntity(normalized_source),
        RelationalTransactionWriteLocus::CreatedEntity(normalized_target),
        RelationalTransactionWriteLocus::CreatedRelation(normalized_relation),
    ]);
    assert_eq!(
        transaction
            .footprint()
            .reads()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected_reads
    );
    assert_eq!(
        transaction
            .footprint()
            .writes()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected_writes
    );
}

fn created_entity(client_key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: crate::facade::identity::PartitionId::main(),
        kind_id: crate::facade::identity::KindId(1),
        client_key: crate::facade::symbols::ClientKey::raw(client_key),
    }
}

fn normalized_entity(
    intents: &[MutationIntent],
    symbols: &crate::symbols::data::StringInterner,
    expected: &str,
) -> CreatedEntityRef {
    intents
        .iter()
        .find_map(|intent| match intent {
            MutationIntent::Create(CreateIntent::Entity(entity))
                if entity.client_key.resolve_with_interner(symbols) == Some(expected) =>
            {
                Some(CreatedEntityRef {
                    partition_id: entity.partition_id,
                    kind_id: entity.kind_id,
                    client_key: entity.client_key.clone(),
                })
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("normalized entity for {expected}"))
}

fn normalized_relation(intents: &[MutationIntent]) -> CreatedRelationRef {
    intents
        .iter()
        .find_map(|intent| match intent {
            MutationIntent::Create(CreateIntent::Relation(relation)) => Some(CreatedRelationRef {
                partition_id: relation.partition_id,
                kind_id: relation.kind_id,
                client_key: relation.client_key.clone(),
                source: relation.source.clone(),
                target: relation.target.clone(),
            }),
            _ => None,
        })
        .expect("normalized relation intent")
}
