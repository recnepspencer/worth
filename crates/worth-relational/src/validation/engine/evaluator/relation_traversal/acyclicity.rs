use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredAcyclicityContract;
use crate::transactions::data::EntityReference;
use crate::validation::data::{InvariantClass, InvariantViolation, InvariantViolationFields};

use super::super::super::context::InvariantExecutionContext;
use super::planned_successors::planned_successor_map;
use super::relation_successors::PreparedSuccessorTraversal;
use super::traversal_budget::{traversal_budget_exceeded_violation, RelationTraversalBudget};

pub(in crate::validation::engine::evaluator) fn evaluate_acyclicity_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredAcyclicityContract,
) -> Option<InvariantViolation> {
    let scope = match context.required_relation_integrity_scope(contract.relation_kind_id, class) {
        Ok(scope) => scope,
        Err(violation) => return Some(violation),
    };
    if scope.planned_edges.is_empty() {
        return None;
    }

    context.metrics().count_relation_contracts_evaluated(1);
    let planned_successors = planned_successor_map(&scope.planned_edges, context);
    if !context.checkpoint(0) {
        return None;
    }
    let traversal = PreparedSuccessorTraversal {
        scope,
        class,
        contract_id: &contract.contract_id,
        relation_kind_id: contract.relation_kind_id,
        planned_successors: &planned_successors,
    };
    for edge in &scope.planned_edges {
        if !context.checkpoint(1) {
            return None;
        }
        context.metrics().count_relation_slot_scans(1);
        let reaches_cycle = if edge.source == edge.target {
            Ok(true)
        } else {
            relation_kind_reaches(
                &traversal,
                context,
                context.relation_integrity_scope_budget(),
                edge.target.clone(),
                edge.source.clone(),
            )
        };
        match reaches_cycle {
            Ok(true) => {
                return Some(InvariantViolation {
                    class,
                    code: DiagnosticCode::InvariantViolation,
                    detail: format!(
                        "acyclicity contract '{}' detected a cycle for relation kind {:?}",
                        contract.contract_id, contract.relation_kind_id
                    ),
                    fields: InvariantViolationFields::Acyclicity {
                        contract_id: contract.contract_id.clone(),
                        relation_kind_id: contract.relation_kind_id,
                        source: edge.source.clone(),
                        target: edge.target.clone(),
                    },
                });
            }
            Ok(false) => {}
            Err(violation) => return Some(violation),
        }
    }
    None
}

fn relation_kind_reaches(
    traversal: &PreparedSuccessorTraversal<'_>,
    context: &InvariantExecutionContext<'_, '_>,
    budget: &crate::config::data::RelationIntegrityScopeBudget,
    start: EntityReference,
    target: EntityReference,
) -> Result<bool, InvariantViolation> {
    let mut visited = std::collections::BTreeSet::new();
    let mut frontier = vec![start.clone()];
    let mut traversal_budget =
        RelationTraversalBudget::for_planned_successors(budget, traversal.planned_successors);

    visited.insert(start);
    traversal_budget.record_entity_visit().map_err(|_| {
        traversal_budget_exceeded_violation(
            traversal.class,
            traversal.contract_id,
            traversal.relation_kind_id,
            traversal_budget,
            traversal.planned_successors,
        )
    })?;

    while let Some(entity_id) = frontier.pop() {
        if !context.checkpoint(1) {
            return Ok(false);
        }
        for next in traversal.successors(&entity_id, &mut traversal_budget, context)? {
            if !context.checkpoint(1) {
                return Ok(false);
            }
            if next == target {
                return Ok(true);
            }
            if !visited.insert(next.clone()) {
                continue;
            }
            traversal_budget.record_entity_visit().map_err(|_| {
                traversal_budget_exceeded_violation(
                    traversal.class,
                    traversal.contract_id,
                    traversal.relation_kind_id,
                    traversal_budget,
                    traversal.planned_successors,
                )
            })?;
            frontier.push(next);
        }
    }

    Ok(false)
}
