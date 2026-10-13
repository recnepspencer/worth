use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredAcyclicityContract;
use crate::validation::data::{InvariantClass, InvariantViolation, InvariantViolationFields};

use super::super::super::context::InvariantExecutionContext;
mod graph;
mod index_order;
mod search;
use graph::TraversalGraph;
use search::ReachabilitySearch;

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
    let graph = TraversalGraph::prepare(scope, context)?;
    let mut search = ReachabilitySearch::prepare(&graph, context)?;
    for edge in &scope.planned_edges {
        if !context.checkpoint(1) {
            return None;
        }
        context.metrics().count_relation_slot_scans(1);
        let reaches_cycle = if edge.source == edge.target {
            true
        } else {
            search.reaches(&graph, &edge.target, &edge.source, context)?
        };
        if reaches_cycle {
            if !context
                .claim_contract_violation(&contract.contract_id, &[&edge.source, &edge.target])
            {
                return None;
            }
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
