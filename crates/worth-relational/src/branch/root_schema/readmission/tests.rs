//! Exact readmission retains its actual compiler bill through escaped children.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, AspectKey, ScalarAspectType,
};

#[test]
fn exact_readmission_retains_original_compiler_funding_until_last_child() {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(crate::schema::data::EntityKindRegistration {
            kind_id: crate::identity::data::KindId(91),
            kind_name: "exact.schema.kind".to_owned(),
            schema_id: crate::schema::data::SchemaId("exact.schema".to_owned()),
            schema_version_id: crate::schema::data::SchemaVersionId(2),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let expected = registry.authority_snapshot();
    let semantics =
        crate::schema::data::runtime_descriptor_semantics_policy().current_write_version();
    let zero = SerialMemoryBudget::new(Some(0));
    let zero_host =
        SerialRequest::from_memory(zero.clone(), CancellationSource::new().token(), None);
    assert!(RelationalBranchRootSchemaAuthority::readmit_exact(
        1,
        registry.clone(),
        Vec::new(),
        &expected,
        semantics,
        crate::schema::SchemaCompilationControl::Request(ExecutionRequest::serial(&zero_host)),
    )
    .is_err());
    assert_eq!(zero.charged_memory_bytes(), 0);

    let memory = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let control = crate::schema::SchemaCompilationControl::Request(request);
    let mut contracts = Vec::with_capacity(17);
    contracts.push(AspectContract::scalar(
        AspectKey::new("retained-historical-contract").unwrap(),
        AspectIdentity(7),
        AspectContractRevision(1),
        ScalarAspectType::String,
    ));
    let quote =
        compilation_quote::SchemaCompilationQuote::for_exact(&registry, &contracts, control)
            .unwrap();
    let schema = RelationalBranchRootSchemaAuthority::readmit_exact(
        2, registry, contracts, &expected, semantics, control,
    )
    .unwrap()
    .unwrap();
    let registry = schema.registry_arc();
    let plans = schema.aspect_plans_arc();
    assert_eq!(registry.entity_kinds.len(), 1);
    assert_eq!(plans.entity_plans.len(), 1);
    assert_eq!(schema.authority_digest(), expected.digest());
    drop(schema);
    drop(registry);
    assert_eq!(memory.charged_memory_bytes(), quote.retained);
    drop(plans);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
