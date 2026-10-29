//! A home serves the schema its recovered main head records (D5): the initial
//! schema until a durable schema transition, its target after. Opening with
//! any other declared registry — the other side of the transition, or the
//! same kind contracts under another version label — refuses before service
//! and leaves the home intact. Each branch still recovers under the schema
//! boundary its own commits recorded. A registry with other kind contracts
//! refuses before service.

pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::KindId};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntimeApi};
pub(crate) use {worth_relational::facade::schema::EntityKindRegistration, worth_relational::facade::schema::KindAspectContractDeclarations, worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaVersionId};
pub(crate) use {worth_store_relational::RelationalStoreHome, worth_store_relational::RelationalStoreHomeOpenRefusal, worth_store_relational::RelationalStoreHomeRequest};

pub(crate) use {crate::durable_run::apply, crate::durable_run::assert_holds, crate::durable_run::open, crate::durable_run::open_declaring, crate::durable_run::recorded_schema, crate::durable_run::served_schema, crate::durable_run::steps_after, crate::durable_run::steps_before};
pub(crate) use {crate::schema_transition::basis_of, crate::schema_transition::SOURCE, crate::schema_transition::TARGET};
pub(crate) use {crate::workload, crate::workload::Script};

pub(crate) const SEED: u64 = 0x5eed_0202;
pub(crate) const COMMITS: usize = 12;
/// The transition is the commit at script step `2 * COMMITS / 3`.
pub(crate) const THROUGH_TRANSITION: usize = 2 * COMMITS / 3 + 1;
/// The kind contracts of the home's schemas under a label it never used.
pub(crate) const OTHER_LABEL: SchemaVersionId = SchemaVersionId(7);

pub(crate) fn branch(name: &str) -> BranchId {
    BranchId(name.to_owned())
}

pub(crate) fn foreign_registry() -> RelationalSchemaRegistry {
    worth_relational::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "other.entity".to_string(),
            schema_id: SchemaId("other".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::default(),
        })
        .expect("the foreign registry")
}