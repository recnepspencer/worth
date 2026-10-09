use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredAcyclicityContract;
use crate::transactions::data::EntityReference;
use crate::validation::data::{InvariantClass, InvariantViolation, InvariantViolationFields};

use super::super::super::context::InvariantExecutionContext;
use super::planned_successors::planned_successor_map;
use super::relation_successors::PreparedSuccessorTraversal;

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
        planned_successors: &planned_successors,
    };
    for edge in &scope.planned_edges {
        if !context.checkpoint(1) {
            return None;
        }
        context.metrics().count_relation_slot_scans(1);
        let reaches_cycle = if edge.source == edge.target {
            true
        } else {
            relation_kind_reaches(
                &traversal,
                context,
                edge.target.clone(),
                edge.source.clone(),
            )
        };
        if reaches_cycle {
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
    }
    None
}

fn relation_kind_reaches(
    traversal: &PreparedSuccessorTraversal<'_>,
    context: &InvariantExecutionContext<'_, '_>,
    start: EntityReference,
    target: EntityReference,
) -> bool {
    let mut visited = std::collections::BTreeSet::new();
    let mut frontier = vec![start.clone()];

    visited.insert(start);

    while let Some(entity_id) = frontier.pop() {
        if !context.checkpoint(1) {
            return false;
        }
        for next in traversal.successors(&entity_id, context) {
            if !context.checkpoint(1) {
                return false;
            }
            if next == target {
                return true;
            }
            if !visited.insert(next.clone()) {
                continue;
            }
            frontier.push(next);
        }
    }

    false
}
