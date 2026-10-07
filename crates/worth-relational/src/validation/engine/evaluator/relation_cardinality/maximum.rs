use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredCardinalityMaximumContract;
use crate::validation::data::{
    InvariantClass, InvariantViolation, InvariantViolationFields, RelationCardinalityBoundary,
};

use super::super::super::context::InvariantExecutionContext;
use super::super::super::request::PreparedRelationIntegrityScope;
use super::super::common::{canonicalize_violations, relation_violation};

pub(in crate::validation::engine::evaluator) fn evaluate_cardinality_maximum_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMaximumContract,
) -> Vec<InvariantViolation> {
    let scope = match context.required_relation_integrity_scope(contract.relation_kind_id, class) {
        Ok(scope) => scope,
        Err(violation) => return vec![violation],
    };
    if scope.is_empty() {
        return Vec::new();
    }

    context.metrics().count_relation_contracts_evaluated(1);
    let mut violations = Vec::new();
    collect_source_maximum_violations(context, class, contract, scope, &mut violations);
    if !context.checkpoint(0) {
        return Vec::new();
    }
    collect_target_maximum_violations(context, class, contract, scope, &mut violations);
    if !context.checkpoint(0) {
        return Vec::new();
    }
    collect_pair_maximum_violations(context, class, contract, scope, &mut violations);
    canonicalize_violations(violations)
}

fn collect_source_maximum_violations(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMaximumContract,
    scope: &PreparedRelationIntegrityScope,
    violations: &mut Vec<InvariantViolation>,
) {
    let Some(limit) = contract.source_max else {
        return;
    };
    for (key, count) in &scope.source_counts {
        if !context.checkpoint(1) {
            return;
        }
        context.metrics().count_relation_cardinality_checks(1);
        if (*count as u64) <= limit {
            continue;
        }
        if !context.claim_result(4096) {
            return;
        }
        violations.push(relation_violation(
            class,
            DiagnosticCode::RelationCardinalityViolation,
            format!(
                "relation contract '{}' overflowed source cardinality for entity {:?}: {} > {}",
                contract.contract_id, key.entity_id, count, limit
            ),
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id: contract.contract_id.clone(),
                relation_kind_id: contract.relation_kind_id,
                entity_id: key.entity_id.clone(),
                boundary: RelationCardinalityBoundary::Source,
                count: *count,
                limit,
            },
        ));
    }
}

fn collect_target_maximum_violations(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMaximumContract,
    scope: &PreparedRelationIntegrityScope,
    violations: &mut Vec<InvariantViolation>,
) {
    let Some(limit) = contract.target_max else {
        return;
    };
    for (key, count) in &scope.target_counts {
        if !context.checkpoint(1) {
            return;
        }
        context.metrics().count_relation_cardinality_checks(1);
        if (*count as u64) <= limit {
            continue;
        }
        if !context.claim_result(4096) {
            return;
        }
        violations.push(relation_violation(
            class,
            DiagnosticCode::RelationCardinalityViolation,
            format!(
                "relation contract '{}' overflowed target cardinality for entity {:?}: {} > {}",
                contract.contract_id, key.entity_id, count, limit
            ),
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id: contract.contract_id.clone(),
                relation_kind_id: contract.relation_kind_id,
                entity_id: key.entity_id.clone(),
                boundary: RelationCardinalityBoundary::Target,
                count: *count,
                limit,
            },
        ));
    }
}

fn collect_pair_maximum_violations(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMaximumContract,
    scope: &PreparedRelationIntegrityScope,
    violations: &mut Vec<InvariantViolation>,
) {
    let Some(limit) = contract.pair_max else {
        return;
    };
    for (key, count) in &scope.directed_pair_counts {
        if !context.checkpoint(1) {
            return;
        }
        context.metrics().count_relation_cardinality_checks(1);
        if (*count as u64) <= limit {
            continue;
        }
        if !context.claim_result(4096) {
            return;
        }
        violations.push(relation_violation(
            class,
            DiagnosticCode::RelationCardinalityViolation,
            format!(
                "relation contract '{}' overflowed pair cardinality for {:?}->{:?}: {} > {}",
                contract.contract_id, key.source, key.target, count, limit
            ),
            InvariantViolationFields::RelationCardinalityPair {
                contract_id: contract.contract_id.clone(),
                relation_kind_id: contract.relation_kind_id,
                source: key.source.clone(),
                target: key.target.clone(),
                count: *count,
                limit,
            },
        ));
    }
}
