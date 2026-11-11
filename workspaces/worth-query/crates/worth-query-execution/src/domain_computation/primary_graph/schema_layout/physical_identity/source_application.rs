//! Actual A contracts select physical application roles before any B lowering.
use super::super::registry_lowering::{
    register_application_relation, register_entity, LoweredApplicationContractBindings,
};
use super::*;
use worth_query_installation::facade::ApplicationSchemaMember;

pub(super) fn resolve(
    schema: &ErasedApplicationSchemaDeclaration,
    contracts: &LoweredApplicationContractBindings,
    source: &RelationalSchemaRegistry,
    snapshot: &SchemaAuthoritySnapshot,
) -> Result<
    (BTreeMap<String, KindId>, BTreeMap<String, KindId>),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let mut entities = BTreeMap::new();
    let mut relations = BTreeMap::new();
    let mut physical = RelationalSchemaRegistry::new();
    for member in schema.members() {
        let ApplicationSchemaMember::Entity { entity } = member else {
            continue;
        };
        let mut selected = None;
        for candidate in snapshot
            .entity_kinds
            .iter()
            .filter(|candidate| &candidate.kind_name == entity)
        {
            let actual = source
                .entity_registration(candidate.kind_id)
                .map_err(relational_schema_denial)?;
            let expected = register_entity(
                RelationalSchemaRegistry::new(),
                &actual.schema_id,
                actual.schema_version_id,
                entity,
                actual.kind_id,
                contracts.by_entity.get(entity).cloned().unwrap_or_default(),
            )?;
            if expected
                .entity_registration(actual.kind_id)
                .map_err(relational_schema_denial)?
                == actual
                && selected.replace(actual.clone()).is_some()
            {
                return Err(invalid_member("ambiguous original application entity role"));
            }
        }
        let selected = selected
            .ok_or_else(|| invalid_member("original application entity role is missing"))?;
        entities.insert(entity.clone(), selected.kind_id);
        physical = physical
            .register_entity_kind(selected)
            .map_err(relational_schema_denial)?;
    }
    for member in schema.members() {
        let ApplicationSchemaMember::Relation {
            relation,
            from,
            to,
            integrity,
        } = member
        else {
            continue;
        };
        let mut selected = None;
        for candidate in snapshot
            .relation_kinds
            .iter()
            .filter(|candidate| &candidate.kind_name == relation)
        {
            let actual = source
                .relation_registration(candidate.kind_id)
                .map_err(relational_schema_denial)?;
            let expected = register_application_relation(
                physical.clone(),
                &actual.schema_id,
                actual.schema_version_id,
                relation,
                actual.kind_id,
                required_kind(&entities, from)?,
                required_kind(&entities, to)?,
                integrity,
                &entities,
                contracts
                    .by_relation
                    .get(relation)
                    .cloned()
                    .unwrap_or_default(),
            )?;
            if expected
                .relation_registration(actual.kind_id)
                .map_err(relational_schema_denial)?
                == actual
                && selected.replace(actual.kind_id).is_some()
            {
                return Err(invalid_member(
                    "ambiguous original application relation role",
                ));
            }
        }
        relations.insert(
            relation.clone(),
            selected
                .ok_or_else(|| invalid_member("original application relation role is missing"))?,
        );
    }
    Ok((entities, relations))
}
