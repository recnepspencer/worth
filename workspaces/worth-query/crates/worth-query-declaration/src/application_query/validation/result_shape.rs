use std::collections::BTreeSet;

use crate::application_query::{
    ApplicationQueryCardinality, ApplicationQueryResultField, ApplicationQueryResultRelation,
    ApplicationQueryResultShape, ApplicationQueryResultTraversalDirection,
};

pub(super) fn continuation_parent_path_is_exact(
    shape: &ApplicationQueryResultShape,
    target_slot_type: &str,
) -> Option<bool> {
    for relation in shape.relations() {
        if relation.slot_type() == target_slot_type {
            return Some(true);
        }
        if let Some(descendant_is_exact) =
            continuation_parent_path_is_exact(relation.nested_shape(), target_slot_type)
        {
            return Some(
                descendant_is_exact
                    && relation.cardinality() == ApplicationQueryCardinality::ExactlyOne,
            );
        }
    }
    None
}

pub(super) fn query_is_consistent(shape: &ApplicationQueryResultShape, query_type: &str) -> bool {
    shape.query_type() == query_type
        && shape
            .fields()
            .iter()
            .all(|field| field.query_type() == query_type)
        && shape.relations().iter().all(|relation| {
            relation.query_type() == query_type
                && query_is_consistent(relation.nested_shape(), query_type)
        })
}

pub(super) fn slots_are_unique<'a>(
    shape: &'a ApplicationQueryResultShape,
    slots: &mut BTreeSet<&'a str>,
) -> bool {
    shape
        .fields()
        .iter()
        .all(|field| !field.slot_type().is_empty() && slots.insert(field.slot_type()))
        && shape.relations().iter().all(|relation| {
            !relation.slot_type().is_empty()
                && slots.insert(relation.slot_type())
                && slots_are_unique(relation.nested_shape(), slots)
        })
}

pub(super) fn counts(shape: &ApplicationQueryResultShape) -> (usize, usize, usize) {
    let mut maximum_depth = 0;
    let mut relation_count = 0;
    let mut field_count = shape.fields().len();
    for relation in shape.relations() {
        let nested = counts(relation.nested_shape());
        maximum_depth = maximum_depth.max(nested.0 + 1);
        relation_count += nested.1 + 1;
        field_count += nested.2;
    }
    (maximum_depth, relation_count, field_count)
}

pub(super) fn contains_entity(shape: &ApplicationQueryResultShape, entity: &str) -> bool {
    shape.root_entity() == entity
        || shape
            .relations()
            .iter()
            .any(|relation| contains_entity(relation.nested_shape(), entity))
}

pub(super) fn field_by_slot<'a>(
    shape: &'a ApplicationQueryResultShape,
    slot_type: &str,
) -> Option<&'a ApplicationQueryResultField> {
    shape
        .fields()
        .iter()
        .find(|field| field.slot_type() == slot_type)
        .or_else(|| {
            shape
                .relations()
                .iter()
                .find_map(|relation| field_by_slot(relation.nested_shape(), slot_type))
        })
}

pub(super) fn relation_by_slot<'a>(
    shape: &'a ApplicationQueryResultShape,
    slot_type: &str,
) -> Option<&'a ApplicationQueryResultRelation> {
    shape
        .relations()
        .iter()
        .find(|relation| relation.slot_type() == slot_type)
        .or_else(|| {
            shape
                .relations()
                .iter()
                .find_map(|relation| relation_by_slot(relation.nested_shape(), slot_type))
        })
}

pub(super) fn many_relation_count(shape: &ApplicationQueryResultShape) -> usize {
    shape
        .relations()
        .iter()
        .map(|relation| {
            usize::from(relation.cardinality() == ApplicationQueryCardinality::Many)
                + many_relation_count(relation.nested_shape())
        })
        .sum()
}

pub(super) fn relation_parent_entity(relation: &ApplicationQueryResultRelation) -> &str {
    match relation.direction() {
        ApplicationQueryResultTraversalDirection::Forward => relation.from(),
        ApplicationQueryResultTraversalDirection::Reverse => relation.to(),
    }
}
