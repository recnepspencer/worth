use crate::application_query::{
    ApplicationQueryResultShape, WorthQueryPortableApplicationQueryParts,
};

pub(super) fn is_canonical(definition: &WorthQueryPortableApplicationQueryParts) -> bool {
    strictly_increasing(definition.parameters())
        && strictly_increasing(definition.predicates())
        && strictly_increasing(definition.related_predicates())
        && definition
            .related_predicates()
            .iter()
            .all(|predicate| strictly_increasing(predicate.target_predicates()))
        && strictly_increasing(definition.root_paths())
        && result_shape_is_canonical(definition.result_shape())
        && strictly_increasing(definition.disclosure().rules())
}

fn result_shape_is_canonical(shape: &ApplicationQueryResultShape) -> bool {
    strictly_increasing(shape.fields())
        && strictly_increasing(shape.relations())
        && shape
            .relations()
            .iter()
            .all(|relation| result_shape_is_canonical(relation.nested_shape()))
}

fn strictly_increasing<Value: Ord>(values: &[Value]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
