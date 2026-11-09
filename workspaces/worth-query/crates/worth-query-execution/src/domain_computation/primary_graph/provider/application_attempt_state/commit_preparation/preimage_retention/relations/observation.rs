//! The original completed decision facts authenticate exact historical adjacency.
use super::*;
pub(super) fn scope_matches(
    scope: &WorthQueryOperationGraphReadScope,
    locus: &InstalledPreImageRelationLocus,
) -> bool {
    matches!(scope, WorthQueryOperationGraphReadScope::Relation(scope) if scope.relation() == locus.relation() && scope.from() == locus.from() && scope.to() == locus.to())
}
pub(super) fn observes(
    fact: &Decision,
    locus: &InstalledPreImageRelationLocus,
    kind: worth_relational::facade::identity::KindId,
    id: worth_relational::facade::identity::RelationId,
    ends: (
        worth_relational::facade::identity::EntityId,
        worth_relational::facade::identity::EntityId,
    ),
    absent: bool,
) -> bool {
    let Decision::Application { read_scope, fact } = fact else {
        return false;
    };
    if !scope_matches(read_scope.as_ref(), locus) {
        return false;
    }
    match fact.as_ref() {
        Fact::Relation {
            relation_kind,
            from,
            to,
            matching_relations,
        } => {
            *relation_kind == kind
                && (*from, *to) == ends
                && if absent {
                    matching_relations.is_empty()
                } else {
                    matching_relations.contains(&id)
                }
        }
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            relations,
            ..
        } => {
            if *relation_kind != kind
                || *anchor
                    != if *direction == Direction::Outgoing {
                        ends.0
                    } else {
                        ends.1
                    }
            {
                return false;
            }
            if absent {
                !relations.iter().any(|edge| (edge.from, edge.to) == ends)
            } else {
                relations
                    .iter()
                    .any(|edge| edge.relation_id == id && (edge.from, edge.to) == ends)
            }
        }
        _ => false,
    }
}

pub(super) fn observation_work(decision: &Decision, locus: &InstalledPreImageRelationLocus) -> u64 {
    let Decision::Application { read_scope, fact } = decision else {
        return 1;
    };
    let WorthQueryOperationGraphReadScope::Relation(scope) = read_scope.as_ref() else {
        return 1;
    };
    let mut work = 1u64;
    for length in [
        scope.relation().len(),
        scope.from().len(),
        scope.to().len(),
        locus.relation().len(),
        locus.from().len(),
        locus.to().len(),
    ] {
        work = work.saturating_add(length as u64);
    }
    work.saturating_add(match fact.as_ref() {
        Fact::Relation {
            matching_relations, ..
        } => matching_relations.len() as u64,
        Fact::Adjacency { relations, .. } => relations.len() as u64,
        _ => 0,
    })
}
