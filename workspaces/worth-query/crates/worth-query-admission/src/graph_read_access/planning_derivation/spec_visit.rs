//! One compiled planning-spec walk. The shell performs the typed row visit.
use super::{RelationRequirementBasis, RowSpec};
use crate::graph_read_access::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryCanonicalGraphReadPlanningInput,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadPredicateFamily, WorthQueryGraphReadTraversalOperator,
};

pub(super) enum SpecStep<'a> {
    Visit,
    Row(RowSpec<'a>),
}

/// Walks the planning input once. `false` means the shell's visit refused.
#[inline(never)]
pub(super) fn drive<'a>(
    input: &'a WorthQueryCanonicalGraphReadPlanningInput,
    step: &mut dyn FnMut(SpecStep<'a>) -> bool,
) -> bool {
    if !step(SpecStep::Visit) {
        return false;
    }
    for relation in input.shape().relations() {
        if !step(SpecStep::Visit) {
            return false;
        }
        for operator in relation.operators() {
            if !step(SpecStep::Visit) {
                return false;
            }
            if *operator == WorthQueryGraphReadTraversalOperator::AggregateSelection {
                if !step(SpecStep::Row(RowSpec::Relation {
                    relation,
                    operator,
                    kind: WorthQueryGraphReadAccessRequirementKind::AggregateSelection,
                    basis: RelationRequirementBasis::AggregateSelection,
                })) {
                    return false;
                }
                continue;
            }
            let forward = RowSpec::Relation {
                relation,
                operator,
                kind: WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
                basis: RelationRequirementBasis::DirectionalAdjacency,
            };
            let reverse = RowSpec::Relation {
                relation,
                operator,
                kind: WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency,
                basis: RelationRequirementBasis::ReverseAdjacency,
            };
            match operator {
                WorthQueryGraphReadTraversalOperator::BoundedAncestor
                | WorthQueryGraphReadTraversalOperator::SharedEndpoint
                | WorthQueryGraphReadTraversalOperator::SharedAttachment => {
                    if !step(SpecStep::Row(reverse)) {
                        return false;
                    }
                }
                WorthQueryGraphReadTraversalOperator::FrontierSearch => {
                    if !step(SpecStep::Row(forward)) {
                        return false;
                    }
                    if !step(SpecStep::Row(reverse)) {
                        return false;
                    }
                }
                WorthQueryGraphReadTraversalOperator::AggregateSelection => {}
                WorthQueryGraphReadTraversalOperator::DeclarationTraversal
                | WorthQueryGraphReadTraversalOperator::CompleteClosure
                | WorthQueryGraphReadTraversalOperator::DirectEdge => match relation.direction() {
                    WorthQueryAdmittedGraphReadRelationDirection::Ancestor => {
                        if !step(SpecStep::Row(reverse)) {
                            return false;
                        }
                    }
                    WorthQueryAdmittedGraphReadRelationDirection::Forward
                    | WorthQueryAdmittedGraphReadRelationDirection::Descendant => {
                        if !step(SpecStep::Row(forward)) {
                            return false;
                        }
                    }
                },
                WorthQueryGraphReadTraversalOperator::SuccessorWalk
                | WorthQueryGraphReadTraversalOperator::BoundedDescendant
                | WorthQueryGraphReadTraversalOperator::AnchoredFrontier => {
                    if !step(SpecStep::Row(forward)) {
                        return false;
                    }
                }
            }
            if !matches!(operator, WorthQueryGraphReadTraversalOperator::DirectEdge) {
                for kind in [
                    WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
                    WorthQueryGraphReadAccessRequirementKind::VisitedSet,
                ] {
                    if !step(SpecStep::Row(RowSpec::Relation {
                        relation,
                        operator,
                        kind,
                        basis: RelationRequirementBasis::TraversalWorkset,
                    })) {
                        return false;
                    }
                }
            }
            if matches!(
                operator,
                WorthQueryGraphReadTraversalOperator::AnchoredFrontier
                    | WorthQueryGraphReadTraversalOperator::SharedEndpoint
                    | WorthQueryGraphReadTraversalOperator::SharedAttachment
                    | WorthQueryGraphReadTraversalOperator::FrontierSearch
            ) && !step(SpecStep::Row(RowSpec::Relation {
                relation,
                operator,
                kind: WorthQueryGraphReadAccessRequirementKind::DedupSet,
                basis: RelationRequirementBasis::TraversalWorkset,
            })) {
                return false;
            }
        }
    }
    if input.shape().component_selection_required()
        && !step(SpecStep::Row(RowSpec::ComponentSelection))
    {
        return false;
    }
    if !step(SpecStep::Row(RowSpec::ResultBuffer)) {
        return false;
    }
    if (input.shape().predicate_family() != &WorthQueryGraphReadPredicateFamily::None
        || !input.shape().predicate_fields().is_empty())
        && !step(SpecStep::Row(RowSpec::Predicate))
    {
        return false;
    }
    if input.shape().ordering_posture() != &WorthQueryGraphReadOrderingPosture::Unordered
        && !step(SpecStep::Row(RowSpec::Ordering))
    {
        return false;
    }
    if input.shape().root_union_dedup_required() && !step(SpecStep::Row(RowSpec::RootUnionDedup)) {
        return false;
    }
    if input.shape().relationship_proof_required() && !step(SpecStep::Row(RowSpec::Proof)) {
        return false;
    }
    if !step(SpecStep::Row(RowSpec::Lifecycle)) {
        return false;
    }
    if input.live_maintenance_required() && !step(SpecStep::Row(RowSpec::LiveMaintenance)) {
        return false;
    }
    true
}
