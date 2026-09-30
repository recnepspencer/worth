use std::collections::BTreeSet;

use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredConnectivityMinimumContract;
use crate::transactions::data::EntityReference;
use crate::validation::data::{InvariantClass, InvariantViolation, InvariantViolationFields};

use super::super::super::context::InvariantExecutionContext;
use super::super::common::{contract_candidate_kind_matches, entity_reference_kind};
use super::planned_successors::planned_successor_map;
use super::relation_successors::PreparedSuccessorTraversal;
use super::traversal_budget::{traversal_budget_exceeded_violation, RelationTraversalBudget};
use super::visible_entities::visible_entities_of_kinds;

pub(in crate::validation::engine::evaluator) fn evaluate_connectivity_minimum_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredConnectivityMinimumContract,
) -> Option<InvariantViolation> {
    let scope = match context.required_relation_integrity_scope(contract.relation_kind_id, class) {
        Ok(scope) => scope,
        Err(violation) => return Some(violation),
    };
    context.metrics().count_relation_contracts_evaluated(1);
    let source_entities = visible_entities_of_kinds(context, &contract.source_kind_ids);
    if !context.checkpoint(0) {
        return None;
    }
    if source_entities.is_empty() {
        return None;
    }

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
    for source in source_entities {
        if !context.checkpoint(1) {
            return None;
        }
        let reachable_target_count = match reachable_target_count_for_connectivity(
            context,
            &traversal,
            source.clone(),
            &contract.target_kind_ids,
        ) {
            Ok(count) => count,
            Err(violation) => return Some(violation),
        };
        if reachable_target_count < contract.minimum_reachable_targets as usize {
            return Some(InvariantViolation {
                class,
                code: DiagnosticCode::InvariantViolation,
                detail: format!(
                    "connectivity minimum contract '{}' requires at least {} reachable target(s) for {:?}",
                    contract.contract_id,
                    contract.minimum_reachable_targets,
                    source
                ),
                fields: InvariantViolationFields::ConnectivityMinimum {
                    contract_id: contract.contract_id.clone(),
                    relation_kind_id: contract.relation_kind_id,
                    source,
                    reachable_target_count,
                    minimum_reachable_targets: contract.minimum_reachable_targets,
                },
            });
        }
    }
    None
}

fn reachable_target_count_for_connectivity(
    context: &InvariantExecutionContext<'_, '_>,
    traversal: &PreparedSuccessorTraversal<'_>,
    source: EntityReference,
    target_kind_ids: &[crate::identity::data::KindId],
) -> Result<usize, InvariantViolation> {
    let mut visited = BTreeSet::new();
    let mut frontier = vec![source.clone()];
    let mut reachable_targets = BTreeSet::new();
    let mut traversal_budget = RelationTraversalBudget::for_planned_successors(
        context.relation_integrity_scope_budget(),
        traversal.planned_successors,
    );

    visited.insert(source);
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
            return Ok(0);
        }
        for next in traversal.successors(&entity_id, &mut traversal_budget, context)? {
            if !context.checkpoint(1) {
                return Ok(0);
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
            if let Some(kind_id) = entity_reference_kind(context, traversal.class, &next)? {
                if contract_candidate_kind_matches(kind_id, target_kind_ids) {
                    reachable_targets.insert(next.clone());
                }
            }
            frontier.push(next);
        }
    }

    Ok(reachable_targets.len())
}
