use worth_query_admission::facade::{
    application_query::WorthQueryApplicationQueryLane,
    graph_read_access::WorthQueryGraphReadAccessRequirementSet,
};
use worth_query_admission::integration::derive_graph_read_access_requirements_for_contract;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingDirection,
};
use worth_query_installation::facade::{
    prepare_canonical_read_graph_planning_basis, WorthQueryApplicationCanonicalArtifact,
    WorthQueryPreparedReadGraphPlanningContract, WorthQueryReadGraphOrderingView,
    WorthQueryReadGraphPlanningContract, WorthQueryReadGraphPredicateView,
    WorthQueryReadGraphProjectionView, WorthQueryReadGraphRelationDirection,
    WorthQueryReadGraphRelationView,
};

use super::{requirement_semantics, support};

#[derive(Clone, Copy, Debug)]
enum GraphVariation {
    Baseline,
    ReverseRelation,
    DeeperRelation,
    EqualityPredicate,
    DescendingOrdering,
    OptionalRoot,
    SecondRelation,
    NarrowRelationResult,
}

struct VariedGraph<'a, Graph> {
    graph: &'a Graph,
    variation: GraphVariation,
    canonical: WorthQueryApplicationCanonicalArtifact,
}

struct BaselineRequirements<'a> {
    mature: &'a WorthQueryGraphReadAccessRequirementSet,
    application: &'a WorthQueryGraphReadAccessRequirementSet,
}

impl<'a, Graph> VariedGraph<'a, Graph>
where
    Graph: WorthQueryPreparedReadGraphPlanningContract,
{
    fn new(graph: &'a Graph, variation: GraphVariation) -> Self {
        let mut prepared = Self {
            graph,
            variation,
            canonical: graph.canonical_planning_basis().clone(),
        };
        prepared.canonical =
            prepare_canonical_read_graph_planning_basis(&prepared, canonical_budget())
                .expect("the one-axis graph fits its canonical budget");
        prepared
    }
}

impl<Graph> WorthQueryPreparedReadGraphPlanningContract for VariedGraph<'_, Graph>
where
    Graph: WorthQueryPreparedReadGraphPlanningContract,
{
    fn canonical_planning_basis(&self) -> &WorthQueryApplicationCanonicalArtifact {
        &self.canonical
    }
}

impl<Graph> WorthQueryReadGraphPlanningContract for VariedGraph<'_, Graph>
where
    Graph: WorthQueryReadGraphPlanningContract,
{
    fn schema_basis_digest(&self) -> &worth_foundational::facade::CanonicalDigestId {
        self.graph.schema_basis_digest()
    }

    fn root_entity(&self) -> &str {
        self.graph.root_entity()
    }

    fn cardinality(&self) -> ApplicationQueryCardinality {
        if matches!(self.variation, GraphVariation::OptionalRoot) {
            ApplicationQueryCardinality::OptionalOne
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
            + usize::from(matches!(self.variation, GraphVariation::SecondRelation))
    }

    fn relation(&self, index: usize) -> Option<WorthQueryReadGraphRelationView<'_>> {
        let base_index = if index == self.graph.relation_count()
            && matches!(self.variation, GraphVariation::SecondRelation)
        {
            0
        } else {
            index
        };
        let mut relation = self.graph.relation(base_index)?;
        if base_index == 0 {
            match self.variation {
                GraphVariation::ReverseRelation => {
                    relation.direction = WorthQueryReadGraphRelationDirection::Reverse;
                }
                GraphVariation::DeeperRelation => relation.depth += 1,
                GraphVariation::NarrowRelationResult => {
                    relation.cardinality = ApplicationQueryCardinality::ExactlyOne;
                }
                _ => {}
            }
        }
        Some(relation)
    }

    fn predicate_count(&self) -> usize {
        if matches!(self.variation, GraphVariation::EqualityPredicate) {
            1
        } else {
            self.graph.predicate_count()
        }
    }

    fn predicate(&self, index: usize) -> Option<WorthQueryReadGraphPredicateView<'_>> {
        if matches!(self.variation, GraphVariation::EqualityPredicate) && index == 0 {
            let field = self.graph.ordering(0)?;
            return Some(WorthQueryReadGraphPredicateView {
                aspect: field.aspect,
                field: field.field,
                parameter: "one-axis-equality",
                scalar_family: field.scalar_family,
            });
        }
        self.graph.predicate(index)
    }

    fn ordering_count(&self) -> usize {
        self.graph.ordering_count()
    }

    fn ordering(&self, index: usize) -> Option<WorthQueryReadGraphOrderingView<'_>> {
        let mut ordering = self.graph.ordering(index)?;
        if index == 0 && matches!(self.variation, GraphVariation::DescendingOrdering) {
            ordering.direction = ApplicationQueryOrderingDirection::Descending;
        }
        Some(ordering)
    }

    fn maximum_traversal_depth(&self) -> usize {
        self.graph.maximum_traversal_depth()
            + usize::from(matches!(self.variation, GraphVariation::DeeperRelation))
    }
}

