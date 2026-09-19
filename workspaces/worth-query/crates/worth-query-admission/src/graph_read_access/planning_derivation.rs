use worth_foundational::facade::{CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryCanonicalGraphReadPlanningInput,
    WorthQueryGraphReadAccessComplexityContract, WorthQueryGraphReadAccessInvalidationBasis,
    WorthQueryGraphReadAccessMemoryEstimateBasis, WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadOrderingFieldAuthority,
    WorthQueryGraphReadOrderingPosture, WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadPredicateFieldAuthority, WorthQueryGraphReadRelationAuthority,
    WorthQueryGraphReadResultPressure, WorthQueryGraphReadTraversalOperator,
};

pub fn derive_canonical_graph_read_access_requirements(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    budget: CanonicalDigestWorkBudget,
    prior_work: WorthQueryCanonicalWorkEvidence,
) -> Result<WorthQueryGraphReadAccessRequirementSet, CanonicalDigestDerivationDenial> {
    let mut rows = traversal_rows(input);
    rows.extend(predicate_rows(input));
    rows.extend(ordering_rows(input));
    if input.shape().root_union_dedup_required() {
        rows.push(bound_row(input, root_union_dedup_row()));
    }
    rows.extend(proof_rows(input));
    rows.push(lifecycle_row(input));
    if input.live_maintenance_required() {
        rows.push(live_maintenance_row(input));
    }
    let identity = input.identity();
    WorthQueryGraphReadAccessRequirementSet::new(
        *identity.read_graph_digest(),
        *identity.access_shape_digest(),
        *identity.selectivity_shape_digest(),
        rows,
        budget,
        prior_work,
    )
}

fn root_union_dedup_row() -> WorthQueryGraphReadAccessRequirementRow {
    WorthQueryGraphReadAccessRequirementRow::new(
        WorthQueryGraphReadAccessRequirementKind::DedupSet,
        WorthQueryGraphReadAccessRebuildBasis::OperationResolutionProof,
        WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta,
        WorthQueryGraphReadAccessComplexityContract::BoundedTraversalWorkset,
        WorthQueryGraphReadAccessMemoryEstimateBasis::FrontierDepthBound,
    )
}

fn traversal_rows(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
) -> Vec<WorthQueryGraphReadAccessRequirementRow> {
    let mut rows = Vec::new();
    for relation in input.shape().relations() {
        for operator in relation.operators() {
            rows.extend(operator_rows(input, relation, operator));
        }
    }
    rows.push(bound_row(
        input,
        result_buffer_row(input.shape().result_pressure().clone()),
    ));
    rows
}

fn operator_rows(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    relation: &super::WorthQueryGraphReadPlanningRelation,
    operator: &WorthQueryGraphReadTraversalOperator,
) -> Vec<WorthQueryGraphReadAccessRequirementRow> {
    let mut rows = adjacency_rows(input, relation, operator);
    if requires_traversal_workset(operator) {
        for kind in [
            WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
            WorthQueryGraphReadAccessRequirementKind::VisitedSet,
        ] {
            rows.push(relation_row(
                input,
                relation,
                operator,
                kind,
                RelationRequirementBasis::TraversalWorkset,
            ));
        }
    }
    if requires_dedup_set(operator) {
        rows.push(relation_row(
            input,
            relation,
            operator,
            WorthQueryGraphReadAccessRequirementKind::DedupSet,
            RelationRequirementBasis::TraversalWorkset,
        ));
    }
    rows
}

fn adjacency_rows(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    relation: &super::WorthQueryGraphReadPlanningRelation,
    operator: &WorthQueryGraphReadTraversalOperator,
) -> Vec<WorthQueryGraphReadAccessRequirementRow> {
    let forward = || {
        relation_row(
            input,
            relation,
            operator,
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
            RelationRequirementBasis::DirectionalAdjacency,
        )
    };
    let reverse = || {
        relation_row(
            input,
            relation,
            operator,
            WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency,
            RelationRequirementBasis::ReverseAdjacency,
        )
    };
    match operator {
        WorthQueryGraphReadTraversalOperator::BoundedAncestor
        | WorthQueryGraphReadTraversalOperator::SharedEndpoint
        | WorthQueryGraphReadTraversalOperator::SharedAttachment => vec![reverse()],
        WorthQueryGraphReadTraversalOperator::FrontierSearch => vec![forward(), reverse()],
        WorthQueryGraphReadTraversalOperator::DeclarationTraversal => match relation.direction() {
            WorthQueryAdmittedGraphReadRelationDirection::Ancestor => vec![reverse()],
            WorthQueryAdmittedGraphReadRelationDirection::Forward
            | WorthQueryAdmittedGraphReadRelationDirection::Descendant => vec![forward()],
        },
        WorthQueryGraphReadTraversalOperator::DirectEdge => match relation.direction() {
            WorthQueryAdmittedGraphReadRelationDirection::Ancestor => vec![reverse()],
            WorthQueryAdmittedGraphReadRelationDirection::Forward
            | WorthQueryAdmittedGraphReadRelationDirection::Descendant => vec![forward()],
        },
        WorthQueryGraphReadTraversalOperator::SuccessorWalk
        | WorthQueryGraphReadTraversalOperator::BoundedDescendant
        | WorthQueryGraphReadTraversalOperator::AnchoredFrontier => vec![forward()],
    }
}

