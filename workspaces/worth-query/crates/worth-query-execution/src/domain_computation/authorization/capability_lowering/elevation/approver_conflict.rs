//! Exact-approver rebasing of installed conflict meaning.

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityGraphRule, ErasedApplicationCapabilityContract,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationFieldConstraint,
    RelationalAuthorizationFieldOperand, RelationalAuthorizationPathPlan,
    RelationalAuthorizationPredicate, RelationalAuthorizationRelatedEntityConstraint,
};
use worth_runtime_bridge::facade::BridgeAuthorizationRuleEffect;

use super::super::accumulator::WorthQueryCapabilityRuleLoweringAccumulator;
use super::super::{clause_identity, grant_ordinal, lower_guard};
use super::ElevationRelations;
use crate::domain_computation::authorization::capability_binding_lowering::lower_context_anchors;
use crate::domain_computation::authorization::capability_registry::WorthQueryCapabilityPathTemplate;
use crate::domain_computation::authorization::lowering::lower_authorization_path;
use crate::domain_computation::authorization::{
    authorization_denial, WorthQueryOperationAuthorizationDenial,
};
use crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout;

pub(super) fn compile(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    relations: &ElevationRelations,
    lowering: &mut WorthQueryCapabilityRuleLoweringAccumulator,
) -> Result<Vec<Vec<usize>>, WorthQueryOperationAuthorizationDenial> {
    let conflict = contract
        .composition()
        .decision()
        .conflict()
        .graph()
        .ok_or_else(|| missing_conflict(contract))?;
    let requirements = lower_paths(
        contract,
        layout,
        capability,
        relations,
        conflict,
        lowering.path_count(),
    )?;
    Ok(lowering
        .add_rule(BridgeAuthorizationRuleEffect::Prohibited, requirements)
        .into_path_indices())
}

#[allow(clippy::too_many_arguments)]
fn lower_paths(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    relations: &ElevationRelations,
    conflict: &ApplicationCapabilityGraphRule,
    first_path_index: usize,
) -> Result<Vec<Vec<WorthQueryCapabilityPathTemplate>>, WorthQueryOperationAuthorizationDenial> {
    let mut requirements = Vec::with_capacity(conflict.requirements().len());
    let mut next_path_index = first_path_index;
    for requirement in conflict.requirements() {
        let mut paths = Vec::with_capacity(requirement.clauses().len());
        for clause in requirement.clauses() {
            let path_index = next_path_index;
            next_path_index += 1;
            let base = lower_authorization_path(layout, clause.path())?;
            let mut context_anchors = lower_context_anchors(contract, layout, clause)?;
            for anchor in &mut context_anchors {
                anchor.ordinal += 2;
            }
            paths.push(WorthQueryCapabilityPathTemplate {
                plan: prefix_approver(relations, base),
                identity: clause_identity(capability, path_index),
                guard: lower_guard(contract, clause.guard())?,
                grant_ordinal: grant_ordinal(contract, clause.path())?.map(|ordinal| ordinal + 2),
                elevation_ordinals: vec![1],
                elevation_resource_ordinal: None,
                context_anchors,
            });
        }
        requirements.push(paths);
    }
    if requirements.is_empty() {
        return Err(authorization_denial(
            contract.name(),
            "governed elevation conflict meaning is empty",
        ));
    }
    Ok(requirements)
}

fn prefix_approver(
    relations: &ElevationRelations,
    base: RelationalAuthorizationPathPlan,
) -> RelationalAuthorizationPathPlan {
    const PREFIX_LENGTH: usize = 2;
    let traversals = [
        relations.requester.clone(),
        relations.approver_reverse.clone(),
    ]
    .into_iter()
    .chain(base.traversals().iter().cloned());
    let predicates = base
        .predicates()
        .iter()
        .map(|predicate| shift_predicate(predicate, PREFIX_LENGTH));
    let constraints = base
        .field_constraints()
        .iter()
        .map(|constraint| shift_constraint(constraint, PREFIX_LENGTH));
    let anchors = base
        .entity_anchors()
        .iter()
        .map(|anchor| shift_anchor(*anchor, PREFIX_LENGTH));
    let related = base
        .related_entities()
        .iter()
        .map(|relation| shift_related(relation, PREFIX_LENGTH));
    RelationalAuthorizationPathPlan::new(traversals, predicates)
        .with_field_constraints(constraints)
        .with_entity_anchors(anchors)
        .with_related_entities(related)
}

fn shift_predicate(
    predicate: &RelationalAuthorizationPredicate,
    offset: usize,
) -> RelationalAuthorizationPredicate {
    RelationalAuthorizationPredicate::compare(
        predicate.traversal_ordinal() + offset,
        predicate.entity_kind(),
        predicate.field().clone(),
        predicate.comparison(),
        predicate.expected().clone(),
    )
}

fn shift_constraint(
    constraint: &RelationalAuthorizationFieldConstraint,
    offset: usize,
) -> RelationalAuthorizationFieldConstraint {
    RelationalAuthorizationFieldConstraint::new(
        shift_operand(constraint.left(), offset),
        constraint.comparison(),
        shift_operand(constraint.right(), offset),
    )
}

fn shift_operand(
    operand: &RelationalAuthorizationFieldOperand,
    offset: usize,
) -> RelationalAuthorizationFieldOperand {
    RelationalAuthorizationFieldOperand::new(
        operand.traversal_ordinal() + offset,
        operand.entity_kind(),
        operand.field().clone(),
    )
}

fn shift_anchor(
    anchor: RelationalAuthorizationEntityAnchor,
    offset: usize,
) -> RelationalAuthorizationEntityAnchor {
    RelationalAuthorizationEntityAnchor::new(
        anchor.traversal_ordinal() + offset,
        anchor.entity_kind(),
        anchor.entity(),
    )
}

fn shift_related(
    relation: &RelationalAuthorizationRelatedEntityConstraint,
    offset: usize,
) -> RelationalAuthorizationRelatedEntityConstraint {
    RelationalAuthorizationRelatedEntityConstraint::new(
        relation.traversal_ordinal() + offset,
        relation.traversal().clone(),
        relation.entity(),
    )
}

fn missing_conflict(
    contract: &ErasedApplicationCapabilityContract,
) -> WorthQueryOperationAuthorizationDenial {
    authorization_denial(
        contract.name(),
        "governed elevation requires installed approver conflict meaning",
    )
}
