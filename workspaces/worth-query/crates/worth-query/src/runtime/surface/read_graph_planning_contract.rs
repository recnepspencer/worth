use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryCardinality, ApplicationQueryOrderingDirection},
    authoring::QueryFamily,
};
use worth_query_installation::facade::{
    prepare_canonical_read_graph_planning_basis, WorthQueryApplicationCanonicalArtifact,
    WorthQueryPreparedReadGraphPlanningContract, WorthQueryReadGraphOrderingMechanism,
    WorthQueryReadGraphOrderingView, WorthQueryReadGraphPlanningContract,
    WorthQueryReadGraphPredicateView, WorthQueryReadGraphProjectionView,
    WorthQueryReadGraphRelationDirection, WorthQueryReadGraphRelationView,
};

use super::{WorthQueryReadBuiltInOperator, WorthQueryReadGraph};
use crate::{
    canonicalization::CanonicalQueryBundle, identity::SchemaBasisDigest,
    validation::ValidatedQueryBundle,
};

pub(super) struct WorthQueryReadGraphPlanningPreparation<'a> {
    pub(super) schema_basis: &'a SchemaBasisDigest,
    pub(super) built_in_operators: &'a [WorthQueryReadBuiltInOperator],
    pub(super) declared_traversal_depth_limit: usize,
    pub(super) canonical: &'a CanonicalQueryBundle,
    pub(super) validated: &'a ValidatedQueryBundle,
}

pub(super) fn prepare_mature_read_graph_planning_basis(
    graph: &WorthQueryReadGraphPlanningPreparation<'_>,
) -> WorthQueryApplicationCanonicalArtifact {
    prepare_canonical_read_graph_planning_basis(
        graph,
        worth_foundational::facade::CanonicalDigestWorkBudget::new(4_096, 1024 * 1024)
            .expect("the mature read-graph canonical budget is nonzero"),
    )
    .expect("installed mature read graphs fit the declared canonical budget")
}

impl WorthQueryReadGraphPlanningContract for WorthQueryReadGraphPlanningPreparation<'_> {
    fn schema_basis_digest(&self) -> &CanonicalDigestId {
        self.schema_basis.digest()
    }

    fn root_entity(&self) -> &str {
        self.canonical.query().root().as_str()
    }

    fn cardinality(&self) -> ApplicationQueryCardinality {
        match self.validated.query().family() {
            QueryFamily::Detail => ApplicationQueryCardinality::OptionalOne,
            QueryFamily::Collection => ApplicationQueryCardinality::Many,
        }
    }

    fn projection_count(&self) -> usize {
        self.validated.query().projection().len()
    }

    fn projection(&self, index: usize) -> Option<WorthQueryReadGraphProjectionView<'_>> {
        planning_projection(self.validated, index)
    }

    fn relation_count(&self) -> usize {
        self.validated.query().traversal().len()
    }

    fn relation(&self, index: usize) -> Option<WorthQueryReadGraphRelationView<'_>> {
        planning_relation(self.validated, self.built_in_operators, index)
    }

    fn predicate_count(&self) -> usize {
        self.validated.query().predicates().entries().len()
    }

    fn predicate(&self, index: usize) -> Option<WorthQueryReadGraphPredicateView<'_>> {
        planning_predicate(self.validated, index)
    }

    fn ordering_count(&self) -> usize {
        self.validated.query().ordering().entries().len()
    }

    fn ordering(&self, index: usize) -> Option<WorthQueryReadGraphOrderingView<'_>> {
        planning_ordering(self.validated, index)
    }

    fn maximum_traversal_depth(&self) -> usize {
        self.declared_traversal_depth_limit
    }
}

impl WorthQueryReadGraphPlanningContract for WorthQueryReadGraph {
    fn schema_basis_digest(&self) -> &CanonicalDigestId {
        self.schema_basis().digest()
    }

    fn root_entity(&self) -> &str {
        self.canonical().query().root().as_str()
    }

    fn cardinality(&self) -> ApplicationQueryCardinality {
        match self.validated().query().family() {
            QueryFamily::Detail => ApplicationQueryCardinality::OptionalOne,
            QueryFamily::Collection => ApplicationQueryCardinality::Many,
        }
    }

    fn projection_count(&self) -> usize {
        self.validated().query().projection().len()
    }

