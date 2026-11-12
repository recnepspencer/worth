use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::{
    EndpointDeletionIntegrityMode, LoweredEndpointDeletionIntegrityContract,
    LoweredSymmetryContract, LoweredUniquenessContract, SymmetryMode, UniquenessScope,
};
use crate::transactions::data::EntityReference;
use crate::validation::data::{InvariantClass, InvariantViolation, InvariantViolationFields};

use super::super::context::InvariantExecutionContext;
use super::common::{canonicalize_violations, relation_violation};

mod endpoint_deletion;
pub(super) use endpoint_deletion::evaluate_endpoint_deletion_integrity_contract;
#[cfg(test)]
mod endpoint_deletion_tests;

pub(super) fn evaluate_uniqueness_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredUniquenessContract,
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
    match contract.scope {
        UniquenessScope::DirectedSemanticEdge => {
            for (key, count) in &scope.directed_pair_counts {
                if !context.checkpoint(1) {
                    return Vec::new();
                }
                context.metrics().count_relation_uniqueness_checks(1);
                if *count > 1 {
                    if !context.claim_result(4096) {
                        return Vec::new();
                    }
                    violations.push(relation_violation(
                        class,
                        DiagnosticCode::RelationUniquenessViolation,
                        format!(
                            "relation contract '{}' forbids duplicate directed edge {:?}->{:?}",
                            contract.contract_id, key.source, key.target
                        ),
                        InvariantViolationFields::RelationUniqueness {
                            contract_id: contract.contract_id.clone(),
                            relation_kind_id: contract.relation_kind_id,
                            scope: contract.scope,
                            source: key.source.clone(),
                            target: key.target.clone(),
                            count: *count,
                        },
                    ));
                }
            }
        }
        UniquenessScope::NormalizedSymmetricEdge => {
            for (key, count) in &scope.normalized_pair_counts {
                if !context.checkpoint(1) {
                    return Vec::new();
                }
                context.metrics().count_relation_uniqueness_checks(1);
                if *count > 1 {
                    if !context.claim_result(4096) {
                        return Vec::new();
                    }
                    violations.push(relation_violation(
                        class,
                        DiagnosticCode::RelationUniquenessViolation,
                        format!(
                            "relation contract '{}' forbids duplicate normalized edge {:?}<->{:?}",
                            contract.contract_id, key.source, key.target
                        ),
                        InvariantViolationFields::RelationUniqueness {
                            contract_id: contract.contract_id.clone(),
                            relation_kind_id: contract.relation_kind_id,
                            scope: contract.scope,
                            source: key.source.clone(),
                            target: key.target.clone(),
                            count: *count,
                        },
                    ));
                }
            }
        }
    }
    canonicalize_violations(violations)
}

pub(super) fn evaluate_symmetry_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredSymmetryContract,
) -> Vec<InvariantViolation> {
    let scope = match context.required_relation_integrity_scope(contract.relation_kind_id, class) {
        Ok(scope) => scope,
        Err(violation) => return vec![violation],
    };
    let paired = matches!(
        contract.mode,
        SymmetryMode::PairedInverseRequired | SymmetryMode::PairedTwinRequired
    );
    if if paired {
        scope.affected_pairs.is_empty()
    } else {
        scope.planned_edges.is_empty()
    } {
        return Vec::new();
    }
    context.metrics().count_relation_contracts_evaluated(1);
    let mut violations = Vec::new();
    // Pair counts may contain incidental edges from partial incidence reads.
    // Only explicit changed/revalidated pairs define paired symmetry's domain.
    let planned = scope
        .planned_edges
        .iter()
        .take(if paired { 0 } else { scope.planned_edges.len() })
        .map(|edge| (&edge.source, &edge.target, None));
    let surviving = scope
        .affected_pairs
        .iter()
        .take(if paired {
            scope.affected_pairs.len()
        } else {
            0
        })
        .map(|pair| (&pair.source, &pair.target, Some(pair)));
    for (source, target, affected) in planned.chain(surviving) {
        if !context.checkpoint(1) {
            return Vec::new();
        }
        if let Some(pair) = affected
            && scope
                .directed_pair_counts
                .get(pair)
                .copied()
                .unwrap_or_default()
                == 0
        {
            continue;
        }
        context.metrics().count_relation_symmetry_checks(1);
        match contract.mode {
            SymmetryMode::CanonicalUndirected => {
                if target < source {
                    if !context.claim_result(4096) {
                        return Vec::new();
                    }
                    violations.push(relation_violation(
                        class,
                        DiagnosticCode::RelationSymmetryViolation,
                        format!(
                            "relation contract '{}' requires canonical undirected ordering",
                            contract.contract_id
                        ),
                        InvariantViolationFields::RelationSymmetry {
                            contract_id: contract.contract_id.clone(),
                            relation_kind_id: contract.relation_kind_id,
                            source: source.clone(),
                            target: target.clone(),
                            mode: contract.mode,
                        },
                    ));
                }
            }
            SymmetryMode::PairedInverseRequired | SymmetryMode::PairedTwinRequired => {
                let inverse = super::super::request::PreparedRelationPairKey {
                    source: target.clone(),
                    target: source.clone(),
                };
                if scope
                    .directed_pair_counts
                    .get(&inverse)
                    .copied()
                    .unwrap_or_default()
                    == 0
                {
                    if !context.claim_result(4096) {
                        return Vec::new();
                    }
                    violations.push(relation_violation(
                        class,
                        DiagnosticCode::RelationSymmetryViolation,
                        format!(
                            "relation contract '{}' requires an inverse/twin edge for {:?}->{:?}",
                            contract.contract_id, source, target
                        ),
                        InvariantViolationFields::RelationSymmetry {
                            contract_id: contract.contract_id.clone(),
                            relation_kind_id: contract.relation_kind_id,
                            source: source.clone(),
                            target: target.clone(),
                            mode: contract.mode,
                        },
                    ));
                }
            }
            SymmetryMode::InverseProhibited => {
                let inverse = super::super::request::PreparedRelationPairKey {
                    source: target.clone(),
                    target: source.clone(),
                };
                if scope
                    .directed_pair_counts
                    .get(&inverse)
                    .copied()
                    .unwrap_or_default()
                    > 0
                {
                    if !context.claim_result(4096) {
                        return Vec::new();
                    }
                    violations.push(relation_violation(
                        class,
                        DiagnosticCode::RelationSymmetryViolation,
                        format!(
                            "relation contract '{}' prohibits inverse duplication for {:?}->{:?}",
                            contract.contract_id, source, target
                        ),
                        InvariantViolationFields::RelationSymmetry {
                            contract_id: contract.contract_id.clone(),
                            relation_kind_id: contract.relation_kind_id,
                            source: source.clone(),
                            target: target.clone(),
                            mode: contract.mode,
                        },
                    ));
                }
            }
        }
    }
    canonicalize_violations(violations)
}
