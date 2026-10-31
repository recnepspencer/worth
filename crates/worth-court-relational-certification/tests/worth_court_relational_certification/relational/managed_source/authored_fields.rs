//! Real declared Struct-field and endpoint commands feed conservative Native history.
use super::*;
use super::{
    history::{basis, collect, edge, edge_id, endpoints, selected},
    registry::{self, Profile},
};
use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};
use worth_relational::facade::history::*;

fn run(
    profile: Profile,
    test: impl FnOnce(&RelationalRuntime, &ManagedIndexSourceProgram, ExecutionRequest<'_, '_>),
) {
    let (schema, program) = registry::profile(profile);
    let court = Arc::new(owner::Court::default());
    let rule = CustomInvariantRegistration::new(owner::Reader {
        maximum_work: None,
        program: program.clone(),
        court: court.clone(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program.clone()])
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .custom_invariant(rule)
        .build()
        .unwrap();
    let _registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court)))
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    test(&runtime, &program, ExecutionRequest::serial(&host));
}
fn fields(field: &str, value: &str) -> AspectFieldPatch {
    AspectFieldPatch::from_locator(
        planned_single_field_locator(registry::key(), registry::field(field)),
        AspectValue::String(value.into()),
    )
}
fn birth(kind: u32, label: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
        fields: fields("value", label),
    }))
}
fn update(entity: EntityId, field: &str, value: &str) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: fields(field, value),
        },
    ))
}
fn whole(entity: EntityId, value: Option<&str>) -> MutationIntent {
    let basis = PortableAspectContractBasis::from_contract(&registry::contract());
    let operation = match value {
        Some(value) => PortableAspectPatchOperation::SetWhole {
            basis,
            value: ContractValidationInput::Struct(
                StructAspectValue::new(BTreeMap::from([(
                    registry::field("value"),
                    AspectValue::String(value.into()),
                )]))
                .unwrap(),
            ),
        },
        None => PortableAspectPatchOperation::ClearWhole { basis },
    };
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id: entity,
            aspect_patch: PortableRecordAspectPatch::new([operation]),
        },
    ))
}
fn write(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    intent: MutationIntent,
) -> CommitResult {
    commit(
        runtime,
        &execution,
        WorkerIntentBatch::new("authored-field-history").push(intent),
    )
}
fn scan(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    original: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    target: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    scope: EntityId,
) -> Vec<CommitId> {
    let selector = runtime
        .authored_managed_source_selector(execution, program, scope)
        .unwrap();
    collect(
        runtime,
        execution,
        runtime
            .authored_managed_source_history_cursor(
                execution,
                original,
                selected(runtime, target),
                &[selector],
            )
            .unwrap(),
    )
}

#[test]
fn declared_field_history_keeps_row_whole_aspect_and_projected_endpoint_noops_across_rewiring() {
    run(Profile::Fields, |runtime, program, execution| {
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("field-history-birth")
                .push(birth(1, "A"))
                .push(birth(1, "B"))
                .push(birth(1, "C"))
                .push(birth(4, "x"))
                .push(birth(4, "y"))
                .push(birth(4, "z")),
        );
        let [a, b, c] =
            ["A", "B", "C"].map(|label| born.created_entity(&created(1, label)).unwrap());
        let [x, y, z] =
            ["x", "y", "z"].map(|label| born.created_entity(&created(4, label)).unwrap());
        let linked = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("field-history-links")
                .push(edge(a, x, "A-x"))
                .push(edge(b, y, "B-y"))
                .push(edge(c, z, "C-z")),
        );
        let a_x = edge_id(&linked, a, x, "A-x");
        let original = basis(runtime);
        let row_noop = write(runtime, execution, update(x, "value", "x"));
        let whole_noop = write(runtime, execution, whole(x, Some("x")));
        let cleared = write(runtime, execution, whole(x, None));
        let clear_noop = write(runtime, execution, whole(x, None));
        let restored = write(runtime, execution, whole(x, Some("x")));
        let endpoint_noop = write(runtime, execution, update(a, "value", "A"));
        // This profile deliberately retains a conservative field-kind superset.
        // An unrelated field on x remains a candidate, not exact Source relevance.
        let other_field = write(runtime, execution, update(x, "other", "not-projected"));
        let other_scope = write(runtime, execution, update(y, "value", "y"));
        let moved = endpoints(runtime, execution, a_x, b, x);
        let new_endpoint = write(runtime, execution, update(b, "value", "B"));
        let disjoint = write(runtime, execution, update(c, "value", "C"));
        // This retires B-x and B-y for audit via actual induced physical changes.
        let retired = write(
            runtime,
            execution,
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: b,
            })),
        );
        let target = basis(runtime);
        assert_eq!(
            scan(runtime, execution, program, &original, &target, a),
            [
                &row_noop,
                &whole_noop,
                &cleared,
                &clear_noop,
                &restored,
                &endpoint_noop,
                &other_field,
                &moved
            ]
            .map(|outcome| outcome.commit.commit_id)
        );
        assert_eq!(
            scan(runtime, execution, program, &original, &target, b),
            [
                other_scope.commit.commit_id,
                moved.commit.commit_id,
                new_endpoint.commit.commit_id,
                retired.commit.commit_id
            ]
        );
        assert_eq!(
            scan(runtime, execution, program, &original, &target, c),
            [disjoint.commit.commit_id]
        );
    });
}

#[test]
fn membership_only_history_omits_entity_scalar_commands_but_keeps_scope_replacement() {
    run(Profile::SchemaFieldsOnly, |runtime, program, execution| {
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("membership-fields-birth")
                .push(birth(1, "A"))
                .push(birth(1, "B"))
                .push(birth(4, "x")),
        );
        let a = born.created_entity(&created(1, "A")).unwrap();
        let b = born.created_entity(&created(1, "B")).unwrap();
        let x = born.created_entity(&created(4, "x")).unwrap();
        write(runtime, execution, edge(a, x, "A-x"));
        let original = basis(runtime);
        write(runtime, execution, update(x, "value", "x"));
        write(runtime, execution, whole(x, Some("x")));
        write(runtime, execution, whole(x, None));
        write(runtime, execution, update(a, "value", "A"));
        let target = basis(runtime);
        assert!(scan(runtime, execution, program, &original, &target, a).is_empty());
        write(
            runtime,
            execution,
            update(b, "value", "changed-before-replacement"),
        );
        let target = basis(runtime);
        assert!(scan(runtime, execution, program, &original, &target, b).is_empty());
        let replaced = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("replace-empty-scope").push(MutationIntent::Entity(
                EntityMutationIntent::Replace(ReplaceEntityIntent {
                    entity_id: b,
                    replacement: EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: ClientKey::raw("new-B"),
                        fields: fields("value", "new-B"),
                    },
                }),
            )),
        );
        let target = basis(runtime);
        assert_eq!(
            scan(runtime, execution, program, &original, &target, b),
            [replaced.commit.commit_id]
        );
    });
}