    fn projection(&self, index: usize) -> Option<WorthQueryReadGraphProjectionView<'_>> {
        planning_projection(self.validated(), index)
    }

    fn relation_count(&self) -> usize {
        self.validated().query().traversal().len()
    }

    fn relation(&self, index: usize) -> Option<WorthQueryReadGraphRelationView<'_>> {
        planning_relation(self.validated(), self.built_in_operators(), index)
    }

    fn predicate_count(&self) -> usize {
        self.validated().query().predicates().entries().len()
    }

    fn predicate(&self, index: usize) -> Option<WorthQueryReadGraphPredicateView<'_>> {
        planning_predicate(self.validated(), index)
    }

    fn ordering_count(&self) -> usize {
        self.validated().query().ordering().entries().len()
    }

    fn ordering(&self, index: usize) -> Option<WorthQueryReadGraphOrderingView<'_>> {
        planning_ordering(self.validated(), index)
    }

    fn maximum_traversal_depth(&self) -> usize {
        self.declared_traversal_depth_limit()
    }
}

impl WorthQueryPreparedReadGraphPlanningContract for WorthQueryReadGraph {
    fn canonical_planning_basis(&self) -> &WorthQueryApplicationCanonicalArtifact {
        self.canonical_planning_basis()
    }
}

fn planning_projection(
    validated: &ValidatedQueryBundle,
    index: usize,
) -> Option<WorthQueryReadGraphProjectionView<'_>> {
    let projection = validated.query().projection().get(index)?;
    let output_name = validated
        .result_shape()
        .bindings()
        .iter()
        .find(|binding| {
            binding.native_source_aspect_key() == projection.native_aspect_key()
                && binding.native_source_field_key() == projection.native_field_key()
        })
        .map_or(projection.native_field_key().as_str(), |binding| {
            binding.delivered_name()
        });
    Some(WorthQueryReadGraphProjectionView {
        aspect: projection.native_aspect_key(),
        field: projection.native_field_key(),
        output_name,
    })
}

fn planning_relation<'a>(
    validated: &'a ValidatedQueryBundle,
    operators: &[WorthQueryReadBuiltInOperator],
    index: usize,
) -> Option<WorthQueryReadGraphRelationView<'a>> {
    let traversal = validated.query().traversal().get(index)?;
    Some(WorthQueryReadGraphRelationView {
        relation: traversal.relation_name().as_str(),
        complete_closure: false,
        direction: relation_direction(operators),
        cardinality: ApplicationQueryCardinality::Many,
        depth: usize::from(traversal.depth()),
    })
}

fn planning_predicate(
    validated: &ValidatedQueryBundle,
    index: usize,
) -> Option<WorthQueryReadGraphPredicateView<'_>> {
    let predicate = validated.query().predicates().entries().get(index)?;
    Some(WorthQueryReadGraphPredicateView {
        aspect: predicate.native_aspect_key(),
        field: predicate.native_field_key(),
        parameter: predicate.value_basis(),
        scalar_family: *predicate.field_kind(),
    })
}

fn planning_ordering(
    validated: &ValidatedQueryBundle,
    index: usize,
) -> Option<WorthQueryReadGraphOrderingView<'_>> {
    let ordering = validated.query().ordering().entries().get(index)?;
    let direction = match ordering.direction() {
        "ascending" => ApplicationQueryOrderingDirection::Ascending,
        "descending" => ApplicationQueryOrderingDirection::Descending,
        _ => return None,
    };
    Some(WorthQueryReadGraphOrderingView {
        collection_path: "root",
        aspect: ordering.native_aspect_key(),
        field: ordering.native_field_key(),
        direction,
        scalar_family: *ordering.field_kind(),
        mechanism: WorthQueryReadGraphOrderingMechanism::ProviderOrdered,
    })
}

fn relation_direction(
    operators: &[WorthQueryReadBuiltInOperator],
) -> WorthQueryReadGraphRelationDirection {
    if operators.iter().any(|operator| {
        matches!(
            operator,
            WorthQueryReadBuiltInOperator::BoundedAncestor
                | WorthQueryReadBuiltInOperator::SharedEndpoint
                | WorthQueryReadBuiltInOperator::SharedAttachment
        )
    }) {
        WorthQueryReadGraphRelationDirection::Reverse
    } else {
        WorthQueryReadGraphRelationDirection::Forward
    }
}
