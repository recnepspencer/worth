pub use std::collections::BTreeMap;

pub use crate::relational_certification::ordinary_context::world::supply_chain::program_schema::schema_registry;
pub use crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::symbols::ClientKey;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::definition::DefinitionError,
    crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition,
};
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id,
    crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id,
};
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey,
};
pub use {
    worth_foundational::facade::AspectFieldLocator, worth_foundational::facade::AspectKey,
    worth_foundational::facade::AspectValue, worth_foundational::facade::FieldKey,
    worth_foundational::facade::InternedString,
};
pub use {
    worth_relational::facade::schema::RelationalSchemaRegistry,
    worth_relational::facade::schema::SchemaVersionId,
};
pub use {
    worth_relational::facade::transactions::planned_single_field_locator,
    worth_relational::facade::transactions::CreatedEntityRef,
    worth_relational::facade::transactions::EntityReference,
    worth_relational::facade::transactions::EntitySpec,
    worth_relational::facade::transactions::RelationSpec,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainProgramError {
    Definition(DefinitionError),
    Schema(worth_relational::facade::schema::SchemaRegistryError),
    UnsupportedSchemaVersion(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion),
}

#[derive(Clone, Debug)]
pub struct CompiledSupplyChainProgram {
    pub definition: SupplyChainWorldDefinition,
    pub schema_registry: RelationalSchemaRegistry,
    pub entity_specs: Vec<EntitySpec>,
    pub relation_specs: Vec<RelationSpec>,
}

impl CompiledSupplyChainProgram {
    pub fn compile(
        definition: SupplyChainWorldDefinition,
    ) -> Result<Self, SupplyChainProgramError> {
        let definition = definition
            .validate()
            .map_err(crate::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError::Definition)?;
        let schema_registry =
            schema_registry(SchemaVersionId(1)).map_err(crate::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError::Schema)?;
        let entity_specs = definition
            .entities
            .iter()
            .map(|(key, record)| entity_spec(*key, record))
            .collect::<Vec<_>>();
        let relation_specs = definition
            .relations
            .iter()
            .map(|(key, edge)| relation_spec(*key, edge.source, edge.target))
            .collect::<Vec<_>>();
        Ok(Self {
            definition,
            schema_registry,
            entity_specs,
            relation_specs,
        })
    }

    pub fn all_entity_specs(&self) -> impl Iterator<Item = &EntitySpec> {
        self.entity_specs.iter()
    }

    pub fn all_relation_specs(&self) -> impl Iterator<Item = &RelationSpec> {
        self.relation_specs.iter()
    }

    pub fn definition(&self) -> &SupplyChainWorldDefinition {
        &self.definition
    }

    pub fn schema_registry(&self) -> &RelationalSchemaRegistry {
        &self.schema_registry
    }

    pub fn schema_registry_for_version(
        &self,
        version: crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion,
    ) -> Result<RelationalSchemaRegistry, SupplyChainProgramError> {
        let version = if version == crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1 {
            SchemaVersionId(1)
        } else if version == crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2 {
            SchemaVersionId(2)
        } else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError::UnsupportedSchemaVersion(version));
        };
        schema_registry(version).map_err(crate::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError::Schema)
    }

    pub fn entity_specs(&self) -> &[EntitySpec] {
        &self.entity_specs
    }

    pub fn relation_specs(&self) -> &[RelationSpec] {
        &self.relation_specs
    }

    pub fn definition_mut_for_test(&mut self) -> &mut SupplyChainWorldDefinition {
        &mut self.definition
    }

    pub fn relation_specs_mut_for_test(&mut self) -> &mut Vec<RelationSpec> {
        &mut self.relation_specs
    }

    pub fn with_schema_registry_for_test(
        mut self,
        schema_registry: RelationalSchemaRegistry,
    ) -> Self {
        self.schema_registry = schema_registry;
        self
    }
}

pub fn entity_client_key(key: EntityKey) -> ClientKey {
    worth_relational::facade::symbols::ClientKey::raw(format!(
        "entity::{:?}::{}",
        key.kind, key.ordinal
    ))
}

pub fn relation_client_key(key: RelationKey) -> ClientKey {
    worth_relational::facade::symbols::ClientKey::raw(format!(
        "relation::{:?}::{}",
        key.kind, key.ordinal
    ))
}

