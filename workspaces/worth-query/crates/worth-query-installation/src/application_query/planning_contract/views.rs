use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryResultTraversalDirection,
    ApplicationQueryRootPathDirection,
};

use super::{
    WorthQueryReadGraphGuardView, WorthQueryReadGraphOrderingMechanism,
    WorthQueryReadGraphOrderingView, WorthQueryReadGraphPredicateView,
    WorthQueryReadGraphProjectionView, WorthQueryReadGraphRelationDirection,
    WorthQueryReadGraphRelationView,
};
use crate::application_query::graph_access_contract::WorthQueryInstalledGraphReadMeaning;
use crate::application_query::{
    WorthQueryInstalledGraphOrdering, WorthQueryInstalledGraphPredicate,
    WorthQueryInstalledGraphRelation, WorthQueryInstalledRootPathGuard,
    WorthQueryInstalledRootPathStep,
};

pub(super) fn projection(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphProjectionView<'_>> {
    meaning
        .projections
        .get(index)
        .map(|projection| WorthQueryReadGraphProjectionView {
            aspect: projection.aspect_key(),
            field: projection.field_key(),
            output_name: projection.output_name(),
        })
}

pub(super) fn relation(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphRelationView<'_>> {
    let root_count = root_relation_count(meaning);
    if index < root_count {
        return root_relation(meaning, index);
    }
    meaning
        .relations
        .get(index - root_count)
        .map(result_relation_view)
}

pub(super) fn root_relation_count(meaning: &WorthQueryInstalledGraphReadMeaning) -> usize {
    meaning
        .root_paths
        .iter()
        .map(|path| path.steps().len())
        .sum()
}

pub(super) fn root_relation(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphRelationView<'_>> {
    let mut remaining = index;
    for path in &meaning.root_paths {
        if let Some(step) = path.steps().get(remaining) {
            return Some(root_step_view(step));
        }
        remaining = remaining.saturating_sub(path.steps().len());
    }
    None
}

pub(super) fn root_guard_count(meaning: &WorthQueryInstalledGraphReadMeaning) -> usize {
    meaning
        .root_paths
        .iter()
        .map(|path| path.guards().len())
        .sum()
}

pub(super) fn guard(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphGuardView<'_>> {
    let mut remaining = index;
    for path in &meaning.root_paths {
        if let Some(guard) = path.guards().get(remaining) {
            return Some(guard_view(guard));
        }
        remaining = remaining.saturating_sub(path.guards().len());
    }
    None
}

pub(super) fn predicate(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphPredicateView<'_>> {
    let predicate = meaning.predicates.get(index).or_else(|| {
        meaning
            .relations
            .iter()
            .filter_map(|relation| relation.predicate())
            .nth(index.saturating_sub(meaning.predicates.len()))
    });
    predicate.map(predicate_view)
}

pub(super) fn all_predicate_count(meaning: &WorthQueryInstalledGraphReadMeaning) -> usize {
    meaning.predicates.len()
        + meaning
            .relations
            .iter()
            .filter(|relation| relation.predicate().is_some())
            .count()
}

pub(super) fn ordering(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphOrderingView<'_>> {
    meaning.ordering.get(index).map(ordering_view)
}

pub(super) fn root_step_view(
    step: &WorthQueryInstalledRootPathStep,
) -> WorthQueryReadGraphRelationView<'_> {
    WorthQueryReadGraphRelationView {
        relation: step.relation(),
        direction: match step.direction() {
            ApplicationQueryRootPathDirection::Forward => {
                WorthQueryReadGraphRelationDirection::Forward
            }
            ApplicationQueryRootPathDirection::Reverse => {
                WorthQueryReadGraphRelationDirection::Reverse
            }
        },
        cardinality: ApplicationQueryCardinality::Many,
        depth: step.depth(),
    }
}

pub(super) fn result_relation_view(
    relation: &WorthQueryInstalledGraphRelation,
) -> WorthQueryReadGraphRelationView<'_> {
    WorthQueryReadGraphRelationView {
        relation: relation.relation(),
        direction: match relation.direction() {
            ApplicationQueryResultTraversalDirection::Forward => {
                WorthQueryReadGraphRelationDirection::Forward
            }
            ApplicationQueryResultTraversalDirection::Reverse => {
                WorthQueryReadGraphRelationDirection::Reverse
            }
        },
        cardinality: relation.cardinality(),
        depth: relation.depth(),
    }
}

pub(super) fn guard_view(
    guard: &WorthQueryInstalledRootPathGuard,
) -> WorthQueryReadGraphGuardView<'_> {
    WorthQueryReadGraphGuardView {
        after_step: guard.after_step(),
        entity: guard.entity(),
        aspect: guard.aspect(),
        field: guard.field(),
        scalar_family: guard.scalar_family(),
        value_type: guard.value_type(),
        expected: guard.expected(),
    }
}

pub(super) fn predicate_view(
    predicate: &WorthQueryInstalledGraphPredicate,
) -> WorthQueryReadGraphPredicateView<'_> {
    WorthQueryReadGraphPredicateView {
        aspect: predicate.aspect_key(),
        field: predicate.field_key(),
        parameter: predicate.parameter(),
        scalar_family: predicate.scalar_family(),
    }
}

pub(super) fn ordering_view(
    ordering: &WorthQueryInstalledGraphOrdering,
) -> WorthQueryReadGraphOrderingView<'_> {
    WorthQueryReadGraphOrderingView {
        collection_path: ordering.collection_path(),
        aspect: ordering.aspect_key(),
        field: ordering.field_key(),
        direction: ordering.direction(),
        scalar_family: ordering.scalar_family(),
        mechanism: WorthQueryReadGraphOrderingMechanism::BoundedProjectedCollection,
    }
}
