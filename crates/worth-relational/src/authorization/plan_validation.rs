use crate::identity::data::KindId;

use super::{
    RelationalAuthorizationFieldOperand, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathPlan, RelationalAuthorizationPlanDenial,
    RelationalAuthorizationTraversal, RelationalAuthorizationTraversalDirection,
};

pub(super) fn validate_plan(
    plan: &RelationalAuthorizationObservationPlan,
) -> Result<(), RelationalAuthorizationPlanDenial> {
    if plan.paths().is_empty() {
        return Err(RelationalAuthorizationPlanDenial::NoPaths);
    }
    for (path_index, path) in plan.paths().iter().enumerate() {
        validate_path(plan, path_index, path)?;
    }
    Ok(())
}

fn validate_path(
    plan: &RelationalAuthorizationObservationPlan,
    path_index: usize,
    path: &RelationalAuthorizationPathPlan,
) -> Result<(), RelationalAuthorizationPlanDenial> {
    let kinds = path_kinds(plan, path_index, path)?;
    validate_predicates(path_index, path, &kinds)?;
    validate_field_constraints(path_index, path, &kinds)?;
    validate_entity_anchors(path_index, path, &kinds)?;
    validate_related_entities(path_index, path, &kinds)?;
    validate_exact_adjacencies(path_index, path, &kinds)
}

fn path_kinds(
    plan: &RelationalAuthorizationObservationPlan,
    path_index: usize,
    path: &RelationalAuthorizationPathPlan,
) -> Result<Vec<KindId>, RelationalAuthorizationPlanDenial> {
    let mut kinds = Vec::with_capacity(path.traversals().len() + 1);
    kinds.push(plan.principal_kind());
    for (traversal_index, traversal) in path.traversals().iter().enumerate() {
        let expected = kinds[traversal_index];
        let (actual, next) = traversal_kinds(traversal);
        if actual != expected {
            return Err(RelationalAuthorizationPlanDenial::DiscontinuousTraversal {
                path: path_index,
                traversal: traversal_index,
                expected,
                actual,
            });
        }
        kinds.push(next);
    }
    let final_kind = *kinds.last().expect(/* Proof: Witness and plan construction seed the principal before appending path entries; no operation removes that initial entry. */ "principal kind is always present");
    if final_kind != plan.scope_kind() {
        return Err(RelationalAuthorizationPlanDenial::PathEndsAtWrongKind {
            path: path_index,
            expected: plan.scope_kind(),
            actual: final_kind,
        });
    }
    Ok(kinds)
}

fn validate_predicates(
    path_index: usize,
    path: &RelationalAuthorizationPathPlan,
    kinds: &[KindId],
) -> Result<(), RelationalAuthorizationPlanDenial> {
    for predicate in path.predicates() {
        if predicate.field().field_path().fields().len() != 1 {
            return Err(
                RelationalAuthorizationPlanDenial::PredicateFieldPathNotSingle {
                    path: path_index,
                    ordinal: predicate.traversal_ordinal(),
                    fields: predicate.field().field_path().fields().len(),
                },
            );
        }
        let Some(expected) = kinds.get(predicate.traversal_ordinal()).copied() else {
            return Err(RelationalAuthorizationPlanDenial::PredicateOutsidePath {
                path: path_index,
                ordinal: predicate.traversal_ordinal(),
                traversals: path.traversals().len(),
            });
        };
        if predicate.entity_kind() != expected {
            return Err(
                RelationalAuthorizationPlanDenial::PredicateTargetsWrongKind {
                    path: path_index,
                    ordinal: predicate.traversal_ordinal(),
                    expected,
                    actual: predicate.entity_kind(),
                },
            );
        }
    }
    Ok(())
}

fn validate_field_constraints(
    path_index: usize,
    path: &RelationalAuthorizationPathPlan,
    kinds: &[KindId],
) -> Result<(), RelationalAuthorizationPlanDenial> {
    for (constraint_index, constraint) in path.field_constraints().iter().enumerate() {
        validate_field_operand(path_index, constraint_index, path, kinds, constraint.left())?;
        validate_field_operand(
            path_index,
            constraint_index,
            path,
            kinds,
            constraint.right(),
        )?;
    }
    Ok(())
}

