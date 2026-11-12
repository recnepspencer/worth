use worth_execution::ExecutionRequest;
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::schema::{
    HistoricalInterpretationSensitivity, ProposedSchemaTransition, RelationalSchemaRegistry,
    SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId,
    SchemaPublicationImpact, SchemaReconciliationPolicy, SchemaStratum, SchemaSubscriberImpact,
    SchemaVersionId, SubscriberBoundaryVisibility,
};
use worth_testkit_relational::aspect_storage::definition::field_key;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
use worth_testkit_relational::durable_publication::batch_create;

// Exact version-only schema transitions through the shipping Native owner.
pub(super) fn registry(version: u32) -> RelationalSchemaRegistry {
    AspectSchemaFixture {
        schema_version_id: SchemaVersionId(version),
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry()
}
pub(super) fn transition(target: SchemaVersionId) -> ProposedSchemaTransition {
    ProposedSchemaTransition {
        source_schema_id: SchemaId("test".into()),
        source_schema_version_id: SchemaVersionId(target.0 - 1),
        target_schema_id: SchemaId("test".into()),
        target_schema_version_id: target,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".into()),
                target,
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(
            SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}
pub(super) fn transition_main(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target: u32,
    key: &str,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_schema_transition(
            &basis,
            transition(SchemaVersionId(target)),
            Some(SchemaReconciliationPolicy::PreserveInformation),
            registry(target),
        )
        .unwrap();
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    let transitioned = transaction.commit(execution, runtime).unwrap();
    assert!(transitioned.envelope().schema_transition.is_some());
}