fn predicate_rows(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
) -> Vec<WorthQueryGraphReadAccessRequirementRow> {
    if input.shape().predicate_family() == &WorthQueryGraphReadPredicateFamily::None
        && input.shape().predicate_fields().is_empty()
    {
        return Vec::new();
    }
    let authorities = input
        .shape()
        .predicate_fields()
        .iter()
        .map(|field| {
            WorthQueryGraphReadPredicateFieldAuthority::new(
                *input.identity().schema_basis_digest(),
                field.aspect().clone(),
                field.field().clone(),
                field.native_family(),
            )
        })
        .collect();
    vec![bound_row(
        input,
        WorthQueryGraphReadAccessRequirementRow::new(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
            WorthQueryGraphReadAccessRebuildBasis::SelectivityProof,
            WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeFieldDelta,
            WorthQueryGraphReadAccessComplexityContract::CandidatePredicateSupport,
            WorthQueryGraphReadAccessMemoryEstimateBasis::PredicateCandidateSet,
        )
        .with_predicate_family(input.shape().predicate_family().clone())
        .with_predicate_field_authorities(authorities),
    )]
}

fn ordering_rows(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
) -> Vec<WorthQueryGraphReadAccessRequirementRow> {
    if input.shape().ordering_posture() == &WorthQueryGraphReadOrderingPosture::Unordered {
        return Vec::new();
    }
    let authorities = input
        .shape()
        .ordering_fields()
        .iter()
        .map(|field| {
            WorthQueryGraphReadOrderingFieldAuthority::new(
                *input.identity().schema_basis_digest(),
                field.collection_path(),
                field.aspect().clone(),
                field.field().clone(),
                field.direction(),
                field.native_family(),
            )
        })
        .collect();
    vec![bound_row(
        input,
        WorthQueryGraphReadAccessRequirementRow::new(
            WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
            WorthQueryGraphReadAccessRebuildBasis::AuthoritativeFieldTruth,
            WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeFieldDelta,
            WorthQueryGraphReadAccessComplexityContract::CandidateOrderingSupport,
            WorthQueryGraphReadAccessMemoryEstimateBasis::OrderedCandidateSet,
        )
        .with_ordering_posture(input.shape().ordering_posture().clone())
        .with_ordering_field_authorities(authorities),
    )]
}

fn proof_rows(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
) -> Vec<WorthQueryGraphReadAccessRequirementRow> {
    if !input.shape().relationship_proof_required() {
        return Vec::new();
    }
    vec![bound_row(
        input,
        WorthQueryGraphReadAccessRequirementRow::new(
            WorthQueryGraphReadAccessRequirementKind::ProofSupport,
            WorthQueryGraphReadAccessRebuildBasis::ReadGraphProof,
            WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta,
            WorthQueryGraphReadAccessComplexityContract::ProofEvidenceSupport,
            WorthQueryGraphReadAccessMemoryEstimateBasis::ProofEvidenceSet,
        ),
    )]
}

fn lifecycle_row(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
) -> WorthQueryGraphReadAccessRequirementRow {
    bound_row(
        input,
        WorthQueryGraphReadAccessRequirementRow::new(
            WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle,
            WorthQueryGraphReadAccessRebuildBasis::RuntimeSupportRequired,
            WorthQueryGraphReadAccessInvalidationBasis::RuntimeLifecycleDelta,
            WorthQueryGraphReadAccessComplexityContract::LifecycleSupportAdmission,
            WorthQueryGraphReadAccessMemoryEstimateBasis::LifecycleManagedSupport,
        )
        .with_lifecycle_class(input.shape().lifecycle_class().clone()),
    )
}

