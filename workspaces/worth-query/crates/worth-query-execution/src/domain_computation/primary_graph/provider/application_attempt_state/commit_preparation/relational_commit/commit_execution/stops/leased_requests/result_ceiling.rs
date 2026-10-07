//! A real index result ceiling retains its cause at the application boundary.
use super::*;
use worth_foundational::facade::{
    aspects, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue, CanonicalFieldPath,
    FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_relational::facade::{
    schema::{AspectBinding, DeclaredAspectContractBinding},
    transactions::BulkEntityCreateIntent,
};

#[test]
fn index_result_capacity_reaches_application_caller() {
    in_isolated_process(
        concat!(
            module_path!(),
            "::index_result_capacity_reaches_application_caller"
        ),
        || {
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
                )
                .unwrap();
            let committed = transaction.commit(&runtime).unwrap();
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
            let baseline = runtime
                .index_authority()
                .build_for_commit(build_request.clone());
            assert!(baseline.execution_denial.is_none());
            let published = baseline.generations[0].generation_id;
            // The existing large-index oracle uses one worker and 64 KiB:
            // admission fits, then Arc-backed rows reach the 8 KiB result ceiling.
            let lease = authority()
                .request_lease(request(64 * 1024, 1_000_000))
                .unwrap();
            let build = runtime
                .index_authority()
                .build_for_commit_with_lease(build_request.clone(), &lease);
            assert!(build.generations.is_empty());
            let denial = build
                .execution_denial
                .expect("index result ceiling refused");
            let DerivedIndexExecutionDenialKind::Cause(cause) = denial.kind;
            assert_eq!(
                resource(application_outcome(
                    super::super::relational_execution_stop(cause, denial.partition_identity,)
                )),
                Resource::ResultCapacityExceeded
            );
            assert_eq!(
                runtime
                    .index_access()
                    .latest_generation(index.index_id, &build_request.branch_id)
                    .unwrap()
                    .generation_id,
                published
            );
        },
    );
}
