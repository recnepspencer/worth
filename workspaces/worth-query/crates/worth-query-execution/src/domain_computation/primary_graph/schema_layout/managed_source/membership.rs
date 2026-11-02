//! Lower complete scope membership through Native's existing path grammar.
use super::*;

pub(super) fn path(
    source: &ErasedApplicationQueryDefinition,
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
    registry: &RelationalSchemaRegistry,
    basis: (&SchemaId, SchemaVersionId),
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
            nodes.push(match step.source_kind() {
                worth_query_declaration::facade::application_query::ApplicationQueryRootPathSourceKind::Stored => Node::Stored {
                    relation: required_kind(relations, step.relation())?,
                    from: required_kind(entities, step.from())?,
                    to: required_kind(entities, step.to())?,
                },
                worth_query_declaration::facade::application_query::ApplicationQueryRootPathSourceKind::Derived => {
                    let identity = DerivedRelationIdentity { schema: basis.0.clone(), version: basis.1, name: step.relation().into() };
                    let handle = registry.derived_relation(&identity).ok_or_else(|| unsupported(source.name()))?;
                    if handle.parts().from != required_kind(entities, step.from())?.into()
                        || handle.parts().to != required_kind(entities, step.to())?.into() {
                        return Err(unsupported(source.name()));
                    }
                    Node::Derived(handle.parts().identity.clone())
                }
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
