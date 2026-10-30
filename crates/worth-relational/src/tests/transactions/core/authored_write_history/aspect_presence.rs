//! Presence-changing scalar lowering and exact installed binding custody.
use super::*;
use crate::facade::identity::KindId;
use crate::facade::runtime::RelationalAspectPresenceSupportDenial as Denial;
use crate::schema::data::*;
use worth_foundational::facade::{AbsenceLaw, AspectShape, ScalarAspectType};

#[test]
fn scalar_update_initializes_presence_with_only_an_exact_field_posting() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = schema("actual_value").build_runtime();
    let created = create_empty(&runtime, execution, "initial");
    let entity = changed_entities(&created)[0];
    let source = test_owner_main_basis(&runtime).unwrap();
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    assert!(!present(&before, &execution, entity));
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("initialize-optional").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: single_string_aspect_field_patch(
                        aspect_key("optional"),
                        field_key("actual_value"),
                        "initialized",
                    ),
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
    let target = test_owner_main_basis(&runtime).unwrap();
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
        Err(crate::facade::runtime::RelationalFieldSupportComparisonDenial::ForeignOwner)
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
            crate::facade::runtime::RelationalFieldSupportComparisonDenial::Execution(
                crate::execution::RelationalExecutionDenialCause::Cancelled
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

#[test]
fn paired_presence_refuses_a_changed_installed_entity_field_binding() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut runtime = schema("actual_value").build_runtime();
    let created = create_empty(&runtime, execution, "before-binding-change");
    let entity = changed_entities(&created)[0];
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let mut next = schema("renamed_value");
    next.schema_version_id = SchemaVersionId(2);
    runtime.set_schema_registry_for_test(next.build_registry());
    let transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".into()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".into()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::ProjectionContract,
                SchemaId("test".into()),
                SchemaVersionId(2),
                Some(KindId(1)),
                "optional-binding",
            ),
            vec![SchemaStratum::PublicationContract],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::ProjectionContractChanged {
                projection_name: "optional-binding".into(),
            },
        )
        .with_boundary_visibility_proof(
            SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    };
    let input = test_owner_transaction_validation_input_for_main(&runtime).with_schema_transition(
        transition,
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );
    let mut transaction = runtime
        .begin_branch_transaction(input.basis(), input.intent().clone())
        .unwrap();
    transaction
        .push_batch(
            empty_batch("after-binding-change"),
            execution.allocation_policy(),
        )
        .unwrap();
    let changed = transaction.commit(&execution, &runtime).unwrap();
    let after = runtime
        .read_truth()
        .project_snapshot(&changed.snapshot)
        .unwrap();
    assert_eq!(
        after
            .entity_aspect_presence_scalar_field(
                &execution,
                &after,
                entity,
                KindId(1),
                &aspect_key("optional"),
            )
            .unwrap(),
        Some(&field_key("renamed_value")),
        "target really installed the changed binding"
    );
    assert_eq!(
        before.entity_aspect_presence_scalar_field(
            &execution,
            &after,
            entity,
            KindId(1),
            &aspect_key("optional"),
        ),
        Err(Denial::IncompatibleBinding)
    );
    assert_eq!(
        before.compare_entity_aspect_support(&execution, &after, entity, &aspect_key("optional")),
        Err(crate::facade::runtime::RelationalFieldSupportComparisonDenial::SourceUnavailable)
    );
    for outcome in [&created, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
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
        partition_id: crate::identity::data::PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
        fields: AspectFieldPatch::new(Default::default()),
    })))
}

fn create_empty(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    key: &str,
) -> CommitResult {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(empty_batch(key), execution.allocation_policy())
        .unwrap();
    transaction.commit(&execution, runtime).unwrap()
}

fn present(
    view: &crate::facade::runtime::VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
) -> bool {
    view.read_exact_entity_state(execution, entity, |_, state| {
        state.is_some_and(|state| state.get(&aspect_key("optional")).is_some())
    })
    .unwrap()
    .unwrap()
}
