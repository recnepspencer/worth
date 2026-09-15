use worth_foundational::facade::{
    AspectKey, AspectValue, CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget, FieldKey,
    ScalarAspectType,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingDirection,
    ApplicationQueryResultTraversalDirection,
};

use super::graph_access_contract::{
    WorthQueryInstalledGraphPlanningPreparation, WorthQueryInstalledGraphReadMeaning,
};
use super::{
    canonical_basis::prepare_planning_basis, WorthQueryApplicationCanonicalArtifact,
    WorthQueryInstalledGraphReadContract,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryReadGraphProjectionView<'a> {
    pub aspect: &'a AspectKey,
    pub field: &'a FieldKey,
    pub output_name: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryReadGraphRelationDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryReadGraphRelationView<'a> {
    pub relation: &'a str,
    pub direction: WorthQueryReadGraphRelationDirection,
    pub cardinality: ApplicationQueryCardinality,
    pub depth: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryReadGraphPredicateView<'a> {
    pub aspect: &'a AspectKey,
    pub field: &'a FieldKey,
    pub parameter: &'a str,
    pub scalar_family: ScalarAspectType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryReadGraphGuardView<'a> {
    pub after_step: usize,
    pub entity: &'a str,
    pub aspect: &'a AspectKey,
    pub field: &'a FieldKey,
    pub scalar_family: ScalarAspectType,
    pub value_type: &'a str,
    pub expected: &'a AspectValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryReadGraphOrderingMechanism {
    ProviderOrdered,
    BoundedProjectedCollection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryReadGraphOrderingView<'a> {
    pub collection_path: &'a str,
    pub aspect: &'a AspectKey,
    pub field: &'a FieldKey,
    pub direction: ApplicationQueryOrderingDirection,
    pub scalar_family: ScalarAspectType,
    pub mechanism: WorthQueryReadGraphOrderingMechanism,
}

pub trait WorthQueryReadGraphPlanningContract {
    fn schema_basis_digest(&self) -> &worth_foundational::facade::CanonicalDigestId;
    fn root_entity(&self) -> &str;
    fn cardinality(&self) -> ApplicationQueryCardinality;
    fn projection_count(&self) -> usize;
    fn projection(&self, index: usize) -> Option<WorthQueryReadGraphProjectionView<'_>>;
    fn relation_count(&self) -> usize;
    fn relation(&self, index: usize) -> Option<WorthQueryReadGraphRelationView<'_>>;
    fn root_union_dedup_required(&self) -> bool {
        false
    }
    fn predicate_count(&self) -> usize;
    fn predicate(&self, index: usize) -> Option<WorthQueryReadGraphPredicateView<'_>>;
    fn guard_count(&self) -> usize {
        0
    }
    fn guard(&self, _index: usize) -> Option<WorthQueryReadGraphGuardView<'_>> {
        None
    }
    fn ordering_count(&self) -> usize;
    fn ordering(&self, index: usize) -> Option<WorthQueryReadGraphOrderingView<'_>>;
    fn maximum_traversal_depth(&self) -> usize;
}

pub trait WorthQueryPreparedReadGraphPlanningContract: WorthQueryReadGraphPlanningContract {
    fn canonical_planning_basis(&self) -> &WorthQueryApplicationCanonicalArtifact;
}

pub fn prepare_canonical_read_graph_planning_basis(
    graph: &impl WorthQueryReadGraphPlanningContract,
    budget: CanonicalDigestWorkBudget,
) -> Result<WorthQueryApplicationCanonicalArtifact, CanonicalDigestDerivationDenial> {
    prepare_planning_basis(graph, budget)
}

impl WorthQueryReadGraphPlanningContract for WorthQueryInstalledGraphPlanningPreparation<'_> {
    fn schema_basis_digest(&self) -> &worth_foundational::facade::CanonicalDigestId {
        &self.meaning.schema_basis_digest
    }

    fn root_entity(&self) -> &str {
        &self.meaning.root_entity
    }

    fn cardinality(&self) -> ApplicationQueryCardinality {
        self.meaning.cardinality
    }

    fn projection_count(&self) -> usize {
        self.meaning.projections.len()
    }

    fn projection(&self, index: usize) -> Option<WorthQueryReadGraphProjectionView<'_>> {
        projection(self.meaning, index)
    }

    fn relation_count(&self) -> usize {
        root_relation_count(self.meaning) + self.meaning.relations.len()
    }

    fn relation(&self, index: usize) -> Option<WorthQueryReadGraphRelationView<'_>> {
        relation(self.meaning, index)
    }

    fn root_union_dedup_required(&self) -> bool {
        !self.meaning.root_paths.is_empty()
    }

    fn predicate_count(&self) -> usize {
        all_predicate_count(self.meaning)
    }

    fn predicate(&self, index: usize) -> Option<WorthQueryReadGraphPredicateView<'_>> {
        predicate(self.meaning, index)
    }

    fn guard_count(&self) -> usize {
        root_guard_count(self.meaning)
    }

    fn guard(&self, index: usize) -> Option<WorthQueryReadGraphGuardView<'_>> {
        guard(self.meaning, index)
    }

    fn ordering_count(&self) -> usize {
        self.meaning.ordering.len()
    }

    fn ordering(&self, index: usize) -> Option<WorthQueryReadGraphOrderingView<'_>> {
        ordering(self.meaning, index)
    }

    fn maximum_traversal_depth(&self) -> usize {
        self.meaning.maximum_traversal_depth
    }
}

