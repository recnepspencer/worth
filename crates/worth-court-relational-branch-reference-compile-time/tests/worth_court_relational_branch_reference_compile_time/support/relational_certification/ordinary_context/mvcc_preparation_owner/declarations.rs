pub(crate) use {worth_relational::facade::identity::KindId, worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::mvcc::RelationalTransactionIntent};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntimeApi};
pub(crate) use {worth_relational::facade::schema::EntityKindRegistration, worth_relational::facade::schema::KindAspectContractDeclarations, worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaVersionId};
pub(crate) use {worth_relational::facade::symbols::ClientKey};
pub(crate) use {worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn installed_entity_kind() -> EntityKindRegistration {
    EntityKindRegistration {
        kind_id: KindId(7),
        kind_name: "production-held-port-installed".to_owned(),
        schema_id: SchemaId("production-preparation-owner".to_owned()),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::default(),
    }
}