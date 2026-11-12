//! Exact schema readmission compiles the declared kind and matches authority.
use super::*;
use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, AspectKey, ScalarAspectType,
};

#[test]
fn exact_readmission_compiles_declared_kind_with_matching_authority() {
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
    let control = crate::schema::SchemaCompilationControl::Configuration;
    let mut contracts = Vec::with_capacity(17);
    contracts.push(AspectContract::scalar(
        AspectKey::new("retained-historical-contract").unwrap(),
        AspectIdentity(7),
        AspectContractRevision(1),
        ScalarAspectType::String,
    ));
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
}
