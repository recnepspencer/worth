//! Bind each supported endpoint projection to an actual Native schema-owned route.
use super::*;
use worth_foundational::facade::encode_canonical_text_hex;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryResultShape, ApplicationQueryResultTraversalDirection,
};
use worth_relational::facade::runtime::ManagedIndexSourceProjectionRoute;

pub(super) fn lower(
    sources: super::super::source_registration::SourceRegistration<'_>,
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
        let slot = relation.slot_identity();
        let route_identity = DerivedRelationIdentity {
            schema: identity.schema.clone(),
            version: identity.version,
            name: projection_relation_name(&identity.name, slot.as_str()),
        };
        registry = sources
            .derived(
                registry,
                vec![DerivedRelationDefinitionParts {
                    identity: route_identity.clone(),
                    from: parent.into(),
                    to: endpoint.into(),
                    nodes,
                    laws: None,
                }],
            )
            .map_err(|_| unsupported(source.name()))?;
        let path = registry
            .derived_relation(&route_identity)
            .ok_or_else(|| unsupported(source.name()))?;
        let index = routes.len();
        let columns = scalar_fields(relation.nested_shape(), fields, source.name())?;
        let name = slot.as_str().to_owned();
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

fn projection_relation_name(source_name: &str, slot: &str) -> String {
    let encoded_slot = encode_canonical_text_hex(slot);
    format!("projection::{source_name}::{encoded_slot}")
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
            if field.entity() != shape.root_entity() {
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

#[test]
fn projection_relation_names_are_valid_and_distinct_for_dotted_slots() {
    let mut names = std::collections::BTreeSet::new();
    for slot in [
        "a.b",
        "a_b",
        "612e62",
        "面.境界",
        "nested-catalog.parent-relation.v1",
    ] {
        let name = projection_relation_name("managed-index-source::catalog", slot);
        assert!(
            worth_foundational::is_valid_simple_identifier(&name),
            "{name}"
        );
        assert!(names.insert(name), "duplicate route name for {slot}");
    }
}
