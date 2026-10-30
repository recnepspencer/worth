//! Compile the actual installed Source into Native's existing path owner.
use super::{
    required_kind, WorthQueryPrimaryFieldLayout, WorthQueryPrimaryGraphInstallationDenial as Denial,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AspectFieldLocator, RelationDirection};
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
        let nodes = path(source, entities, relations)?;
        let projection = projection(source, fields)?;
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
            .bind_managed_index_source(
                &handle,
                projection
                    .into_iter()
                    .map(|locator| ManagedIndexSourceField { locator })
                    .collect(),
            )
            .map_err(|_| unsupported(source_name))?;
        programs.insert(definition.parts().name.clone(), program);
    }
    Ok((registry, programs))
}

fn path(
    source: &ErasedApplicationQueryDefinition,
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
) -> Result<Vec<Node>, Denial> {
    if !source.predicates().is_empty()
        || !source.parameters().is_empty()
        || source.root_paths().is_empty()
        || source.disclosure().posture() != worth_query_declaration::facade::application_query::ApplicationQueryDisclosurePosture::Public
        || !source.disclosure().rules().is_empty()
    {
        return Err(unsupported(source.name()));
    }
    let mut nodes = Vec::new();
    let mut union = None;
    for path in source.root_paths() {
        if path.start_entity() != source.scope_entity()
            || path.terminal_entity() != source.root_entity()
            || !path.guards().is_empty()
            || path.steps().is_empty()
        {
            return Err(unsupported(source.name()));
        }
        let mut composed = None;
        for step in path.steps() {
            if step.traversal() != ApplicationQueryRootPathTraversal::OneHop {
                return Err(unsupported(source.name()));
            }
            let leaf = nodes.len();
            nodes.push(Node::Stored {
                relation: required_kind(relations, step.relation())?,
                from: required_kind(entities, step.from())?,
                to: required_kind(entities, step.to())?,
            });
            let leaf = if step.direction() == RelationDirection::Reverse {
                nodes.push(Node::Reverse { input: leaf });
                nodes.len() - 1
            } else {
                leaf
            };
            composed = Some(if let Some(left) = composed {
                nodes.push(Node::Compose { left, right: leaf });
                nodes.len() - 1
            } else {
                leaf
            });
        }
        let right = composed.ok_or_else(|| unsupported(source.name()))?;
        union = Some(if let Some(left) = union {
            nodes.push(Node::Union { left, right });
            nodes.len() - 1
        } else {
            right
        });
    }
    Ok(nodes)
}

fn projection(
    source: &ErasedApplicationQueryDefinition,
    fields: &BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
) -> Result<Vec<AspectFieldLocator>, Denial> {
    if !source.result_shape().relations().is_empty() {
        return Err(unsupported(source.name()));
    }
    source
        .result_shape()
        .fields()
        .iter()
        .map(|field| {
            if field.entity() != source.root_entity() {
                return Err(unsupported(source.name()));
            }
            fields
                .get(&(
                    field.entity().to_owned(),
                    field.aspect().to_owned(),
                    field.field().to_owned(),
                ))
                .map(|layout| layout.locator.clone())
                .ok_or_else(|| unsupported(source.name()))
        })
        .collect()
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
