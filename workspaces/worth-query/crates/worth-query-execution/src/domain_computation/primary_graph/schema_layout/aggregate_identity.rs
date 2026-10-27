//! Query declaration binds one Native aggregate owner, outside the application catalog.
use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
    WorthQueryInstalledApplicationSchemaContractCatalog,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    register_aggregate_identity_storage, register_aggregate_membership,
    AggregateIdentityContractDeclaration, AggregateIdentitySource, AggregateIdentityStorage,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

use super::{
    kind_space_exhausted, relational_schema_denial, required_kind,
    WorthQueryPrimaryGraphInstallationDenial,
};
#[cfg(test)]
mod tests;

pub(super) fn lower(
    mut registry: RelationalSchemaRegistry,
    schema: &ErasedApplicationSchemaDeclaration,
    contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    entity_kinds: &BTreeMap<String, KindId>,
    relation_kinds: &BTreeMap<String, KindId>,
) -> Result<RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial> {
    let definitions: BTreeMap<_, _> = schema
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Relation {
                relation,
                from,
                integrity,
                ..
            } if integrity.orbit => Some((relation.as_str(), from.as_str())),
            _ => None,
        })
        .collect();
    if definitions.is_empty() {
        return Ok(registry);
    }
    let authority = registry.authority_snapshot();
    let last_kind = authority
        .entity_kinds
        .iter()
        .map(|kind| kind.kind_id)
        .chain(authority.relation_kinds.iter().map(|kind| kind.kind_id))
        .map(|kind| kind.0)
        .max()
        .unwrap_or(0);
    let next_kind = |offset: u32| {
        last_kind
            .checked_add(offset)
            .map(KindId)
            .ok_or_else(kind_space_exhausted)
    };
    let [aggregate_aspect, lineage_aspect] =
        super::platform_identity_allocator::allocate_aggregate_aspect_identities(contracts)?;
    let storage = AggregateIdentityStorage {
        aggregate_kind: next_kind(1)?,
        lineage_kind: next_kind(2)?,
        aggregate_aspect,
        lineage_aspect,
    };
    registry = register_aggregate_identity_storage(registry, schema_id, version, storage)
        .map_err(relational_schema_denial)?;
    for (index, (relation, member)) in definitions.into_iter().enumerate() {
        let offset = u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(3))
            .ok_or_else(kind_space_exhausted)?;
        registry = register_aggregate_membership(
            registry,
            schema_id,
            version,
            AggregateIdentityContractDeclaration {
                definition: definition_identity(schema, relation),
                source: AggregateIdentitySource::Orbit(required_kind(relation_kinds, relation)?),
                member_kind: required_kind(entity_kinds, member)?,
                membership_kind: next_kind(offset)?,
                storage,
            },
        )
        .map_err(relational_schema_denial)?;
    }
    Ok(registry)
}

fn definition_identity(
    schema: &ErasedApplicationSchemaDeclaration,
    relation: &str,
) -> AggregateDefinitionIdentity {
    let mut hash = Sha256::new();
    hash.update(b"WORTH_QUERY_ORBIT_DEFINITION_V1\0");
    for name in [schema.owner(), schema.name(), relation] {
        hash.update((name.len() as u64).to_be_bytes());
        hash.update(name.as_bytes());
    }
    hash.update(schema.major().to_be_bytes());
    AggregateDefinitionIdentity::from_bytes(hash.finalize().into())
}
