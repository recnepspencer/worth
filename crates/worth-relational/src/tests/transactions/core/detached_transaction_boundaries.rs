use crate::facade::history::BranchId;
use crate::facade::mvcc::{RelationalTransactionReadLocus, RelationalTransactionWriteLocus};
use crate::facade::transactions::{
    BulkEntityCreateIntent, CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityReference,
    MutationIntent, RelationMutationIntent, RelationSpec, UpdateRelationEndpointsIntent,
    WorkerIntentBatch,
};
use crate::tests::support::*;

#[test]
fn relation_overlays_and_validation_footprints_stay_exact_basis_local() {
    let mut runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "relation-source");
    let target = create_entity(&runtime, "relation-target");
    let alternate_target = create_entity(&runtime, "alternate-relation-target");
    let relation = create_relation(&runtime, source, target, "root-edge");
    fork_from_main(&mut runtime, "storm");
    fork_from_main(&mut runtime, "maintenance");

    let (storm_basis, mut storm) = begin_on(&runtime, "storm");
    let (maintenance_basis, mut maintenance) = begin_on(&runtime, "maintenance");
    let storm_root_read = storm.read_relation(relation).expect("root read projects");
    assert_eq!(
        storm_root_read.base().map(|record| record.relation_id),
        Some(relation)
    );
    assert!(storm_root_read.staged_mutations().is_empty());
    assert_eq!(
        storm.read_relation(relation).expect("repeat read projects"),
        storm_root_read
    );

    let storm_relation_mutation =
        RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
            relation_id: relation,
            kind_id: crate::facade::identity::KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(alternate_target),
        });
    storm
        .push_batch(
            WorkerIntentBatch::new("storm-relation-update")
                .push(MutationIntent::Relation(storm_relation_mutation.clone())),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    assert_eq!(
        storm
            .read_relation(relation)
            .expect("staged relation read projects")
            .effective()
            .map(|record| record.target.clone()),
        Some(EntityReference::Existing(alternate_target))
    );
    assert_eq!(
        storm
            .read_relation(relation)
            .expect("staged relation read projects")
            .staged_mutations(),
        &[storm_relation_mutation]
    );
    let maintenance_read = maintenance
        .read_relation(relation)
        .expect("maintenance relation read projects");
    assert!(maintenance_read.staged_mutations().is_empty());
    assert_eq!(
        maintenance_read
            .effective()
            .map(|record| record.target.clone()),
        Some(EntityReference::Existing(target))
    );

    let created = CreatedRelationRef {
        partition_id: crate::facade::identity::PartitionId::main(),
        kind_id: crate::facade::identity::KindId(2),
        client_key: crate::facade::symbols::ClientKey::raw("maintenance-edge"),
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(alternate_target),
    };
    let maintenance_create = CreateIntent::Relation(RelationSpec {
        partition_id: created.partition_id,
        kind_id: created.kind_id,
        client_key: created.client_key.clone(),
        source: created.source.clone(),
        target: created.target.clone(),
        fields: Default::default(),
    });
    maintenance
        .push_batch(
            WorkerIntentBatch::new("maintenance-relation-create")
                .push(MutationIntent::Create(maintenance_create.clone())),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    assert_eq!(
        maintenance
            .read_created_relation(&created)
            .expect("read footprint fits")
            .expect("maintenance reads its exact relation create")
            .cloned()
            .collect::<Vec<_>>(),
        vec![maintenance_create]
    );
    assert!(storm
        .read_created_relation(&created)
        .expect("read footprint fits")
        .is_none());

    let storm = storm
        .validate(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("storm relation overlay validates against its root");
    let maintenance = maintenance
        .validate(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("maintenance relation overlay validates against its root");
    assert_eq!(storm.footprint().basis(), storm_basis.descriptor());
    assert_eq!(
        maintenance.footprint().basis(),
        maintenance_basis.descriptor()
    );
    let normalized_created = maintenance
        .footprint()
        .writes()
        .find_map(|locus| match locus {
            RelationalTransactionWriteLocus::CreatedRelation(created) => Some(created.clone()),
            _ => None,
        })
        .expect("maintenance relation create is normalized in its footprint");
    let expected_reads = std::collections::BTreeSet::from([
        RelationalTransactionReadLocus::Existing(crate::facade::transactions::RecordRef::Entity(
            source,
        )),
        RelationalTransactionReadLocus::Existing(crate::facade::transactions::RecordRef::Entity(
            alternate_target,
        )),
        RelationalTransactionReadLocus::Existing(crate::facade::transactions::RecordRef::Relation(
            relation,
        )),
        RelationalTransactionReadLocus::CreatedRelation(normalized_created.clone()),
        RelationalTransactionReadLocus::ValidationPartition(
            crate::facade::identity::PartitionId::main(),
        ),
        RelationalTransactionReadLocus::RelationSchema(crate::facade::identity::KindId(2)),
    ]);
    assert_eq!(
        storm
            .footprint()
            .reads()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected_reads
    );
    assert_eq!(
        maintenance
            .footprint()
            .reads()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected_reads
    );
    assert_eq!(
        storm
            .footprint()
            .writes()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([RelationalTransactionWriteLocus::Existing(
            crate::facade::transactions::RecordRef::Relation(relation),
        )])
    );
    assert_eq!(
        maintenance
            .footprint()
            .writes()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([RelationalTransactionWriteLocus::CreatedRelation(
            normalized_created,
        )])
    );
    let expected_partitions =
        std::collections::BTreeSet::from([crate::facade::identity::PartitionId::main()]);
    assert_eq!(
        storm
            .footprint()
            .write_partitions()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        expected_partitions
    );
    assert_eq!(
        maintenance
            .footprint()
            .write_partitions()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        expected_partitions
    );
}

#[test]
fn bulk_created_reads_share_one_staged_intent_allocation() {
    let runtime = runtime_with_test_schema();
    let (_, mut transaction) = begin_on(&runtime, "main");
    let client_keys = (0..4_096)
        .map(|ordinal| crate::facade::symbols::ClientKey::raw(format!("bulk-{ordinal}")))
        .collect::<Vec<_>>();
    let created_entities = client_keys
        .iter()
        .cloned()
        .map(created_entity)
        .collect::<Vec<_>>();
    transaction
        .push_batch(
            WorkerIntentBatch::new("bulk-overlay-sharing").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: crate::facade::identity::PartitionId::main(),
                    kind_id: crate::facade::identity::KindId(1),
                    field_patches: vec![Default::default(); client_keys.len()],
                    client_keys,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");

    let first_intent = transaction
        .read_created_entity(&created_entities[0])
        .expect("read footprint fits")
        .expect("first bulk member is indexed")
        .next()
        .expect("first bulk member resolves its source intent")
        as *const CreateIntent;
    for created in &created_entities {
        let mut matches = transaction
            .read_created_entity(created)
            .expect("read footprint fits")
            .expect("every bulk member is indexed");
        let intent = matches
            .next()
            .expect("every bulk member resolves its source intent")
            as *const CreateIntent;
        assert_eq!(intent, first_intent);
        assert!(matches.next().is_none());
    }
    let expected_reads = created_entities
        .iter()
        .cloned()
        .map(RelationalTransactionReadLocus::CreatedEntity)
        .collect::<std::collections::BTreeSet<_>>();
    let expected_writes = created_entities
        .into_iter()
        .map(RelationalTransactionWriteLocus::CreatedEntity)
        .collect::<std::collections::BTreeSet<_>>();
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
    assert_eq!(
        transaction
            .footprint()
            .write_partitions()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([crate::facade::identity::PartitionId::main()])
    );
}

fn fork_from_main(runtime: &mut crate::facade::runtime::RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(branch.into()), source)
        .expect("branch fork succeeds");
}

fn begin_on(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
) -> (
    crate::facade::branch::AdmittedRelationalBranchBasis,
    crate::facade::mvcc::BranchBoundRelationalTransaction,
) {
    let identity = runtime
        .branch_identity(&BranchId(branch.into()))
        .expect("branch identity is owner-issued");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("branch basis is owner-admitted");
    let transaction = runtime
        .begin_branch_transaction(
            &basis,
            crate::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("basis belongs to this runtime");
    (basis, transaction)
}

fn created_entity(client_key: crate::facade::symbols::ClientKey) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: crate::facade::identity::PartitionId::main(),
        kind_id: crate::facade::identity::KindId(1),
        client_key,
    }
}
