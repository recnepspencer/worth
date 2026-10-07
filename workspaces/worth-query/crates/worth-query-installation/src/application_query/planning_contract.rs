use worth_foundational::facade::{
    AspectKey, AspectValue, CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget, FieldKey,
    ScalarAspectType,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingDirection,
};

use super::graph_access_contract::WorthQueryInstalledGraphPlanningPreparation;
use super::{
    canonical_basis::prepare_planning_basis, WorthQueryApplicationCanonicalArtifact,
    WorthQueryInstalledGraphReadContract,
};

mod admitted_inventory;
mod views;
use views::{
    all_predicate_count, guard, ordering, predicate, projection, relation, root_guard_count,
    root_relation_count,
};

pub use admitted_inventory::{
    WorthQueryAdmittedReadGraphPlanningInventory, WorthQueryPlanningInventoryStop,
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

impl WorthQueryInstalledGraphReadContract {
    pub fn admitted_planning_inventory<Stop>(
        &self,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        WorthQueryAdmittedReadGraphPlanningInventory<'_>,
        WorthQueryPlanningInventoryStop<Stop>,
    > {
        WorthQueryAdmittedReadGraphPlanningInventory::prepare(self.meaning(), prepare)
    }
}