impl WorthQueryReadGraphPlanningContract for WorthQueryInstalledGraphReadContract {
    fn schema_basis_digest(&self) -> &worth_foundational::facade::CanonicalDigestId {
        self.schema_basis_digest()
    }

    fn root_entity(&self) -> &str {
        self.root_entity()
    }

    fn cardinality(&self) -> ApplicationQueryCardinality {
        self.cardinality()
    }

    fn projection_count(&self) -> usize {
        self.projections().len()
    }

    fn projection(&self, index: usize) -> Option<WorthQueryReadGraphProjectionView<'_>> {
        projection(self.meaning(), index)
    }

    fn relation_count(&self) -> usize {
        root_relation_count(self.meaning()) + self.relations().len()
    }

    fn relation(&self, index: usize) -> Option<WorthQueryReadGraphRelationView<'_>> {
        relation(self.meaning(), index)
    }

    fn root_union_dedup_required(&self) -> bool {
        !self.root_paths().is_empty()
    }

    fn predicate_count(&self) -> usize {
        all_predicate_count(self.meaning())
    }

    fn predicate(&self, index: usize) -> Option<WorthQueryReadGraphPredicateView<'_>> {
        predicate(self.meaning(), index)
    }

    fn guard_count(&self) -> usize {
        root_guard_count(self.meaning())
    }

    fn guard(&self, index: usize) -> Option<WorthQueryReadGraphGuardView<'_>> {
        guard(self.meaning(), index)
    }

    fn ordering_count(&self) -> usize {
        self.ordering().len()
    }

    fn ordering(&self, index: usize) -> Option<WorthQueryReadGraphOrderingView<'_>> {
        ordering(self.meaning(), index)
    }

    fn maximum_traversal_depth(&self) -> usize {
        self.maximum_traversal_depth()
    }
}

impl WorthQueryPreparedReadGraphPlanningContract for WorthQueryInstalledGraphReadContract {
    fn canonical_planning_basis(&self) -> &WorthQueryApplicationCanonicalArtifact {
        self.canonical_planning_basis()
    }
}

fn projection(
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

fn relation(
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
        .map(|relation| WorthQueryReadGraphRelationView {
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
        })
}

fn root_relation_count(meaning: &WorthQueryInstalledGraphReadMeaning) -> usize {
    meaning
        .root_paths
        .iter()
        .map(|path| path.steps().len())
        .sum()
}

fn root_relation(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphRelationView<'_>> {
    let mut remaining = index;
    for path in &meaning.root_paths {
        if let Some(step) = path.steps().get(remaining) {
            return Some(WorthQueryReadGraphRelationView {
                relation: step.relation(),
                direction: match step.direction() {
                    worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection::Forward => WorthQueryReadGraphRelationDirection::Forward,
                    worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection::Reverse => WorthQueryReadGraphRelationDirection::Reverse,
                },
                cardinality: ApplicationQueryCardinality::Many,
                depth: step.depth(),
            });
        }
        remaining = remaining.saturating_sub(path.steps().len());
    }
    None
}

fn root_guard_count(meaning: &WorthQueryInstalledGraphReadMeaning) -> usize {
    meaning
        .root_paths
        .iter()
        .map(|path| path.guards().len())
        .sum()
}

fn guard(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphGuardView<'_>> {
    let mut remaining = index;
    for path in &meaning.root_paths {
        if let Some(guard) = path.guards().get(remaining) {
            return Some(WorthQueryReadGraphGuardView {
                after_step: guard.after_step(),
                entity: guard.entity(),
                aspect: guard.aspect(),
                field: guard.field(),
                scalar_family: guard.scalar_family(),
                value_type: guard.value_type(),
                expected: guard.expected(),
            });
        }
        remaining = remaining.saturating_sub(path.guards().len());
    }
    None
}

fn predicate(
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
    predicate.map(|predicate| WorthQueryReadGraphPredicateView {
        aspect: predicate.aspect_key(),
        field: predicate.field_key(),
        parameter: predicate.parameter(),
        scalar_family: predicate.scalar_family(),
    })
}

fn all_predicate_count(meaning: &WorthQueryInstalledGraphReadMeaning) -> usize {
    meaning.predicates.len()
        + meaning
            .relations
            .iter()
            .filter(|relation| relation.predicate().is_some())
            .count()
}

fn ordering(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    index: usize,
) -> Option<WorthQueryReadGraphOrderingView<'_>> {
    meaning
        .ordering
        .get(index)
        .map(|ordering| WorthQueryReadGraphOrderingView {
            collection_path: ordering.collection_path(),
            aspect: ordering.aspect_key(),
            field: ordering.field_key(),
            direction: ordering.direction(),
            scalar_family: ordering.scalar_family(),
            mechanism: WorthQueryReadGraphOrderingMechanism::BoundedProjectedCollection,
        })
}
