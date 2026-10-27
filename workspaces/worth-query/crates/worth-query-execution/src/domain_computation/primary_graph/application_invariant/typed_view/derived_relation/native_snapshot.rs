//! Persisted source meaning survives the real checkpoint/selected-root boundary.
use super::native_fixture::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    runtime::*,
    schema::*,
    symbols::ClientKey,
    transactions::*,
};
#[test]
fn checkpoint_restores_exact_derived_meaning_instead_of_rebinding_a_changed_recipe() {
    let empty = RelationalSchemaRegistry::new().authority_snapshot();
    let empty_text = serde_json::to_value(&empty).unwrap();
    assert_eq!(
        empty_text,
        serde_json::json!({
            "primary_schema_id":null, "primary_schema_version_id":null,
            "entity_kinds":[], "relation_kinds":[]
        }),
        "physical-only snapshot keeps the old absent-tail text shape"
    );
    assert_eq!(
        serde_json::from_value::<SchemaAuthoritySnapshot>(empty_text).unwrap(),
        empty
    );
    let schema = registry();
    let expected = schema.authority_snapshot();
    let text = serde_json::to_string(&expected).unwrap();
    assert_eq!(
        serde_json::from_str::<SchemaAuthoritySnapshot>(&text).unwrap(),
        expected
    );
    let changed = registry_with_source(3);
    assert_ne!(
        schema.authority_digest_bytes(),
        changed.authority_digest_bytes()
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema.clone())
        .build();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let mut birth = WorkerIntentBatch::new("snapshot-birth");
    for index in 0..2 {
        birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId::new(1),
            client_key: created(index).client_key,
            fields: Default::default(),
        })));
    }
    birth = birth.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId::new(2),
            client_key: ClientKey::raw("persisted-edge"),
            source: EntityReference::Created(created(0)),
            target: EntityReference::Created(created(1)),
            fields: Default::default(),
        },
    )));
    let born = commit(&runtime, &execution, birth);
    let root = born.created_entity(&created(0)).unwrap();
    let target = born.created_entity(&created(1)).unwrap();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let installed_handle = schema.derived_relation(&identity(2)).unwrap();
    let mut matching = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    matching
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    let (_, matching_basis) = matching
        .observe_branch(&matching.main_branch_identity())
        .unwrap();
    let matching_snapshot = matching
        .snapshots()
        .snapshot_for_observation(&matching_basis.observation())
        .unwrap();
    let matching_pin = matching
        .read_truth()
        .pin_read_capability(&matching_snapshot, &[], &execution)
        .unwrap();
    assert_eq!(matching_pin.derived_relations_from(&execution, &installed_handle, root, None)
        .unwrap().data().endpoints(), [target.into()],
        "exact readmission preserves the actual installed source issuer, not a rebuilt equal handle");
    let mut restored = RelationalRuntimeApi::builder()
        .schema_registry(changed.clone())
        .build();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    let (_, basis) = restored
        .observe_branch(&restored.main_branch_identity())
        .unwrap();
    let snapshot = restored
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let pin = restored
        .read_truth()
        .pin_read_capability(&snapshot, &[], &execution)
        .unwrap();
    assert_eq!(
        pin.schema().authority_snapshot(),
        expected,
        "the actual recovered branch root retains its persisted executable recipe"
    );
    let exact = pin.schema().derived_relation(&identity(2)).unwrap();
    assert_eq!(
        pin.derived_relations_from(&execution, &exact, root, None)
            .unwrap()
            .data()
            .endpoints(),
        [target.into()]
    );
    let foreign = changed.derived_relation(&identity(2)).unwrap();
    assert_eq!(
        pin.derived_relations_from(&execution, &foreign, root, None)
            .err(),
        Some(DerivedRelationReadDenial::Definition(
            DerivedRelationDenial::ForeignDefinition
        ))
    );
}