fn live_maintenance_row(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
) -> WorthQueryGraphReadAccessRequirementRow {
    bound_row(
        input,
        WorthQueryGraphReadAccessRequirementRow::new(
            WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport,
            WorthQueryGraphReadAccessRebuildBasis::RuntimeSupportRequired,
            WorthQueryGraphReadAccessInvalidationBasis::RuntimeLifecycleDelta,
            WorthQueryGraphReadAccessComplexityContract::LifecycleSupportAdmission,
            WorthQueryGraphReadAccessMemoryEstimateBasis::LifecycleManagedSupport,
        )
        .with_lifecycle_class(input.shape().lifecycle_class().clone()),
    )
}

fn relation_row(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    relation: &super::WorthQueryGraphReadPlanningRelation,
    operator: &WorthQueryGraphReadTraversalOperator,
    kind: WorthQueryGraphReadAccessRequirementKind,
    basis: RelationRequirementBasis,
) -> WorthQueryGraphReadAccessRequirementRow {
    let (rebuild, invalidation, complexity, memory) = basis.dimensions();
    bound_row(
        input,
        WorthQueryGraphReadAccessRequirementRow::new(
            kind,
            rebuild,
            invalidation,
            complexity,
            memory,
        )
        .with_relation(
            relation.relation_name(),
            WorthQueryGraphReadRelationAuthority::new(
                *input.identity().schema_basis_digest(),
                relation.relation_name(),
            ),
            relation.direction().clone(),
            relation.depth(),
        )
        .with_fanout_posture(input.shape().fanout_posture().clone())
        .with_traversal_operator(operator.clone()),
    )
}

#[derive(Clone, Copy)]
enum RelationRequirementBasis {
    DirectionalAdjacency,
    ReverseAdjacency,
    TraversalWorkset,
}

impl RelationRequirementBasis {
    fn dimensions(
        self,
    ) -> (
        WorthQueryGraphReadAccessRebuildBasis,
        WorthQueryGraphReadAccessInvalidationBasis,
        WorthQueryGraphReadAccessComplexityContract,
        WorthQueryGraphReadAccessMemoryEstimateBasis,
    ) {
        match self {
            Self::DirectionalAdjacency => (
                WorthQueryGraphReadAccessRebuildBasis::AuthoritativeRelationTruth,
                WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeRelationDelta,
                WorthQueryGraphReadAccessComplexityContract::DirectionalRelationLookup,
                WorthQueryGraphReadAccessMemoryEstimateBasis::RelationDegreeBound,
            ),
            Self::ReverseAdjacency => (
                WorthQueryGraphReadAccessRebuildBasis::AuthoritativeRelationTruth,
                WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeRelationDelta,
                WorthQueryGraphReadAccessComplexityContract::ReverseRelationLookup,
                WorthQueryGraphReadAccessMemoryEstimateBasis::RelationDegreeBound,
            ),
            Self::TraversalWorkset => (
                WorthQueryGraphReadAccessRebuildBasis::OperationResolutionProof,
                WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta,
                WorthQueryGraphReadAccessComplexityContract::BoundedTraversalWorkset,
                WorthQueryGraphReadAccessMemoryEstimateBasis::FrontierDepthBound,
            ),
        }
    }
}

fn result_buffer_row(
    pressure: WorthQueryGraphReadResultPressure,
) -> WorthQueryGraphReadAccessRequirementRow {
    WorthQueryGraphReadAccessRequirementRow::new(
        WorthQueryGraphReadAccessRequirementKind::ResultBuffer,
        WorthQueryGraphReadAccessRebuildBasis::OperationResolutionProof,
        WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta,
        WorthQueryGraphReadAccessComplexityContract::ResultPressureBuffer,
        WorthQueryGraphReadAccessMemoryEstimateBasis::ResultPressureBound,
    )
    .with_result_pressure(pressure)
}

fn bound_row(
    input: &WorthQueryCanonicalGraphReadPlanningInput,
    row: WorthQueryGraphReadAccessRequirementRow,
) -> WorthQueryGraphReadAccessRequirementRow {
    input
        .maximum_cardinality()
        .map_or(row.clone(), |maximum| row.with_maximum_cardinality(maximum))
}

fn requires_traversal_workset(operator: &WorthQueryGraphReadTraversalOperator) -> bool {
    !matches!(operator, WorthQueryGraphReadTraversalOperator::DirectEdge)
}

fn requires_dedup_set(operator: &WorthQueryGraphReadTraversalOperator) -> bool {
    matches!(
        operator,
        WorthQueryGraphReadTraversalOperator::AnchoredFrontier
            | WorthQueryGraphReadTraversalOperator::SharedEndpoint
            | WorthQueryGraphReadTraversalOperator::SharedAttachment
            | WorthQueryGraphReadTraversalOperator::FrontierSearch
    )
}