fn validate_field_operand(
    path_index: usize,
    constraint: usize,
    path: &RelationalAuthorizationPathPlan,
    kinds: &[KindId],
    operand: &RelationalAuthorizationFieldOperand,
) -> Result<(), RelationalAuthorizationPlanDenial> {
    if operand.field().field_path().fields().len() != 1 {
        return Err(
            RelationalAuthorizationPlanDenial::FieldConstraintPathNotSingle {
                path: path_index,
                constraint,
                ordinal: operand.traversal_ordinal(),
                fields: operand.field().field_path().fields().len(),
            },
        );
    }
    let Some(expected) = kinds.get(operand.traversal_ordinal()).copied() else {
        return Err(
            RelationalAuthorizationPlanDenial::FieldConstraintOutsidePath {
                path: path_index,
                constraint,
                ordinal: operand.traversal_ordinal(),
                traversals: path.traversals().len(),
            },
        );
    };
    if operand.entity_kind() != expected {
        return Err(
            RelationalAuthorizationPlanDenial::FieldConstraintTargetsWrongKind {
                path: path_index,
                constraint,
                ordinal: operand.traversal_ordinal(),
                expected,
                actual: operand.entity_kind(),
            },
        );
    }
    Ok(())
}

fn validate_entity_anchors(
    path_index: usize,
    path: &RelationalAuthorizationPathPlan,
    kinds: &[KindId],
) -> Result<(), RelationalAuthorizationPlanDenial> {
    for anchor in path.entity_anchors() {
        let Some(expected) = kinds.get(anchor.traversal_ordinal()).copied() else {
            return Err(RelationalAuthorizationPlanDenial::EntityAnchorOutsidePath {
                path: path_index,
                ordinal: anchor.traversal_ordinal(),
                traversals: path.traversals().len(),
            });
        };
        if anchor.entity_kind() != expected {
            return Err(
                RelationalAuthorizationPlanDenial::EntityAnchorTargetsWrongKind {
                    path: path_index,
                    ordinal: anchor.traversal_ordinal(),
                    expected,
                    actual: anchor.entity_kind(),
                },
            );
        }
    }
    Ok(())
}

fn validate_related_entities(
    path_index: usize,
    path: &RelationalAuthorizationPathPlan,
    kinds: &[KindId],
) -> Result<(), RelationalAuthorizationPlanDenial> {
    for related in path.related_entities() {
        let Some(expected) = kinds.get(related.traversal_ordinal()).copied() else {
            return Err(
                RelationalAuthorizationPlanDenial::RelatedEntityOutsidePath {
                    path: path_index,
                    ordinal: related.traversal_ordinal(),
                    traversals: path.traversals().len(),
                },
            );
        };
        let actual = traversal_start_kind(related.traversal());
        if actual != expected {
            return Err(
                RelationalAuthorizationPlanDenial::RelatedEntityStartsAtWrongKind {
                    path: path_index,
                    ordinal: related.traversal_ordinal(),
                    expected,
                    actual,
                },
            );
        }
    }
    Ok(())
}

fn validate_exact_adjacencies(
    path_index: usize,
    path: &RelationalAuthorizationPathPlan,
    kinds: &[KindId],
) -> Result<(), RelationalAuthorizationPlanDenial> {
    for exact in path.exact_adjacencies() {
        let Some(expected) = kinds.get(exact.traversal_ordinal()).copied() else {
            return Err(
                RelationalAuthorizationPlanDenial::ExactAdjacencyOutsidePath {
                    path: path_index,
                    ordinal: exact.traversal_ordinal(),
                    traversals: path.traversals().len(),
                },
            );
        };
        let actual = traversal_start_kind(exact.traversal());
        if actual != expected {
            return Err(
                RelationalAuthorizationPlanDenial::ExactAdjacencyStartsAtWrongKind {
                    path: path_index,
                    ordinal: exact.traversal_ordinal(),
                    expected,
                    actual,
                },
            );
        }
    }
    Ok(())
}

const fn traversal_kinds(traversal: &RelationalAuthorizationTraversal) -> (KindId, KindId) {
    match traversal.direction() {
        RelationalAuthorizationTraversalDirection::Forward => {
            (traversal.from_kind(), traversal.to_kind())
        }
        RelationalAuthorizationTraversalDirection::Reverse => {
            (traversal.to_kind(), traversal.from_kind())
        }
    }
}

const fn traversal_start_kind(traversal: &RelationalAuthorizationTraversal) -> KindId {
    traversal_kinds(traversal).0
}
