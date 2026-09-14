use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingDirection,
};
use worth_query_installation::facade::{
    prepare_canonical_read_graph_planning_basis, WorthQueryApplicationCanonicalArtifact,
    WorthQueryInstalledGraphReadContract, WorthQueryPreparedReadGraphPlanningContract,
    WorthQueryReadGraphOrderingView, WorthQueryReadGraphPlanningContract,
    WorthQueryReadGraphPredicateView, WorthQueryReadGraphProjectionView,
    WorthQueryReadGraphRelationDirection, WorthQueryReadGraphRelationView,
};

use crate::facade::graph_read_access::{
    WorthQueryGraphReadAccessRequirementRow, WorthQueryGraphReadAccessRequirementSet,
};

use super::*;

#[derive(Clone, Copy)]
enum PlanningVariation {
    ReverseRelation,
    DeeperRelation,
    WithoutPredicate,
    DescendingOrdering,
    ManyRoots,
    ForeignSchemaBasis,
}

struct VariedPlanningGraph<'a> {
    graph: &'a WorthQueryInstalledGraphReadContract,
    variation: PlanningVariation,
    canonical: WorthQueryApplicationCanonicalArtifact,
    foreign_schema: CanonicalDigestId,
}

impl<'a> VariedPlanningGraph<'a> {
    fn new(graph: &'a WorthQueryInstalledGraphReadContract, variation: PlanningVariation) -> Self {
        let mut prepared = Self {
            graph,
            variation,
            canonical: graph.canonical_planning_basis().clone(),
            foreign_schema: CanonicalDigestId::new([0xf0; 32]),
        };
        prepared.canonical =
            prepare_canonical_read_graph_planning_basis(&prepared, test_canonical_budget())
                .expect("the varied planning graph fits its canonical budget");
        prepared
    }
}

impl WorthQueryPreparedReadGraphPlanningContract for VariedPlanningGraph<'_> {
    fn canonical_planning_basis(&self) -> &WorthQueryApplicationCanonicalArtifact {
        &self.canonical
    }
}

impl WorthQueryReadGraphPlanningContract for VariedPlanningGraph<'_> {
    fn schema_basis_digest(&self) -> &CanonicalDigestId {
        if matches!(self.variation, PlanningVariation::ForeignSchemaBasis) {
            &self.foreign_schema
        } else {
            self.graph.schema_basis_digest()
        }
    }

    fn root_entity(&self) -> &str {
        self.graph.root_entity()
    }

    fn cardinality(&self) -> ApplicationQueryCardinality {
        if matches!(self.variation, PlanningVariation::ManyRoots) {
            ApplicationQueryCardinality::Many
        } else {
            self.graph.cardinality()
        }
    }

    fn projection_count(&self) -> usize {
        self.graph.projection_count()
    }

    fn projection(&self, index: usize) -> Option<WorthQueryReadGraphProjectionView<'_>> {
        self.graph.projection(index)
    }

    fn relation_count(&self) -> usize {
        self.graph.relation_count()
    }

    fn relation(&self, index: usize) -> Option<WorthQueryReadGraphRelationView<'_>> {
        let mut relation = self.graph.relation(index)?;
        if index == 0 {
            match self.variation {
                PlanningVariation::ReverseRelation => {
                    relation.direction = WorthQueryReadGraphRelationDirection::Reverse;
                }
                PlanningVariation::DeeperRelation => {
                    relation.depth += 1;
                }
                _ => {}
            }
        }
        Some(relation)
    }

    fn predicate_count(&self) -> usize {
        if matches!(self.variation, PlanningVariation::WithoutPredicate) {
            0
        } else {
            self.graph.predicate_count()
        }
    }

    fn predicate(&self, index: usize) -> Option<WorthQueryReadGraphPredicateView<'_>> {
        if matches!(self.variation, PlanningVariation::WithoutPredicate) {
            None
        } else {
            self.graph.predicate(index)
        }
    }

    fn ordering_count(&self) -> usize {
        self.graph.ordering_count()
    }

    fn ordering(&self, index: usize) -> Option<WorthQueryReadGraphOrderingView<'_>> {
        let mut ordering = WorthQueryReadGraphPlanningContract::ordering(self.graph, index)?;
        if index == 0 && matches!(self.variation, PlanningVariation::DescendingOrdering) {
            ordering.direction = ApplicationQueryOrderingDirection::Descending;
        }
        Some(ordering)
    }

    fn maximum_traversal_depth(&self) -> usize {
        if matches!(self.variation, PlanningVariation::DeeperRelation) {
            self.graph.maximum_traversal_depth() + 1
        } else {
            self.graph.maximum_traversal_depth()
        }
    }
}

#[test]
fn each_planning_dimension_changes_only_its_owned_evidence() {
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let derive = |variation| {
        admitted_requirements(
            &VariedPlanningGraph::new(query.read_graph(), variation),
            WorthQueryApplicationQueryLane::OneShot,
            32,
            parameters.identity(),
        )
    };
    let baseline = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );

    let reverse = derive(PlanningVariation::ReverseRelation);
    assert_ne!(
        baseline.access_shape_digest(),
        reverse.access_shape_digest()
    );
    assert_ne!(
        relation_row(&baseline).relation_direction(),
        relation_row(&reverse).relation_direction()
    );
    assert_ne!(
        relation_row(&baseline).kind(),
        relation_row(&reverse).kind()
    );
    assert_eq!(
        relation_row(&baseline).relation_depth(),
        relation_row(&reverse).relation_depth()
    );

    let deeper = derive(PlanningVariation::DeeperRelation);
    assert_ne!(baseline.access_shape_digest(), deeper.access_shape_digest());
    assert_ne!(
        relation_row(&baseline).relation_depth(),
        relation_row(&deeper).relation_depth()
    );
    assert_eq!(
        relation_row(&baseline).relation_direction(),
        relation_row(&deeper).relation_direction()
    );

    let without_predicate = derive(PlanningVariation::WithoutPredicate);
    assert!(baseline.contains_kind(&WorthQueryGraphReadAccessRequirementKind::PredicateSupport));
    assert!(!without_predicate
        .contains_kind(&WorthQueryGraphReadAccessRequirementKind::PredicateSupport));

    let descending = derive(PlanningVariation::DescendingOrdering);
    assert_ne!(
        ordering_row(&baseline).ordering_field_authorities()[0].direction(),
        ordering_row(&descending).ordering_field_authorities()[0].direction()
    );
    assert_eq!(
        ordering_row(&baseline).ordering_posture(),
        ordering_row(&descending).ordering_posture()
    );

    let many = derive(PlanningVariation::ManyRoots);
    assert_ne!(baseline.read_graph_digest(), many.read_graph_digest());
    assert_ne!(baseline.access_shape_digest(), many.access_shape_digest());
    assert_eq!(baseline.rows(), many.rows());

    let foreign_schema = derive(PlanningVariation::ForeignSchemaBasis);
    assert_ne!(
        baseline.read_graph_digest(),
        foreign_schema.read_graph_digest()
    );
    assert_ne!(
        baseline.access_shape_digest(),
        foreign_schema.access_shape_digest()
    );
    assert_ne!(baseline.rows(), foreign_schema.rows());
}

fn relation_row(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
) -> &WorthQueryGraphReadAccessRequirementRow {
    requirements
        .rows()
        .iter()
        .find(|row| {
            matches!(
                row.kind(),
                WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency
                    | WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency
            )
        })
        .unwrap()
}

fn ordering_row(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
) -> &WorthQueryGraphReadAccessRequirementRow {
    requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::OrderingSupport)
        .unwrap()
}
