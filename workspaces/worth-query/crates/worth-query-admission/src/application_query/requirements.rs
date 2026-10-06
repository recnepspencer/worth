use worth_foundational::facade::{
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingDirection,
};
use worth_query_installation::facade::{
    WorthQueryPreparedReadGraphPlanningContract, WorthQueryReadGraphOrderingMechanism,
    WorthQueryReadGraphPlanningContract, WorthQueryReadGraphRelationDirection,
};

use super::WorthQueryApplicationQueryLane;
use crate::graph_read_access::{
    derive_canonical_graph_read_access_requirements, WorthQueryAdmittedGraphReadRelationDirection,
    WorthQueryCanonicalGraphReadPlanningInput, WorthQueryGraphReadAccessRequirementSet,
    WorthQueryGraphReadFanoutPosture, WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadPlanningIdentity, WorthQueryGraphReadPlanningOrderingField,
    WorthQueryGraphReadPlanningPredicateField, WorthQueryGraphReadPlanningRelation,
    WorthQueryGraphReadPlanningShape, WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadResultPressure, WorthQueryGraphReadTraversalOperator,
};

mod admitted;
mod identity;
use identity::{access_shape_digest, selectivity_shape_digest};

pub use admitted::derive_graph_read_access_requirements_for_contract_admitted;

pub fn derive_graph_read_access_requirements_for_contract(
    graph: &impl WorthQueryPreparedReadGraphPlanningContract,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    selectivity_binding_digest: &CanonicalDigestId,
    budget: CanonicalDigestWorkBudget,
) -> Result<WorthQueryGraphReadAccessRequirementSet, CanonicalDigestDerivationDenial> {
    let (input, work) = planning_input(
        graph,
        lane,
        maximum_result_count,
        selectivity_binding_digest,
        budget,
    )?;
    derive_canonical_graph_read_access_requirements(&input, budget, work)
}

fn planning_input(
    graph: &impl WorthQueryPreparedReadGraphPlanningContract,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    selectivity_binding_digest: &CanonicalDigestId,
    budget: CanonicalDigestWorkBudget,
) -> Result<
    (
        WorthQueryCanonicalGraphReadPlanningInput,
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ),
    CanonicalDigestDerivationDenial,
> {
    let planning_graph_identity = *graph.canonical_planning_basis().digest();
    let relations = relations(graph);
    let relationship_proof_required = !relations.is_empty();
    let fanout = fanout_posture(relations.len());
    let (access_shape_digest, access_work) = access_shape_digest(
        graph,
        planning_graph_identity,
        lane,
        maximum_result_count,
        budget,
    )?;
    let (selectivity_shape_digest, selectivity_work) = selectivity_shape_digest(
        planning_graph_identity,
        access_shape_digest,
        *selectivity_binding_digest,
        budget,
    )?;
    let identity = WorthQueryGraphReadPlanningIdentity::from_admitted_evidence(
        planning_graph_identity,
        access_shape_digest,
        selectivity_shape_digest,
        *graph.schema_basis_digest(),
    );
    let shape = WorthQueryGraphReadPlanningShape::from_admitted_shape(
        relations,
        fanout,
        result_pressure(
            graph.cardinality(),
            graph.projection_count(),
            has_many_relation(graph),
        ),
    )
    .with_predicates(predicate_family(graph), predicate_fields(graph))
    .with_ordering(ordering_posture(graph, lane), ordering_fields(graph))
    .with_relationship_proof_required(relationship_proof_required)
    .with_root_union_dedup_required(graph.root_union_dedup_required());
    Ok((
        WorthQueryCanonicalGraphReadPlanningInput::from_admitted_evidence(identity, shape)
            .with_maximum_cardinality(maximum_result_count)
            .with_live_maintenance_required(lane == WorthQueryApplicationQueryLane::Live),
        access_work.combine(selectivity_work),
    ))
}

fn relations(
    graph: &impl WorthQueryReadGraphPlanningContract,
) -> Vec<WorthQueryGraphReadPlanningRelation> {
    (0..graph.relation_count())
        .map(|index| {
            let relation = graph
                .relation(index)
                .expect("planning contract relation count must be exact");
            let direction = match relation.direction {
                WorthQueryReadGraphRelationDirection::Forward => {
                    WorthQueryAdmittedGraphReadRelationDirection::Forward
                }
                WorthQueryReadGraphRelationDirection::Reverse => {
                    WorthQueryAdmittedGraphReadRelationDirection::Ancestor
                }
            };
            WorthQueryGraphReadPlanningRelation::from_admitted_reference(
                relation.relation,
                direction,
                relation.depth,
                vec![WorthQueryGraphReadTraversalOperator::DirectEdge],
            )
        })
        .collect()
}

