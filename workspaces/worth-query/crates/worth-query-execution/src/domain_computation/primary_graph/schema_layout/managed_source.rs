//! Compile the actual installed Source into Native's existing path owner.
use super::{
    required_kind, WorthQueryPrimaryFieldLayout, WorthQueryPrimaryGraphInstallationDenial as Denial,
};
use std::collections::{BTreeMap, BTreeSet};

mod membership;
mod projection;
use worth_foundational::facade::RelationDirection;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathTraversal, ErasedApplicationQueryDefinition,
};
use worth_query_installation::facade::{
    ApplicationSchemaMember as Member, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::{ManagedIndexSourceField, ManagedIndexSourceProgram};
use worth_relational::facade::schema::{
    DerivedRelationDefinitionParts, DerivedRelationIdentity, DerivedRelationNode as Node,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

pub(super) fn lower(
    mut registry: RelationalSchemaRegistry,
    schema: &ErasedApplicationSchemaDeclaration,
    basis: (&SchemaId, SchemaVersionId),
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
    fields: &BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
) -> Result<
    (
        RelationalSchemaRegistry,
        BTreeMap<String, ManagedIndexSourceProgram>,
    ),
    Denial,
> {
    let required: BTreeSet<_> = schema
        .members()
        .iter()
        .filter_map(|member| match member {
            Member::InvariantManagedIndexSource { requirement, .. } => Some(requirement.index()),
            _ => None,
        })
        .collect();
    let mut programs = BTreeMap::new();
    for member in schema.members() {
        let Member::ManagedIndex { definition } = member else {
            continue;
        };
        if !required.contains(definition.parts().name.as_str()) {
            continue;
        }
        let source_name = &definition.parts().source.binding.parts().query_name;
        let source = schema
            .members()
            .iter()
            .find_map(|member| match member {
                Member::ApplicationQuery { definition } if definition.name() == source_name => {
                    Some(definition)
                }
                _ => None,
            })
            .ok_or_else(|| unsupported(source_name))?;
        let identity = DerivedRelationIdentity {
            schema: basis.0.clone(),
            version: basis.1,
            name: format!("managed-index-source::{}", definition.parts().name),
        };
        let nodes = membership::path(source, entities, relations)?;
        let (next_registry, projection, routes) =
            projection::lower(registry, source, &identity, entities, relations, fields)?;
        registry = next_registry;
        registry = registry
            .register_derived_relations(vec![DerivedRelationDefinitionParts {
                identity: identity.clone(),
                from: required_kind(entities, source.scope_entity())?.into(),
                to: required_kind(entities, source.root_entity())?.into(),
                nodes,
                laws: None,
            }])
            .map_err(|_| unsupported(source_name))?;
        let handle = registry
            .derived_relation(&identity)
            .ok_or_else(|| unsupported(source_name))?;
        let program = registry
            .bind_managed_index_source(&handle, projection, routes)
            .map_err(|_| unsupported(source_name))?;
        programs.insert(definition.parts().name.clone(), program);
    }
    Ok((registry, programs))
}

fn unsupported(name: &str) -> Denial {
    super::super::WorthQueryPrimaryGraphInstallationDenial::new(
        super::super::WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedInvariantSource,
        name,
    )
}

impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn managed_source(
        &self,
        index: &str,
    ) -> Option<&ManagedIndexSourceProgram> {
        self.managed_sources.get(index)
    }
}
