use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredCardinalityMinimumContract;
use crate::validation::data::{
    InvariantClass, InvariantViolation, InvariantViolationFields, RelationCardinalityBoundary,
};

use super::super::super::context::InvariantExecutionContext;
use super::super::common::{canonicalize_violations, relation_violation};
use super::minimum_visible_counts::visible_relation_counts;

pub(in crate::validation::engine::evaluator) fn evaluate_cardinality_minimum_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMinimumContract,
) -> Vec<InvariantViolation> {
    let snapshot = visible_relation_counts(context, contract);
    if !context.checkpoint(0) {
        return Vec::new();
    }
    context.metrics().count_relation_contracts_evaluated(1);
    context
        .metrics()
        .count_relation_cardinality_minimum_certification(
            1,
            snapshot.entity_slot_scans,
            snapshot.relation_slot_scans,
        );

    let mut violations = Vec::new();
    collect_source_minimum_violations(context, class, contract, &snapshot, &mut violations);
    if !context.checkpoint(0) {
        return Vec::new();
    }
    collect_target_minimum_violations(context, class, contract, &snapshot, &mut violations);
    if !context.checkpoint(0) {
        return Vec::new();
    }
    collect_pair_minimum_violations(context, class, contract, &snapshot, &mut violations);
    canonicalize_violations(violations)
}

fn collect_source_minimum_violations(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMinimumContract,
    snapshot: &super::minimum_visible_counts::VisibleRelationCountSnapshot,
    violations: &mut Vec<InvariantViolation>,
) {
    let Some(minimum) = contract.source_min else {
        return;
    };
    for entity_id in snapshot.candidate_source_entities.iter().cloned() {
        if !context.checkpoint(1) {
            return;
        }
        let count = snapshot
            .source_counts
            .get(&entity_id)
            .copied()
            .unwrap_or_default();
        context.metrics().count_relation_cardinality_checks(1);
        if (count as u64) >= minimum {
            continue;
        }
        if !context.claim_result(4096) {
            return;
        }
        violations.push(relation_violation(
            class,
            DiagnosticCode::RelationCardinalityViolation,
            format!(
                "relation contract '{}' underflowed source cardinality for entity {:?}: {} < {}",
                contract.contract_id, entity_id, count, minimum
            ),
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id: contract.contract_id.clone(),
                relation_kind_id: contract.relation_kind_id,
                entity_id,
                boundary: RelationCardinalityBoundary::Source,
                count,
                limit: minimum,
            },
        ));
    }
}

fn collect_target_minimum_violations(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMinimumContract,
    snapshot: &super::minimum_visible_counts::VisibleRelationCountSnapshot,
    violations: &mut Vec<InvariantViolation>,
) {
    let Some(minimum) = contract.target_min else {
        return;
    };
    for entity_id in snapshot.candidate_target_entities.iter().cloned() {
        if !context.checkpoint(1) {
            return;
        }
        let count = snapshot
            .target_counts
            .get(&entity_id)
            .copied()
            .unwrap_or_default();
        context.metrics().count_relation_cardinality_checks(1);
        if (count as u64) >= minimum {
            continue;
        }
        if !context.claim_result(4096) {
            return;
        }
        violations.push(relation_violation(
            class,
            DiagnosticCode::RelationCardinalityViolation,
            format!(
                "relation contract '{}' underflowed target cardinality for entity {:?}: {} < {}",
                contract.contract_id, entity_id, count, minimum
            ),
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id: contract.contract_id.clone(),
                relation_kind_id: contract.relation_kind_id,
                entity_id,
                boundary: RelationCardinalityBoundary::Target,
                count,
                limit: minimum,
            },
        ));
    }
}

fn collect_pair_minimum_violations(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredCardinalityMinimumContract,
    snapshot: &super::minimum_visible_counts::VisibleRelationCountSnapshot,
    violations: &mut Vec<InvariantViolation>,
) {
    let Some(minimum) = contract.pair_min else {
        return;
    };
    for ((source, target), count) in &snapshot.directed_pair_counts {
        if !context.checkpoint(1) {
            return;
        }
        context.metrics().count_relation_cardinality_checks(1);
        if (*count as u64) >= minimum {
            continue;
        }
        if !context.claim_result(4096) {
            return;
        }
        violations.push(relation_violation(
            class,
            DiagnosticCode::RelationCardinalityViolation,
            format!(
                "relation contract '{}' underflowed pair cardinality for {:?}->{:?}: {} < {}",
                contract.contract_id, source, target, count, minimum
            ),
            InvariantViolationFields::RelationCardinalityPair {
                contract_id: contract.contract_id.clone(),
                relation_kind_id: contract.relation_kind_id,
                source: source.clone(),
                target: target.clone(),
                count: *count,
                limit: minimum,
            },
        ));
    }
}
