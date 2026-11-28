//! Canonical identity material for related-record absence/existence predicates.
//! Definitions with no related predicates emit zero entries, preserving exact
//! digests for all prior query definitions.
use super::entry::text;
use super::where_filter::append_where_filter_at;
use crate::application_query::{
    ApplicationQueryPredicateExpected, ApplicationQueryRelatedPredicateMeaning, RelationDirection,
};
use worth_foundational::facade::{prepare_aspect_value_identity_basis, CanonicalBasisEntry};

pub(super) fn append_related_predicates(
    entries: &mut Vec<CanonicalBasisEntry>,
    predicates: &[ApplicationQueryRelatedPredicateMeaning],
) {
    for (index, predicate) in predicates.iter().enumerate() {
        let path = format!("related-predicate[{index}]");
        entries.extend([
            text(format!("{path}.relation"), predicate.relation()),
            text(format!("{path}.from"), predicate.from()),
            text(format!("{path}.to"), predicate.to()),
            text(
                format!("{path}.direction"),
                match predicate.direction() {
                    RelationDirection::Forward => "forward",
                    RelationDirection::Reverse => "reverse",
                },
            ),
            text(
                format!("{path}.polarity"),
                predicate.polarity().canonical_name(),
            ),
        ]);
        for (target_index, target) in predicate.target_predicates().iter().enumerate() {
            let tpath = format!("{path}.target-predicate[{target_index}]");
            let (entity, aspect, field) = target.field();
            entries.extend([
                text(format!("{tpath}.entity"), entity),
                text(format!("{tpath}.aspect"), aspect),
                text(format!("{tpath}.field"), field),
                match target.expected() {
                    ApplicationQueryPredicateExpected::Parameter(parameter) => {
                        text(format!("{tpath}.parameter"), parameter)
                    }
                    ApplicationQueryPredicateExpected::Value(expected) => text(
                        format!("{tpath}.expected"),
                        prepare_aspect_value_identity_basis(expected).as_str(),
                    ),
                },
                text(
                    format!("{tpath}.scalar"),
                    target.scalar_family().canonical_name(),
                ),
                text(
                    format!("{tpath}.presence"),
                    (target.presence()).canonical_name(),
                ),
            ]);
        }
        let filter_prefix = format!("{path}.target-where-filter");
        append_where_filter_at(entries, &filter_prefix, predicate.target_where_filter());
    }
}
