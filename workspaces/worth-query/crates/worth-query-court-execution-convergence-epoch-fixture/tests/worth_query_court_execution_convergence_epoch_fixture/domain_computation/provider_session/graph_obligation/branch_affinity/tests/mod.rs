use std::collections::BTreeMap;

use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;
use worth_runtime_bridge::facade::TruthBranchIdentity;

use worth_query_execution::domain_computation::provider_session::graph_obligation::branch_affinity::WorthQueryGraphWorkBranchAffinity;

#[test]
fn equal_version_snapshot_from_another_branch_cannot_satisfy_affinity() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(
            RelationalSchemaRegistry::new()
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(1),
                    kind_name: "branch-affinity.fixture".into(),
                    schema_id: SchemaId("branch-affinity-fixture".into()),
                    schema_version_id: SchemaVersionId(1),
                    aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
                })
                .expect("branch-affinity fixture schema registers"),
        )
        .build()
        .expect("valid native runtime configuration");
    let mut transaction = {
        let transaction_validation_input = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .expect("main branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("branch-affinity-fixture").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("branch-affinity-root"),
                    fields: AspectFieldPatch::new(BTreeMap::new()),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let committed = transaction
        .commit(execution, &runtime)
        .expect("branch-affinity fixture root commits");
    assert!(runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .is_ok());
    let admitted_branch = BranchId("main".to_owned());
    let substitute_branch = BranchId("hostile".to_owned());
    let (_, fork_basis) = runtime
        .observe_fork_source(&admitted_branch)
        .expect("main branch exposes an exact fork basis");
    runtime
        .fork_branch(execution, substitute_branch.clone(), fork_basis)
        .expect("main basis can fork a sibling branch");
    let admitted_identity = runtime.branch_identity(&admitted_branch).unwrap();
    let substitute_identity = runtime.branch_identity(&substitute_branch).unwrap();
    let (_, admitted_basis) = runtime.observe_branch(&admitted_identity).unwrap();
    let (_, substitute_basis) = runtime.observe_branch(&substitute_identity).unwrap();
    let admitted = runtime
        .snapshots()
        .snapshot_for_observation(&admitted_basis.observation())
        .unwrap();
    let substitute = runtime
        .snapshots()
        .snapshot_for_observation(&substitute_basis.observation())
        .unwrap();
    let affinity = WorthQueryGraphWorkBranchAffinity::from_snapshot(&admitted);

    assert!(affinity.admits_snapshot(&admitted));
    assert!(!affinity.admits_snapshot(&substitute));
    assert_eq!(affinity.relational(), admitted.branch_id());
    assert_eq!(
        affinity.truth(),
        &TruthBranchIdentity::from_relational_branch_id(admitted.branch_id().0.clone())
    );
    assert!(runtime.snapshots().release_snapshot(&admitted).is_ok());
    assert!(runtime.snapshots().release_snapshot(&substitute).is_ok());
}
