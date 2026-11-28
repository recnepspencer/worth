//! A Query-owned runtime enforces Native uniqueness independently of decision facts.
use worth_query_testkit_execution_primary_graph_application_attempt_2::{
    domain_computation::primary_graph::tests::unique_write_world::UniqueWriteWorld,
    test_request_policy::serial_request,
};
use worth_relational::facade::{
    history::BranchId,
    identity::PartitionId,
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    mvcc::RelationalTransactionIntent,
    symbols::ClientKey,
    transactions::{
        ConflictClass, CreateIntent, EntitySpec, InvariantViolationFields,
        MutationIntent, TransactionCommitError, WorkerIntentBatch,
    },
};

#[test]
fn a_native_unique_index_refuses_a_writer_without_query_decision_facts() {
    let host = serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let world = UniqueWriteWorld::new();
    world.create(request, "main", "first-unique", 7);
    world.mutate(request, |runtime| {
        let branch = runtime.branch_identity(&BranchId("main".into())).unwrap();
        let basis = runtime.admit_branch_basis(&branch).unwrap();
        let definition = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "court.account-label.unique".into(),
            kind: DerivedIndexKind::EntityField { field_locator: world.label.clone() },
            branch_scoped: false,
            unique_entity_kind: Some(world.kind),
        }).unwrap();
        let generation = runtime.index_authority().build_for_basis(
            request,
            DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: branch.branch_id().clone(),
                index_ids: vec![definition.index_id],
            },
            &basis,
        );
        assert_eq!(generation.generations.len(), 1);
        let mut transaction = runtime.begin_branch_transaction(
            &basis, RelationalTransactionIntent::ordinary(),
        ).unwrap();
        transaction.push_batch(
            WorkerIntentBatch::new("query-unique-collision").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: world.kind,
                    client_key: ClientKey::raw("second-unique"),
                    fields: world.created_fields("second-unique", 7),
                }),
            )),
            request.allocation_policy(),
        ).unwrap();
        let refused = transaction.commit(request, runtime).unwrap_err();
        assert!(matches!(refused, TransactionCommitError::Conflict { error, .. }
            if matches!(&error.class, ConflictClass::InvariantViolation {
                fields: InvariantViolationFields::UniqueEntityField {
                    entity_kind_id, field_locator, ..
                }, ..
            } if *entity_kind_id == world.kind && field_locator == &world.label)));
        assert_eq!(runtime.admit_branch_basis(&branch).unwrap().reference(), basis.reference());
    });
}
