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
use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityBasis;
use crate::graph_read_access::{
    derive_canonical_graph_read_access_requirements, WorthQueryAdmittedGraphReadRelationDirection,
    WorthQueryCanonicalGraphReadPlanningInput, WorthQueryGraphReadAccessRequirementSet,
    WorthQueryGraphReadFanoutPosture, WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadPlanningIdentity, WorthQueryGraphReadPlanningOrderingField,
    WorthQueryGraphReadPlanningPredicateField, WorthQueryGraphReadPlanningRelation,
    WorthQueryGraphReadPlanningShape, WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadResultPressure, WorthQueryGraphReadTraversalOperator,
};

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
    let mechanisms = (0..graph.ordering_count())
        .filter_map(|index| graph.ordering(index).map(|ordering| ordering.mechanism))
        .collect::<Vec<_>>();
    match mechanisms.as_slice() {
        [] => WorthQueryGraphReadOrderingPosture::Unordered,
        values
            if values
                .iter()
                .all(|value| *value == WorthQueryReadGraphOrderingMechanism::ProviderOrdered) =>
        {
            WorthQueryGraphReadOrderingPosture::ProviderOrdered
        }
        values
            if values.iter().all(|value| {
                *value == WorthQueryReadGraphOrderingMechanism::BoundedProjectedCollection
            }) =>
        {
            WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection
        }
        _ => WorthQueryGraphReadOrderingPosture::Mixed,
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

fn access_shape_digest(
    graph: &impl WorthQueryReadGraphPlanningContract,
    planning_graph_identity: CanonicalDigestId,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    budget: CanonicalDigestWorkBudget,
) -> Result<
    (
        CanonicalDigestId,
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ),
    CanonicalDigestDerivationDenial,
> {
    let mut basis = WorthQueryCanonicalIdentityBasis::new(
        "worth-query.application-query-access-shape",
        "worth-query-application-query-access-shape-v3",
        budget,
    );
    basis.digest("graph", planning_graph_identity)?;
    basis.digest("schema-basis", *graph.schema_basis_digest())?;
    basis.text("root", graph.root_entity())?;
    basis.text("cardinality", cardinality_name(graph.cardinality()))?;
    basis.text("lane", lane.as_str())?;
    basis.unsigned("maximum-result-count", maximum_result_count)?;
    basis.unsigned("relation-count", graph.relation_count())?;
    for index in 0..graph.relation_count() {
        let relation = graph
            .relation(index)
            .expect("planning relation count must be exact");
        let path = format!("relation[{index}]");
        basis.text(format!("{path}.name"), relation.relation)?;
        basis.text(
            format!("{path}.direction"),
            relation_direction_name(relation.direction),
        )?;
        basis.unsigned(format!("{path}.depth"), relation.depth)?;
    }
    basis.derive()
}

fn selectivity_shape_digest(
    planning_graph_identity: CanonicalDigestId,
    access_shape_identity: CanonicalDigestId,
    binding_identity: CanonicalDigestId,
    budget: CanonicalDigestWorkBudget,
) -> Result<
    (
        CanonicalDigestId,
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ),
    CanonicalDigestDerivationDenial,
> {
    let mut basis = WorthQueryCanonicalIdentityBasis::new(
        "worth-query.application-query-selectivity-shape",
        "worth-query-application-query-selectivity-shape-v3",
        budget,
    );
    basis.digest("graph", planning_graph_identity)?;
    basis.digest("access-shape", access_shape_identity)?;
    basis.digest("bindings", binding_identity)?;
    basis.derive()
}

const fn cardinality_name(cardinality: ApplicationQueryCardinality) -> &'static str {
    match cardinality {
        ApplicationQueryCardinality::OptionalOne => "optional-one",
        ApplicationQueryCardinality::ExactlyOne => "exactly-one",
        ApplicationQueryCardinality::Many => "many",
    }
}

const fn relation_direction_name(direction: WorthQueryReadGraphRelationDirection) -> &'static str {
    match direction {
        WorthQueryReadGraphRelationDirection::Forward => "forward",
        WorthQueryReadGraphRelationDirection::Reverse => "reverse",
    }
}
