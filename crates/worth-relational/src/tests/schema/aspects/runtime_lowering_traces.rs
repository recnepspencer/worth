use worth_foundational::{AspectKey, AspectShape, ReferenceAspectType, ScalarAspectType};

use crate::capabilities::AspectPlanSource;
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn runtime_build_lowers_schema_aspect_plans() {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
                lifecycle_aspect(),
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration");
    let plan = runtime.entity_aspect_plan(KindId(1)).unwrap();

    assert_eq!(plan.kind_id, KindId(1));
    assert_eq!(plan.executable_bindings.len(), 2);
    assert_eq!(
        plan.executable_bindings[0].aspect_key(),
        &AspectKey::new("lifecycle").unwrap()
    );
    assert_eq!(
        plan.executable_bindings[1].aspect_key(),
        &AspectKey::new("name").unwrap()
    );
    assert_eq!(
        plan.executable_bindings[0].contract.shape(),
        &AspectShape::Scalar(ScalarAspectType::String)
    );
    assert_eq!(
        plan.executable_bindings[1].contract.shape(),
        &AspectShape::Scalar(ScalarAspectType::String)
    );
}

#[test]
fn schema_and_runtime_expose_consistent_aspect_declaration_and_lowering_traces() {
    let registry = declared_aspect_schema_registry(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity_declaration_trace = registry.entity_aspect_declaration_trace(KindId(1)).unwrap();
    let relation_declaration_trace = registry
        .relation_aspect_declaration_trace(KindId(2))
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry.clone())
        .build()
        .expect("valid native runtime configuration");
    let entity_lowering_trace = runtime.entity_aspect_plan_trace(KindId(1)).unwrap();
    let relation_lowering_trace = runtime.relation_aspect_plan_trace(KindId(2)).unwrap();

    assert_eq!(entity_declaration_trace.kind_id, KindId(1));
    assert_eq!(
        entity_declaration_trace.plan_revision,
        entity_lowering_trace.plan_revision
    );
    assert_eq!(
        entity_declaration_trace
            .declarations
            .iter()
            .map(|row| row.aspect_key.clone())
            .collect::<Vec<_>>(),
        entity_lowering_trace
            .bindings
            .iter()
            .map(|row| row.aspect_key.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(entity_declaration_trace.declarations.len(), 2);
    assert_eq!(relation_declaration_trace.kind_id, KindId(2));
    assert_eq!(
        relation_declaration_trace.plan_revision,
        relation_lowering_trace.plan_revision
    );
    assert_eq!(relation_declaration_trace.declarations.len(), 4);
    assert_eq!(relation_lowering_trace.bindings.len(), 4);
    assert_eq!(
        relation_lowering_trace.bindings[0].aspect_key,
        AspectKey::new("label").unwrap()
    );
    assert_eq!(
        relation_lowering_trace.bindings[1].aspect_key,
        AspectKey::new("lifecycle").unwrap()
    );
    assert_eq!(
        relation_lowering_trace.bindings[2].aspect_key,
        AspectKey::new("source").unwrap()
    );
    assert_eq!(
        relation_lowering_trace.bindings[3].aspect_key,
        AspectKey::new("target").unwrap()
    );
}

#[test]
fn schema_traces_emit_diagnostic_artifacts_without_reinterpreting_semantics() {
    let registry = declared_aspect_schema_registry(CascadeDeletePolicy::CascadeDeleteRelations);
    let declaration_trace = registry.entity_aspect_declaration_trace(KindId(1)).unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration");
    let lowering_trace = runtime.entity_aspect_plan_trace(KindId(1)).unwrap();
    let declaration_artifact = declaration_trace.diagnostic_artifact();
    let lowering_artifact = lowering_trace.diagnostic_artifact();

    assert_eq!(declaration_artifact.scope, DiagnosticsScope::Schema);
    assert_eq!(
        declaration_artifact.kind,
        DiagnosticsArtifactKind::DetailedTrace
    );
    assert_eq!(
        declaration_artifact.entries[0].code,
        DiagnosticCode::AspectDeclarationTraced
    );
    assert_eq!(lowering_artifact.scope, DiagnosticsScope::Schema);
    assert_eq!(
        lowering_artifact.kind,
        DiagnosticsArtifactKind::DetailedTrace
    );
    assert_eq!(
        lowering_artifact.entries[0].code,
        DiagnosticCode::AspectLoweringTraced
    );
}

#[test]
fn aspect_schema_fixture_builds_runtime_with_lowered_plans_for_customized_aspects() {
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            entity_field_aspect(
                crate::tests::support::aspect_key("status"),
                crate::tests::support::field_key("status"),
            ),
            lifecycle_aspect(),
        ],
        relation_aspects: vec![relation_source_aspect(), relation_target_aspect()],
        ..AspectSchemaFixture::default()
    };
    let runtime = fixture.build_runtime();
    let entity_plan = runtime.entity_aspect_plan(KindId(1)).unwrap();
    let relation_plan = runtime.relation_aspect_plan(KindId(2)).unwrap();

    assert_eq!(entity_plan.executable_bindings.len(), 3);
    assert_eq!(relation_plan.executable_bindings.len(), 2);
    assert_eq!(
        entity_plan
            .executable_bindings
            .iter()
            .map(|binding| binding.aspect_key().clone())
            .collect::<Vec<_>>(),
        vec![
            aspect_key("lifecycle"),
            aspect_key("name"),
            aspect_key("status")
        ]
    );
    assert_eq!(
        relation_plan
            .executable_bindings
            .iter()
            .map(|binding| binding.aspect_key().clone())
            .collect::<Vec<_>>(),
        vec![aspect_key("source"), aspect_key("target")]
    );
    assert_eq!(
        relation_plan.executable_bindings[0].contract.shape(),
        &AspectShape::Reference(ReferenceAspectType::Entity)
    );
    assert_eq!(
        relation_plan.executable_bindings[1].contract.shape(),
        &AspectShape::Reference(ReferenceAspectType::Entity)
    );
}
