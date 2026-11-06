//! Real compiler allocation ownership; no substitute execution or publication.
use super::*;
use crate::identity::data::KindId;
use crate::schema::data::{
    EntityKindRegistration, KindAspectContractDeclarations, SchemaId, SchemaVersionId,
};
use worth_execution::SerialMemoryBudget;

fn registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(91),
            kind_name: "retained.schema.kind".to_owned(),
            schema_id: SchemaId("retained.schema".to_owned()),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap()
}

#[test]
fn retained_schema_children_keep_the_original_compilation_charge_until_the_last_share_drops() {
    let control = crate::schema::SchemaCompilationControl::Configuration;
    let previous = RelationalBranchRootSchemaAuthority::empty();
    let registry = registry();
    let quote =
        compilation_quote::SchemaCompilationQuote::for_target(&registry, &previous, control)
            .unwrap();
    let memory = SerialMemoryBudget::new(None);
    let funding = Arc::new(compiled_retention::SchemaCompilationFunding {
        _memory: Some(memory.reserve(quote.retained).unwrap()),
    });
    let temporary = memory.reserve(quote.temporary).unwrap();
    let expected = registry.snapshot_for_compilation(control).unwrap();
    let schema = RelationalBranchRootSchemaAuthority::compile_owned(
        0,
        registry,
        &expected,
        previous.descriptor_semantics_version(),
        compilation::RetainedContracts::Extend(Some(&previous)),
        Some(funding),
        control,
    )
    .unwrap();
    drop(expected);
    drop(temporary);
    let charged = memory.charged_memory_bytes();
    assert!(charged > 0);
    let selected_registry = schema.registry_arc();
    let selected_plans = schema.aspect_plans_arc();
    assert!(selected_registry.same_allocation(&schema.registry_arc()));
    assert_eq!(selected_registry.entity_kinds.len(), 1);
    assert_eq!(selected_plans.entity_plans.len(), 1);
    drop(schema);
    assert_eq!(memory.charged_memory_bytes(), charged);
    drop(selected_registry);
    assert_eq!(memory.charged_memory_bytes(), charged);
    drop(selected_plans);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
