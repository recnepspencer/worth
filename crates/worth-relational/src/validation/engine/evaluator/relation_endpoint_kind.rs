use crate::config::data::CrossContextPolicy;
use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredEndpointKindContract;
use crate::validation::data::{
    InvariantClass, InvariantViolation, InvariantViolationFields, RelationEndpointBoundary,
};

use super::super::context::InvariantExecutionContext;
use super::common::{canonicalize_violations, entity_reference_kind, relation_violation};

pub(super) fn evaluate_endpoint_kind_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredEndpointKindContract,
) -> Vec<InvariantViolation> {
    let scope = match context.required_relation_integrity_scope(contract.relation_kind_id, class) {
        Ok(scope) => scope,
        Err(violation) => return vec![violation],
    };
    if scope.planned_edges.is_empty() {
        return Vec::new();
    }

    context.metrics().count_relation_contracts_evaluated(1);
    let mut violations = Vec::new();
    for edge in &scope.planned_edges {
        if !context.checkpoint(1) {
            return Vec::new();
        }
        context.metrics().count_relation_endpoint_kind_checks(1);
        let source_kind = match entity_reference_kind(context, class, &edge.source) {
            Ok(Some(kind_id)) => kind_id,
            Ok(None) => continue,
            Err(violation) => {
                if !context
                    .claim_contract_violation(&contract.contract_id, &[&edge.source, &edge.target])
                {
                    return Vec::new();
                }
                violations.push(violation);
                continue;
            }
        };
        let target_kind = match entity_reference_kind(context, class, &edge.target) {
            Ok(Some(kind_id)) => kind_id,
            Ok(None) => continue,
            Err(violation) => {
                if !context
                    .claim_contract_violation(&contract.contract_id, &[&edge.source, &edge.target])
                {
                    return Vec::new();
                }
                violations.push(violation);
                continue;
            }
        };
        if !contract.allows_source_kind(source_kind) {
            if !context
                .claim_contract_violation(&contract.contract_id, &[&edge.source, &edge.target])
            {
                return Vec::new();
            }
            violations.push(relation_violation(
                class,
                DiagnosticCode::RelationEndpointKindViolation,
                format!(
                    "relation contract '{}' rejected source kind {:?} for relation kind {:?}",
                    contract.contract_id, source_kind, contract.relation_kind_id
                ),
                InvariantViolationFields::RelationEndpointKindMismatch {
                    contract_id: contract.contract_id.clone(),
                    relation_kind_id: contract.relation_kind_id,
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    source_kind_id: source_kind,
                    target_kind_id: target_kind,
                    boundary: RelationEndpointBoundary::Source,
                },
            ));
        }
        if !contract.allows_target_kind(target_kind) {
            if !context
                .claim_contract_violation(&contract.contract_id, &[&edge.source, &edge.target])
            {
                return Vec::new();
            }
            violations.push(relation_violation(
                class,
                DiagnosticCode::RelationEndpointKindViolation,
                format!(
                    "relation contract '{}' rejected target kind {:?} for relation kind {:?}",
                    contract.contract_id, target_kind, contract.relation_kind_id
                ),
                InvariantViolationFields::RelationEndpointKindMismatch {
                    contract_id: contract.contract_id.clone(),
                    relation_kind_id: contract.relation_kind_id,
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    source_kind_id: source_kind,
                    target_kind_id: target_kind,
                    boundary: RelationEndpointBoundary::Target,
                },
            ));
        }
        if !contract.self_edges_allowed && edge.source == edge.target {
            if !context
                .claim_contract_violation(&contract.contract_id, &[&edge.source, &edge.target])
            {
                return Vec::new();
            }
            violations.push(relation_violation(
                class,
                DiagnosticCode::RelationEndpointKindViolation,
                format!(
                    "relation contract '{}' forbids self edges for relation kind {:?}",
                    contract.contract_id, contract.relation_kind_id
                ),
                InvariantViolationFields::RelationEndpointKindSelfEdge {
                    contract_id: contract.contract_id.clone(),
                    relation_kind_id: contract.relation_kind_id,
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    self_edge: true,
                },
            ));
        }
        if edge.source.partition_id() != edge.target.partition_id()
            && contract.cross_context_policy != CrossContextPolicy::AllowExplicit
        {
            if !context
                .claim_contract_violation(&contract.contract_id, &[&edge.source, &edge.target])
            {
                return Vec::new();
            }
            violations.push(relation_violation(
                class,
                DiagnosticCode::InvalidRelationEndpoint,
                format!(
                    "relation contract '{}' forbids cross-context endpoints for relation kind {:?}",
                    contract.contract_id, contract.relation_kind_id
                ),
                InvariantViolationFields::RelationEndpointKindCrossContext {
                    contract_id: contract.contract_id.clone(),
                    relation_kind_id: contract.relation_kind_id,
                    source_partition_id: edge.source.partition_id(),
                    target_partition_id: edge.target.partition_id(),
                },
            ));
        }
    }
    canonicalize_violations(violations)
}
