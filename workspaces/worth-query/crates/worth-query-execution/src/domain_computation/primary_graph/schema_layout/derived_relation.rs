//! One installed recipe joins descriptive schema meaning to a Native handle.
use super::{invalid_member, required_kind, WorthQueryPrimaryGraphInstallationDenial as Denial};
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::{
    application_derived_relation_physical_sources, ApplicationDerivedRelationNode as Node,
    ApplicationDerivedRelationSourceKind as SourceKind,
};
use worth_query_installation::facade::{
    ApplicationSchemaMember as Member, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    DerivedRelationDefinitionParts, DerivedRelationHandle, DerivedRelationIdentity,
    DerivedRelationNode, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct DerivedRelationLayout {
    pub(in crate::domain_computation::primary_graph) from: Arc<str>,
    pub(in crate::domain_computation::primary_graph) to: Arc<str>,
    pub(in crate::domain_computation::primary_graph) handle: DerivedRelationHandle,
    pub(in crate::domain_computation::primary_graph) sources:
        Arc<[(Arc<str>, KindId, KindId, KindId)]>,
    pub(in crate::domain_computation::primary_graph) operations: Arc<[Arc<str>]>,
}

pub(super) fn lower(
    registry: RelationalSchemaRegistry,
    schema: &ErasedApplicationSchemaDeclaration,
    basis: (&SchemaId, SchemaVersionId),
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
) -> Result<
    (
        RelationalSchemaRegistry,
        BTreeMap<String, DerivedRelationLayout>,
    ),
    Denial,
> {
    let identity = |name: &str| DerivedRelationIdentity {
        schema: basis.0.clone(),
        version: basis.1,
        name: name.to_owned(),
    };
    let mut definitions = Vec::new();
    for member in schema.members() {
        let Member::DerivedRelation { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        let nodes = parts
            .expression
            .nodes
            .iter()
            .map(|node| {
                Ok(match node {
                    Node::Source(source) => match source.kind {
                        SourceKind::Stored => DerivedRelationNode::Stored {
                            relation: required_kind(relations, &source.relation)?,
                            from: required_kind(entities, &source.from)?,
                            to: required_kind(entities, &source.to)?,
                        },
                        SourceKind::Derived => {
                            DerivedRelationNode::Derived(identity(&source.relation))
                        }
                    },
                    Node::Compose { left, right } => DerivedRelationNode::Compose {
                        left: *left,
                        right: *right,
                    },
                    Node::Reverse { input } => DerivedRelationNode::Reverse { input: *input },
                    Node::WithoutSelfEdges { input } => {
                        DerivedRelationNode::WithoutSelfEdges { input: *input }
                    }
                })
            })
            .collect::<Result<Vec<_>, Denial>>()?;
        definitions.push(DerivedRelationDefinitionParts {
            identity: identity(&parts.relation),
            from: required_kind(entities, &parts.from)?,
            to: required_kind(entities, &parts.to)?,
            nodes,
        });
    }
    let registry = registry
        .register_derived_relations(definitions)
        .map_err(|cause| invalid_member(&format!("derived Native registration: {cause:?}")))?;
    let mut layouts = BTreeMap::new();
    for member in schema.members() {
        let Member::DerivedRelation { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        let sources =
            application_derived_relation_physical_sources(schema.members(), &parts.relation)
                .map_err(|_| invalid_member(&parts.relation))?
                .into_iter()
                .map(|source| {
                    Ok((
                        Arc::from(source.relation.as_str()),
                        required_kind(relations, &source.relation)?,
                        required_kind(entities, &source.from)?,
                        required_kind(entities, &source.to)?,
                    ))
                })
                .collect::<Result<Arc<[_]>, Denial>>()?;
        layouts.insert(
            parts.relation.clone(),
            DerivedRelationLayout {
                from: Arc::from(parts.from.as_str()),
                to: Arc::from(parts.to.as_str()),
                sources,
                operations: schema
                    .members()
                    .iter()
                    .filter_map(|member| match member {
                        Member::OperationDerivedRead {
                            operation,
                            relation,
                        } if relation == &parts.relation => Some(Arc::from(operation.as_str())),
                        _ => None,
                    })
                    .collect(),
                handle: registry
                    .derived_relation(&identity(&parts.relation))
                    .expect("registered recipe"),
            },
        );
    }
    Ok((registry, layouts))
}

impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn derived_relation(
        &self,
        name: &str,
    ) -> Option<&DerivedRelationLayout> {
        self.derived_relations.get(name)
    }
}