pub fn entity_spec(key: EntityKey, record: &EntityRecord) -> EntitySpec {
    EntitySpec {
        partition_id: partition_for_entity_kind(key.kind),
        kind_id: entity_kind_id(key.kind),
        client_key: entity_client_key(key),
        fields: entity_fields(record),
    }
}

pub fn relation_spec(key: RelationKey, source: EntityKey, target: EntityKey) -> RelationSpec {
    RelationSpec {
        partition_id: partition_for_entity_kind(source.kind),
        kind_id: relation_kind_id(key.kind),
        client_key: relation_client_key(key),
        source: worth_relational::facade::transactions::EntityReference::Created(
            CreatedEntityRef {
                partition_id: partition_for_entity_kind(source.kind),
                kind_id: entity_kind_id(source.kind),
                client_key: entity_client_key(source),
            },
        ),
        target: worth_relational::facade::transactions::EntityReference::Created(
            CreatedEntityRef {
                partition_id: partition_for_entity_kind(target.kind),
                kind_id: entity_kind_id(target.kind),
                client_key: entity_client_key(target),
            },
        ),
        fields: worth_relational::facade::transactions::AspectFieldPatch::new(
            std::collections::BTreeMap::new(),
        ),
    }
}

pub const fn partition_for_entity_kind(kind: EntityKind) -> PartitionId {
    match kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot | crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection => worth_relational::facade::identity::PartitionId::new(1),
        _ => worth_relational::facade::identity::PartitionId::main(),
    }
}

pub fn entity_fields(record: &EntityRecord) -> AspectFieldPatch {
    let mut fields = std::collections::BTreeMap::new();
    match record {
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(value) => {
            insert_u64(&mut fields, "port_code", value.code as u64);
            insert_string(&mut fields, "name", &value.name);
            insert_debug(&mut fields, "region", value.region);
            insert_debug(&mut fields, "posture", value.posture);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(value) => {
            insert_string(&mut fields, "name", &value.name);
            insert_u64(&mut fields, "capacity", value.capacity.0 as u64);
            insert_debug(&mut fields, "posture", value.posture);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(value) => {
            insert_string(&mut fields, "name", &value.name);
            insert_u64(&mut fields, "depth", value.depth.0 as u64);
            insert_u64(&mut fields, "capacity", value.capacity.0 as u64);
            insert_debug(&mut fields, "posture", value.posture);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(value) => {
            insert_string(&mut fields, "call_sign", &value.call_sign);
            insert_debug(&mut fields, "class", value.class);
            insert_u64(&mut fields, "capacity", value.capacity.0 as u64);
            insert_debug(&mut fields, "posture", value.posture);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value) => {
            insert_debug(&mut fields, "status", value.status);
            insert_u64(&mut fields, "departure", value.departure.0 as u64);
            insert_u64(&mut fields, "arrival", value.arrival.0 as u64);
            insert_u64(&mut fields, "revision", value.revision as u64);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(value) => {
            insert_u64(&mut fields, "sequence", value.sequence as u64);
            insert_u64(&mut fields, "revision", value.revision as u64);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(value) => {
            insert_u64(&mut fields, "mass", value.mass.0 as u64);
            insert_string(&mut fields, "customer_code", &value.customer_code.0);
            insert_debug(&mut fields, "hazard", value.hazard);
            insert_debug(&mut fields, "booking", value.booking);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(value) => {
            insert_debug(&mut fields, "result", value.result);
            insert_u64(&mut fields, "minute", value.minute.0 as u64);
        }
    }
    worth_relational::facade::transactions::AspectFieldPatch::new(fields)
}

pub fn insert_string(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    name: &str,
    value: &str,
) {
    insert_value(
        fields,
        name,
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(value.to_owned()),
        ),
    );
}

pub fn insert_debug<T: std::fmt::Debug>(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    name: &str,
    value: T,
) {
    insert_string(fields, name, &format!("{value:?}"));
}

pub fn insert_u64(fields: &mut BTreeMap<AspectFieldLocator, AspectValue>, name: &str, value: u64) {
    insert_value(
        fields,
        name,
        worth_foundational::facade::AspectValue::UInt64(value),
    );
}

pub fn insert_value(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    name: &str,
    value: AspectValue,
) {
    let aspect = worth_foundational::facade::AspectKey::new(name)
        .expect("canonical Supply Chain aspect key");
    let field =
        worth_foundational::facade::FieldKey::new(name).expect("canonical Supply Chain field key");
    fields.insert(planned_single_field_locator(aspect, field), value);
}
