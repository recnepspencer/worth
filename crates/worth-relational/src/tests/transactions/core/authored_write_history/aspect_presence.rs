//! Presence-changing scalar lowering and exact installed binding custody.
use super::*;
use crate::facade::identity::KindId;
use crate::facade::runtime::RelationalAspectPresenceSupportDenial as Denial;
use crate::schema::data::*;
use worth_foundational::facade::{AbsenceLaw, AspectShape, ScalarAspectType};

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
