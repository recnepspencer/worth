use super::entry::text;
use crate::application_query::{
    ApplicationQueryCardinality, ApplicationQueryResultShape,
    ApplicationQueryResultTraversalDirection,
};
use worth_foundational::facade::CanonicalBasisEntry;

pub(super) fn append_result_shape(
    entries: &mut Vec<CanonicalBasisEntry>,
    shape: &ApplicationQueryResultShape,
    path: &str,
) {
    entries.extend([
        text(format!("{path}.query-type"), shape.query_type()),
        text(format!("{path}.root"), shape.root_entity()),
        text(format!("{path}.result-type"), shape.result_type()),
    ]);
    for (index, field) in shape.fields().iter().enumerate() {
        let field_path = format!("{path}.field[{index}]");
        entries.extend([
            text(format!("{field_path}.query-type"), field.query_type()),
            text(format!("{field_path}.slot-type"), field.slot_type()),
            text(format!("{field_path}.entity"), field.entity()),
            text(format!("{field_path}.aspect"), field.aspect()),
            text(format!("{field_path}.field"), field.field()),
            text(format!("{field_path}.output"), field.output_name()),
            text(
                format!("{field_path}.scalar"),
                field.scalar_family().canonical_name(),
            ),
            text(format!("{field_path}.value-type"), field.value_type()),
            text(
                format!("{field_path}.presence"),
                field_presence_name(field.presence()),
            ),
        ]);
    }
    for (index, relation) in shape.relations().iter().enumerate() {
        let relation_path = format!("{path}.relation[{index}]");
        entries.extend([
            text(format!("{relation_path}.query-type"), relation.query_type()),
            text(format!("{relation_path}.slot-type"), relation.slot_type()),
            text(format!("{relation_path}.name"), relation.relation()),
            text(format!("{relation_path}.from"), relation.from()),
            text(format!("{relation_path}.to"), relation.to()),
            text(
                format!("{relation_path}.direction"),
                traversal_direction_name(relation.direction()),
            ),
            text(format!("{relation_path}.output"), relation.output_name()),
            text(
                format!("{relation_path}.cardinality"),
                cardinality_name(relation.cardinality()),
            ),
        ]);
        if let Some(predicate) = relation.predicate() {
            let (entity, aspect, field) = predicate.field();
            entries.extend([
                text(format!("{relation_path}.predicate.entity"), entity),
                text(format!("{relation_path}.predicate.aspect"), aspect),
                text(format!("{relation_path}.predicate.field"), field),
                text(
                    format!("{relation_path}.predicate.parameter"),
                    predicate.parameter(),
                ),
                text(
                    format!("{relation_path}.predicate.scalar"),
                    predicate.scalar_family().canonical_name(),
                ),
            ]);
        }
        append_result_shape(
            entries,
            relation.nested_shape(),
            &format!("{relation_path}.shape"),
        );
    }
}

const fn field_presence_name(
    value: crate::application_schema::ApplicationFieldPresence,
) -> &'static str {
    match value {
        crate::application_schema::ApplicationFieldPresence::Required => "required",
        crate::application_schema::ApplicationFieldPresence::Optional => "optional",
    }
}

pub(super) const fn cardinality_name(value: ApplicationQueryCardinality) -> &'static str {
    match value {
        ApplicationQueryCardinality::OptionalOne => "optional-one",
        ApplicationQueryCardinality::ExactlyOne => "exactly-one",
        ApplicationQueryCardinality::Many => "many",
    }
}

pub(super) const fn traversal_direction_name(
    value: ApplicationQueryResultTraversalDirection,
) -> &'static str {
    match value {
        ApplicationQueryResultTraversalDirection::Forward => "forward",
        ApplicationQueryResultTraversalDirection::Reverse => "reverse",
    }
}
