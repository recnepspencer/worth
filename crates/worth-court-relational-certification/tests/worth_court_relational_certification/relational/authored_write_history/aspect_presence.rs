//! Presence-changing scalar lowering and exact installed binding custody.
use super::{collect, selected};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AbsenceLaw, AspectShape, CanonicalFieldPath, ScalarAspectType};
use worth_relational::facade::{
    history::{RelationalAuthoredWriteScope, RelationalAuthoredWriteSelector},
    identity::{EntityId, KindId, PartitionId},
    runtime::{RelationalAspectPresenceSupportDenial as Denial, RelationalRuntime},
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, entity_field_aspect, field_key,
        publications::{begin_main_transaction, changed_entities, release_test_commit_snapshot},
        AspectSchemaFixture,
    },
    durable_publication::field_patch,
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn scalar_update_initializes_presence_with_only_an_exact_field_posting() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = schema("actual_value").build_runtime();
    let created = create_empty(&runtime, execution, "initial");
    let entity = changed_entities(&created)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    assert!(!present(&before, &execution, entity));
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("initialize-optional").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: field_patch("optional", "actual_value", "initialized"),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let initialized = transaction.commit(&execution, &runtime).unwrap();
    let after = runtime
        .read_truth()
        .project_snapshot(&initialized.snapshot)
        .unwrap();
    assert!(present(&after, &execution, entity));
    assert!(before
        .compare_entity_aspect_support(&execution, &before, entity, &aspect_key("optional"))
        .unwrap());
    assert!(!before
        .compare_entity_aspect_support(&execution, &after, entity, &aspect_key("optional"))
        .unwrap());
    assert_eq!(
        before
            .entity_aspect_presence_scalar_field(
                &execution,
                &after,
                entity,
                KindId(1),
                &aspect_key("optional"),
            )
            .unwrap(),
        Some(&field_key("actual_value"))
    );
    let target = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    for (scope, expected) in [
        (RelationalAuthoredWriteScope::Lifecycle, vec![]),
        (
            RelationalAuthoredWriteScope::WholeAspect(aspect_key("optional")),
            vec![],
        ),
        (
            RelationalAuthoredWriteScope::Field(
                aspect_key("optional"),
                CanonicalFieldPath::single(field_key("actual_value")),
            ),
            vec![initialized.commit.commit_id],
        ),
    ] {
        let key = RelationalAuthoredWriteSelector::Record {
            record: RecordRef::Entity(entity),
            scope,
        };
        let cursor = runtime
            .authored_write_history_cursor(execution, &source, selected(&runtime, &target), &[key])
            .unwrap();
        assert_eq!(collect(&runtime, execution, cursor).0, expected);
    }
    assert_eq!(
        before.entity_aspect_presence_scalar_field(
            &execution,
            &after,
            entity,
            KindId(2),
            &aspect_key("optional"),
        ),
        Err(Denial::EntityMismatch)
    );
    let wildcard = EntityId::new(entity.partition_id, entity.local_slot_value(), 0);
    assert_eq!(
        before.entity_aspect_presence_scalar_field(
            &execution,
            &after,
            wildcard,
            KindId(1),
            &aspect_key("optional"),
        ),
        Err(Denial::EntityMismatch)
    );
    let foreign = schema("actual_value").build_runtime();
    let wrong_generation = EntityId::new(
        entity.partition_id,
        entity.local_slot_value(),
        entity.generation_value().checked_add(1).unwrap(),
    );
    assert_eq!(
        before.entity_aspect_presence_scalar_field(
            &execution,
            &after,
            wrong_generation,
            KindId(1),
            &aspect_key("optional"),
        ),
        Err(Denial::SourceUnavailable)
    );
    let other = create_empty(&foreign, execution, "foreign");
    let other_view = foreign
        .read_truth()
        .project_snapshot(&other.snapshot)
        .unwrap();
    assert_eq!(
        before.compare_entity_aspect_support(
            &execution,
            &other_view,
            entity,
            &aspect_key("optional")
        ),
        Err(
            worth_relational::facade::runtime::RelationalFieldSupportComparisonDenial::ForeignOwner
        )
    );
    let cancelled = worth_execution::CancellationSource::new();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancelled.token(),
        None,
    );
    cancelled.cancel();
    assert_eq!(
        before.compare_entity_aspect_support(
            &ExecutionRequest::serial(&stopped),
            &after,
            entity,
            &aspect_key("optional")
        ),
        Err(
            worth_relational::facade::runtime::RelationalFieldSupportComparisonDenial::Execution(
                RelationalExecutionDenialCause::Cancelled
            )
        )
    );
    assert_eq!(
        before.entity_aspect_presence_scalar_field(
            &execution,
            &other_view,
            entity,
            KindId(1),
            &aspect_key("optional"),
        ),
        Err(Denial::ForeignOwner)
    );
    for outcome in [&created, &initialized] {
        release_test_commit_snapshot(&runtime, outcome);
    }
    release_test_commit_snapshot(&foreign, &other);
}

fn schema(field: &str) -> AspectSchemaFixture {
    let mut binding = entity_field_aspect(aspect_key("optional"), field_key(field));
    let contract = &binding.contract;
    let AspectShape::Scalar(_) = contract.shape() else {
        unreachable!()
    };
    binding.contract = worth_foundational::facade::aspects()
        .contract()
        .for_key(contract.key().clone())
        .identified_by(contract.identity())
        .at_revision(contract.revision())
        .scalar_with(
            ScalarAspectType::String,
            contract.masks().clone(),
            AbsenceLaw::Optional,
            contract.equivalence(),
            contract.evolution(),
        );
    AspectSchemaFixture {
        entity_aspects: vec![binding],
        ..AspectSchemaFixture::default()
    }
}

fn empty_batch(key: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: key.into(),
        fields: AspectFieldPatch::new(Default::default()),
    })))
}

fn create_empty(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    key: &str,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(empty_batch(key), execution.allocation_policy())
        .unwrap();
    transaction.commit(&execution, runtime).unwrap()
}

fn present(
    view: &worth_relational::facade::runtime::VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
) -> bool {
    view.read_exact_entity_state(execution, entity, |_, state| {
        state.is_some_and(|state| state.get(&aspect_key("optional")).is_some())
    })
    .unwrap()
    .unwrap()
}