#[test]
fn mature_and_application_sources_share_every_one_axis_planning_delta() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mature = support::mature_family(request_execution);
    let application = support::installed_application_query();
    let mature_graph = mature.read_graph();
    let application_graph = application.read_graph();
    let baseline_mature = derive(mature_graph, GraphVariation::Baseline, 32, false);
    let baseline_application = derive(application_graph, GraphVariation::Baseline, 32, false);
    assert_eq!(
        requirement_semantics(baseline_mature.rows()),
        requirement_semantics(baseline_application.rows())
    );
    let baseline = BaselineRequirements {
        mature: &baseline_mature,
        application: &baseline_application,
    };

    for variation in [
        GraphVariation::ReverseRelation,
        GraphVariation::DeeperRelation,
        GraphVariation::EqualityPredicate,
        GraphVariation::DescendingOrdering,
        GraphVariation::OptionalRoot,
        GraphVariation::SecondRelation,
        GraphVariation::NarrowRelationResult,
    ] {
        baseline.assert_equal_delta(
            derive(mature_graph, variation, 32, false),
            derive(application_graph, variation, 32, false),
            variation,
        );
    }
    baseline.assert_equal_delta(
        derive(mature_graph, GraphVariation::Baseline, 33, false),
        derive(application_graph, GraphVariation::Baseline, 33, false),
        "maximum-result-count",
    );
    baseline.assert_equal_delta(
        derive(mature_graph, GraphVariation::Baseline, 32, true),
        derive(application_graph, GraphVariation::Baseline, 32, true),
        "live-lifecycle",
    );
}

fn derive(
    graph: &impl WorthQueryPreparedReadGraphPlanningContract,
    variation: GraphVariation,
    maximum_result_count: usize,
    live: bool,
) -> WorthQueryGraphReadAccessRequirementSet {
    derive_graph_read_access_requirements_for_contract(
        &VariedGraph::new(graph, variation),
        if live {
            WorthQueryApplicationQueryLane::Live
        } else {
            WorthQueryApplicationQueryLane::OneShot
        },
        maximum_result_count,
        &worth_foundational::facade::CanonicalDigestId::new([0x11; 32]),
        canonical_budget(),
    )
    .expect("the one-axis requirements fit their canonical budget")
}

fn canonical_budget() -> worth_foundational::facade::CanonicalDigestWorkBudget {
    worth_foundational::facade::CanonicalDigestWorkBudget::new(4_096, 1024 * 1024)
        .expect("the one-axis canonical budget is nonzero")
}

impl BaselineRequirements<'_> {
    fn assert_equal_delta(
        &self,
        varied_mature: WorthQueryGraphReadAccessRequirementSet,
        varied_application: WorthQueryGraphReadAccessRequirementSet,
        axis: impl std::fmt::Debug,
    ) {
        assert_ne!(
            varied_mature.digest(),
            self.mature.digest(),
            "mature source ignored {axis:?}"
        );
        assert_ne!(
            varied_application.digest(),
            self.application.digest(),
            "application source ignored {axis:?}"
        );
        assert_eq!(
            requirement_semantics(varied_mature.rows()),
            requirement_semantics(varied_application.rows()),
            "sources diverged for {axis:?}"
        );
    }
}
