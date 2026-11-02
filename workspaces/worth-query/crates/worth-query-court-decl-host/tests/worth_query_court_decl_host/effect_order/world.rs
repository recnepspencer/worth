use super::*;
pub fn request() -> SerialRequest {
    // Projection retains native read material and therefore needs actual
    // original-request memory custody, not worker-only placement.
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(64 * 1024 * 1024), None),
    );
    SerialRequest::from_policy(&policy, worth_execution::CancellationToken::new(), None)
}
pub fn commit(
    runtime: &mut RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    intents: Vec<MutationIntent>,
) -> Result<CommitResult, TransactionCommitError> {
    let input = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &input,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            intents.into_iter().fold(
                WorkerIntentBatch::new("effect-order"),
                WorkerIntentBatch::push,
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime)
}
pub fn created(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: schema::KIND,
        client_key: ClientKey::raw(key),
    }
}
pub fn create(key: &str, unique: &str, count: u64) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: schema::KIND,
        client_key: ClientKey::raw(key),
        fields: AspectFieldPatch::from(std::collections::BTreeMap::from([
            (
                schema::locator("Alias", "Unique"),
                AspectValue::String(unique.into()),
            ),
            (
                schema::locator("Marker", "Count"),
                AspectValue::UInt64(count),
            ),
        ])),
    }))
}
pub fn clear(id: EntityId, names: &[&str]) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id: id,
            aspect_patch: PortableRecordAspectPatch::new(names.iter().map(|name| {
                PortableAspectPatchOperation::ClearWhole {
                    basis: schema::basis(name),
                }
            })),
        },
    ))
}
pub fn take(id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: id,
            fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                schema::locator("Alias", "Unique"),
                AspectValue::String("token".into()),
            )])),
        },
    ))
}
pub fn field(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    id: EntityId,
    aspect: &str,
    field: &str,
) -> Option<AspectValue> {
    let aspect = AspectKey::new(aspect).unwrap();
    let field = FieldKey::new(field).unwrap();
    runtime
        .read_truth()
        .project_snapshot(snapshot)
        .unwrap()
        .entity_record_with_projection_scope(
            execution,
            id,
            ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
                aspect.clone(),
                [field.clone()],
            )]),
            |record| {
                assert_eq!(
                    record.lifecycle(),
                    worth_relational::facade::storage::RecordLifecycleState::Live
                );
                Some(
                    match record.observe_aspect_field(&aspect, &field).unwrap() {
                        RelationalProjectedFieldObservation::Present(value) => Some(value.clone()),
                        RelationalProjectedFieldObservation::AspectAbsent => None,
                        other => panic!("unexpected native field observation: {other:?}"),
                    },
                )
            },
        )
        .unwrap()
        .expect("literal seeded entity remains live")
}
pub fn unchanged_head(
    runtime: &RelationalRuntime,
    seed: &CommitResult,
) -> worth_relational::facade::snapshots::SnapshotHandle {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let observation = basis.observation();
    assert_eq!(observation.commit_receipt(), Some(&seed.commit));
    runtime
        .snapshots()
        .snapshot_for_observation(&observation)
        .unwrap()
}
