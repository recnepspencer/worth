//! Bind each supported endpoint projection to an actual Native schema-owned route.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryResultShape, ApplicationQueryResultTraversalDirection,
};
use worth_relational::facade::runtime::ManagedIndexSourceProjectionRoute;

pub(super) fn lower(
    mut registry: RelationalSchemaRegistry,
    source: &ErasedApplicationQueryDefinition,
    identity: &DerivedRelationIdentity,
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
    fields: &BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
) -> Result<
    (
        RelationalSchemaRegistry,
        Vec<ManagedIndexSourceField>,
        Vec<ManagedIndexSourceProjectionRoute>,
    ),
    Denial,
> {
    if source.ordering().iter().any(|ordering| {
        !source
            .result_shape()
            .fields()
            .iter()
            .any(|field| field.slot_identity().as_str() == ordering.slot_type())
    }) {
        return Err(unsupported(source.name()));
    }
    let root_fields = scalar_fields(source.result_shape(), fields, source.name())?;
    let mut routes = Vec::new();
    // Preorder keeps every Native parent route earlier than its descendants.
    let mut stack = vec![(source.result_shape(), None, 0usize)];
    while let Some((shape, parent_route, position)) = stack.pop() {
        let Some(relation) = shape.relations().get(position) else {
            continue;
        };
        stack.push((shape, parent_route, position + 1));
        if relation.predicate().is_some() {
            return Err(unsupported(source.name()));
        }
        let from = required_kind(entities, relation.from())?;
        let to = required_kind(entities, relation.to())?;
        let mut nodes = vec![Node::Stored {
            relation: required_kind(relations, relation.relation())?,
            from,
            to,
        }];
        let reverse = relation.direction() == ApplicationQueryResultTraversalDirection::Reverse;
        if reverse {
            nodes.push(Node::Reverse { input: 0 });
        }
        let (parent, endpoint) = if reverse { (to, from) } else { (from, to) };
        if parent != required_kind(entities, shape.root_entity())?
            || endpoint != required_kind(entities, relation.nested_shape().root_entity())?
        {
            return Err(unsupported(source.name()));
        }
        let name = relation.slot_identity().as_str().to_owned();
        let route_identity = DerivedRelationIdentity {
            schema: identity.schema.clone(),
            version: identity.version,
            name: format!("{}::projection::{name}", identity.name),
        };
        registry = registry
            .register_derived_relations(vec![DerivedRelationDefinitionParts {
                identity: route_identity.clone(),
                from: parent.into(),
                to: endpoint.into(),
                nodes,
                laws: None,
            }])
            .map_err(|_| unsupported(source.name()))?;
        let path = registry
            .derived_relation(&route_identity)
            .ok_or_else(|| unsupported(source.name()))?;
        let index = routes.len();
        let columns = scalar_fields(relation.nested_shape(), fields, source.name())?;
        let route = match parent_route {
            None => ManagedIndexSourceProjectionRoute::new(name, path, columns),
            Some(parent) => ManagedIndexSourceProjectionRoute::nested(name, parent, path, columns),
        }
        .map_err(|_| unsupported(source.name()))?;
        routes.push(route);
        stack.push((relation.nested_shape(), Some(index), 0));
    }
    Ok((registry, root_fields, routes))
}

fn scalar_fields(
    shape: &ApplicationQueryResultShape,
    fields: &BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
    source: &str,
) -> Result<Vec<ManagedIndexSourceField>, Denial> {
    shape
        .fields()
        .iter()
        .map(|field| {
            // This Native Source receiver carries child-field absence, not
            // the lossless parent-aspect absence used by ordinary projections.
            if field.entity() != shape.root_entity()
                || field.presence() == worth_query_declaration::facade::application_query::ApplicationQueryResultFieldPresence::OptionalAspect
            {
                return Err(unsupported(source));
            }
            fields
                .get(&(
                    field.entity().to_owned(),
                    field.aspect().to_owned(),
                    field.field().to_owned(),
                ))
                .map(|layout| ManagedIndexSourceField {
                    locator: layout.locator.clone(),
                })
                .ok_or_else(|| unsupported(source))
        })
        .collect()
}
