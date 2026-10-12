//! The real immutable index source shared by bootstrap stop proofs.
use super::*;
pub(super) fn index_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_relational::facade::runtime::RelationalRuntime,
    DerivedIndexBuildRequest,
) {
    let key = AspectKey::new("name").unwrap();
    let field = FieldKey::new("name").unwrap();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        key.clone(),
        CanonicalFieldPath::single(field.clone()),
    );
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "result.entity".to_owned(),
            schema_id: SchemaId("result-probe".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField { field },
                    contract: aspects()
                        .contract()
                        .for_key(key)
                        .identified_by(AspectIdentity(1))
                        .at_revision(aspects().vocabulary().revision(1))
                        .scalar(ScalarAspectType::String),
                },
            ]),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("index-source").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: (0..128)
                        .map(|i| ClientKey::raw(format!("index-heavy-{i}")))
                        .collect(),
                    field_patches: (0..128)
                        .map(|i| {
                            AspectFieldPatch::from_locator(
                                locator.clone(),
                                AspectValue::String(format!("index-heavy-{i}").into()),
                            )
                        })
                        .collect(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "large.index.name".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator,
        },
        branch_scoped: true,
    });
    let build_request = DerivedIndexBuildRequest {
        source_commit_id: committed.commit.commit_id,
        branch_id: BranchId("main".to_owned()),
        index_ids: vec![index.index_id],
    };
    (runtime, build_request)
}
