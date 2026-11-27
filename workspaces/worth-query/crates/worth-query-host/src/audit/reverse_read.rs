//! The one rule deciding whether an installed query is a governed
//! referenced-by read: it has at least one root path, and every root path
//! starts by stepping backward from the query's scope entity along an
//! incoming relation. `referenced_by` admits exactly these queries and
//! `entity_impact` advertises exactly these as reverse reads.
use worth_query_declaration::facade::application_schema::RelationDirection;

use super::WorthQueryRelationEdge;

/// The first step of one root path, in either declared or installed form.
pub(super) struct FirstStep<'query> {
    pub(super) direction: RelationDirection,
    pub(super) relation: &'query str,
    pub(super) from: &'query str,
    pub(super) to: &'query str,
}

/// The distinct incoming relations, sorted, that a query scoped on
/// `scope_entity` reads; `None` when it is not a referenced-by read.
/// `first_steps` yields one entry per root path (`None` for an empty path).
pub(super) fn incoming_relations_read<'query>(
    scope_entity: &str,
    first_steps: impl IntoIterator<Item = Option<FirstStep<'query>>>,
) -> Option<Vec<WorthQueryRelationEdge>> {
    let mut relations = Vec::new();
    let mut paths = 0usize;
    for step in first_steps {
        paths += 1;
        let step = step?;
        if step.direction != RelationDirection::Reverse || step.to != scope_entity {
            return None;
        }
        let edge = WorthQueryRelationEdge::new(step.relation, step.from, step.to);
        if !relations.contains(&edge) {
            relations.push(edge);
        }
    }
    if paths == 0 {
        return None;
    }
    relations.sort();
    Some(relations)
}
