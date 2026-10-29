pub(crate) use std::collections::BTreeMap;

pub(crate) use super::definition::{DefinitionError, SupplyChainWorldDefinition};
pub(crate) use super::program_schema::schema_registry;
pub(crate) use super::schema::EntityRecord;
pub(crate) use super::schema_vocabulary::{entity_kind_id, relation_kind_id};
pub(crate) use super::semantic_key::{EntityKey, EntityKind, RelationKey};
pub(crate) use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, FieldKey, InternedString,
};
pub(crate) use worth_relational::facade::identity::PartitionId;
pub(crate) use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaVersionId};
pub(crate) use worth_relational::facade::symbols::ClientKey;
pub(crate) use worth_relational::facade::transactions::AspectFieldPatch;
pub(crate) use worth_relational::facade::transactions::{
    planned_single_field_locator, CreatedEntityRef, EntityReference, EntitySpec, RelationSpec,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SupplyChainProgramError {
    Definition(DefinitionError),
    Schema(worth_relational::facade::schema::SchemaRegistryError),
    UnsupportedSchemaVersion(super::SchemaVersion),
}

#[derive(Clone, Debug)]
pub(crate) struct CompiledSupplyChainProgram {
    pub(crate) definition: SupplyChainWorldDefinition,
    pub(crate) schema_registry: RelationalSchemaRegistry,
    pub(crate) entity_specs: Vec<EntitySpec>,
    pub(crate) relation_specs: Vec<RelationSpec>,
}

impl CompiledSupplyChainProgram {
    pub(crate) fn compile(
        definition: SupplyChainWorldDefinition,
    ) -> Result<Self, SupplyChainProgramError> {
        let definition = definition
            .validate()
            .map_err(SupplyChainProgramError::Definition)?;
        let schema_registry =
            schema_registry(SchemaVersionId(1)).map_err(SupplyChainProgramError::Schema)?;
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

    pub(crate) fn all_entity_specs(&self) -> impl Iterator<Item = &EntitySpec> {
        self.entity_specs.iter()
    }

    pub(crate) fn all_relation_specs(&self) -> impl Iterator<Item = &RelationSpec> {
        self.relation_specs.iter()
    }

    pub(crate) fn definition(&self) -> &SupplyChainWorldDefinition {
        &self.definition
    }

    pub(crate) fn schema_registry(&self) -> &RelationalSchemaRegistry {
        &self.schema_registry
    }

    pub(crate) fn schema_registry_for_version(
        &self,
        version: super::SchemaVersion,
    ) -> Result<RelationalSchemaRegistry, SupplyChainProgramError> {
        let version = if version == super::SchemaVersion::V1 {
            SchemaVersionId(1)
        } else if version == super::SchemaVersion::V2 {
            SchemaVersionId(2)
        } else {
            return Err(SupplyChainProgramError::UnsupportedSchemaVersion(version));
        };
        schema_registry(version).map_err(SupplyChainProgramError::Schema)
    }

    pub(crate) fn entity_specs(&self) -> &[EntitySpec] {
        &self.entity_specs
    }

    pub(crate) fn relation_specs(&self) -> &[RelationSpec] {
        &self.relation_specs
    }

    pub(crate) fn definition_mut_for_test(&mut self) -> &mut SupplyChainWorldDefinition {
        &mut self.definition
    }

    pub(crate) fn relation_specs_mut_for_test(&mut self) -> &mut Vec<RelationSpec> {
        &mut self.relation_specs
    }

    pub(crate) fn with_schema_registry_for_test(
        mut self,
        schema_registry: RelationalSchemaRegistry,
    ) -> Self {
        self.schema_registry = schema_registry;
        self
    }
}

pub(crate) fn entity_client_key(key: EntityKey) -> ClientKey {
    ClientKey::raw(format!("entity::{:?}::{}", key.kind, key.ordinal))
}

pub(crate) fn relation_client_key(key: RelationKey) -> ClientKey {
    ClientKey::raw(format!("relation::{:?}::{}", key.kind, key.ordinal))
}

pub(crate) fn entity_spec(key: EntityKey, record: &EntityRecord) -> EntitySpec {
    EntitySpec {
        partition_id: partition_for_entity_kind(key.kind),
        kind_id: entity_kind_id(key.kind),
        client_key: entity_client_key(key),
        fields: entity_fields(record),
    }
}

pub(crate) fn relation_spec(
    key: RelationKey,
    source: EntityKey,
    target: EntityKey,
) -> RelationSpec {
    RelationSpec {
        partition_id: partition_for_entity_kind(source.kind),
        kind_id: relation_kind_id(key.kind),
        client_key: relation_client_key(key),
        source: EntityReference::Created(CreatedEntityRef {
            partition_id: partition_for_entity_kind(source.kind),
            kind_id: entity_kind_id(source.kind),
            client_key: entity_client_key(source),
        }),
        target: EntityReference::Created(CreatedEntityRef {
            partition_id: partition_for_entity_kind(target.kind),
            kind_id: entity_kind_id(target.kind),
            client_key: entity_client_key(target),
        }),
        fields: AspectFieldPatch::new(BTreeMap::new()),
    }
}

pub(crate) const fn partition_for_entity_kind(kind: EntityKind) -> PartitionId {
    match kind {
        EntityKind::CargoLot | EntityKind::Inspection => PartitionId::new(1),
        _ => PartitionId::main(),
    }
}

fn entity_fields(record: &EntityRecord) -> AspectFieldPatch {
    let mut fields = BTreeMap::new();
    match record {
        EntityRecord::Port(value) => {
            insert_u64(&mut fields, "port_code", value.code as u64);
            insert_string(&mut fields, "name", &value.name);
            insert_debug(&mut fields, "region", value.region);
            insert_debug(&mut fields, "posture", value.posture);
        }
        EntityRecord::Terminal(value) => {
            insert_string(&mut fields, "name", &value.name);
            insert_u64(&mut fields, "capacity", value.capacity.0 as u64);
            insert_debug(&mut fields, "posture", value.posture);
        }
        EntityRecord::Berth(value) => {
            insert_string(&mut fields, "name", &value.name);
            insert_u64(&mut fields, "depth", value.depth.0 as u64);
            insert_u64(&mut fields, "capacity", value.capacity.0 as u64);
            insert_debug(&mut fields, "posture", value.posture);
        }
        EntityRecord::Vessel(value) => {
            insert_string(&mut fields, "call_sign", &value.call_sign);
            insert_debug(&mut fields, "class", value.class);
            insert_u64(&mut fields, "capacity", value.capacity.0 as u64);
            insert_debug(&mut fields, "posture", value.posture);
        }
        EntityRecord::Voyage(value) => {
            insert_debug(&mut fields, "status", value.status);
            insert_u64(&mut fields, "departure", value.departure.0 as u64);
            insert_u64(&mut fields, "arrival", value.arrival.0 as u64);
            insert_u64(&mut fields, "revision", value.revision as u64);
        }
        EntityRecord::PortCall(value) => {
            insert_u64(&mut fields, "sequence", value.sequence as u64);
            insert_u64(&mut fields, "revision", value.revision as u64);
        }
        EntityRecord::CargoLot(value) => {
            insert_u64(&mut fields, "mass", value.mass.0 as u64);
            insert_string(&mut fields, "customer_code", &value.customer_code.0);
            insert_debug(&mut fields, "hazard", value.hazard);
            insert_debug(&mut fields, "booking", value.booking);
        }
        EntityRecord::Inspection(value) => {
            insert_debug(&mut fields, "result", value.result);
            insert_u64(&mut fields, "minute", value.minute.0 as u64);
        }
    }
    AspectFieldPatch::new(fields)
}

fn insert_string(fields: &mut BTreeMap<AspectFieldLocator, AspectValue>, name: &str, value: &str) {
    insert_value(
        fields,
        name,
        AspectValue::String(InternedString::Raw(value.to_owned())),
    );
}

fn insert_debug<T: std::fmt::Debug>(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    name: &str,
    value: T,
) {
    insert_string(fields, name, &format!("{value:?}"));
}

fn insert_u64(fields: &mut BTreeMap<AspectFieldLocator, AspectValue>, name: &str, value: u64) {
    insert_value(fields, name, AspectValue::UInt64(value));
}

fn insert_value(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    name: &str,
    value: AspectValue,
) {
    let aspect = AspectKey::new(name).expect("canonical Supply Chain aspect key");
    let field = FieldKey::new(name).expect("canonical Supply Chain field key");
    fields.insert(planned_single_field_locator(aspect, field), value);
}