fn predicate_fields(
    graph: &impl WorthQueryReadGraphPlanningContract,
) -> Vec<WorthQueryGraphReadPlanningPredicateField> {
    let predicates = (0..graph.predicate_count()).map(|index| {
        let predicate = graph
            .predicate(index)
            .expect("planning contract predicate count must be exact");
        WorthQueryGraphReadPlanningPredicateField::from_admitted_field(
            predicate.aspect.clone(),
            predicate.field.clone(),
            predicate.scalar_family.canonical_name(),
        )
    });
    let guards = (0..graph.guard_count()).map(|index| {
        let guard = graph
            .guard(index)
            .expect("planning contract guard count must be exact");
        WorthQueryGraphReadPlanningPredicateField::from_admitted_field(
            guard.aspect.clone(),
            guard.field.clone(),
            guard.scalar_family.canonical_name(),
        )
    });
    predicates.chain(guards).collect()
}

fn ordering_fields(
    graph: &impl WorthQueryReadGraphPlanningContract,
) -> Vec<WorthQueryGraphReadPlanningOrderingField> {
    (0..graph.ordering_count())
        .map(|index| {
            let ordering = graph
                .ordering(index)
                .expect("planning contract ordering count must be exact");
            WorthQueryGraphReadPlanningOrderingField::from_admitted_field(
                ordering.collection_path,
                ordering.aspect.clone(),
                ordering.field.clone(),
                match ordering.direction {
                    ApplicationQueryOrderingDirection::Ascending => "ascending",
                    ApplicationQueryOrderingDirection::Descending => "descending",
                },
                ordering.scalar_family.canonical_name(),
            )
        })
        .collect()
}

fn predicate_family(
    graph: &impl WorthQueryReadGraphPlanningContract,
) -> WorthQueryGraphReadPredicateFamily {
    if graph.predicate_count() == 0 && graph.guard_count() == 0 {
        WorthQueryGraphReadPredicateFamily::None
    } else {
        WorthQueryGraphReadPredicateFamily::Equality
    }
}

fn ordering_posture(
    graph: &impl WorthQueryReadGraphPlanningContract,
    lane: WorthQueryApplicationQueryLane,
) -> WorthQueryGraphReadOrderingPosture {
    if lane == WorthQueryApplicationQueryLane::Continuation && graph.ordering_count() != 0 {
        return WorthQueryGraphReadOrderingPosture::IndexedRelatedCollectionSeek;
    }
    let mut posture = OrderingPostureScan::new();
    for index in 0..graph.ordering_count() {
        if let Some(ordering) = graph.ordering(index) {
            posture.observe(ordering.mechanism);
        }
    }
    posture.finish(lane, graph.ordering_count())
}

pub(super) struct OrderingPostureScan {
    any: bool,
    all_provider_ordered: bool,
    all_bounded_projected: bool,
}

impl OrderingPostureScan {
    pub(super) fn new() -> Self {
        Self {
            any: false,
            all_provider_ordered: true,
            all_bounded_projected: true,
        }
    }

    pub(super) fn observe(&mut self, mechanism: WorthQueryReadGraphOrderingMechanism) {
        self.any = true;
        self.all_provider_ordered &=
            mechanism == WorthQueryReadGraphOrderingMechanism::ProviderOrdered;
        self.all_bounded_projected &=
            mechanism == WorthQueryReadGraphOrderingMechanism::BoundedProjectedCollection;
    }

    pub(super) fn finish(
        self,
        lane: WorthQueryApplicationQueryLane,
        ordering_count: usize,
    ) -> WorthQueryGraphReadOrderingPosture {
        if lane == WorthQueryApplicationQueryLane::Continuation && ordering_count != 0 {
            WorthQueryGraphReadOrderingPosture::IndexedRelatedCollectionSeek
        } else if !self.any {
            WorthQueryGraphReadOrderingPosture::Unordered
        } else if self.all_provider_ordered {
            WorthQueryGraphReadOrderingPosture::ProviderOrdered
        } else if self.all_bounded_projected {
            WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection
        } else {
            WorthQueryGraphReadOrderingPosture::Mixed
        }
    }
}

fn fanout_posture(relation_count: usize) -> WorthQueryGraphReadFanoutPosture {
    match relation_count {
        0 => WorthQueryGraphReadFanoutPosture::None,
        1 => WorthQueryGraphReadFanoutPosture::SingleRelation,
        _ => WorthQueryGraphReadFanoutPosture::MultiRelation,
    }
}

fn result_pressure(
    cardinality: ApplicationQueryCardinality,
    projection_count: usize,
    has_many_relation: bool,
) -> WorthQueryGraphReadResultPressure {
    match cardinality {
        _ if has_many_relation => WorthQueryGraphReadResultPressure::CollectionWide,
        ApplicationQueryCardinality::OptionalOne | ApplicationQueryCardinality::ExactlyOne => {
            WorthQueryGraphReadResultPressure::Detail
        }
        ApplicationQueryCardinality::Many if projection_count <= 3 => {
            WorthQueryGraphReadResultPressure::CollectionNarrow
        }
        ApplicationQueryCardinality::Many => WorthQueryGraphReadResultPressure::CollectionWide,
    }
}

fn has_many_relation(graph: &impl WorthQueryReadGraphPlanningContract) -> bool {
    (0..graph.relation_count()).any(|index| {
        graph
            .relation(index)
            .is_some_and(|relation| relation.cardinality == ApplicationQueryCardinality::Many)
    })
}
