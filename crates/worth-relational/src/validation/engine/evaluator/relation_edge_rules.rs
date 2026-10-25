use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::{
    EndpointDeletionIntegrityMode, LoweredEndpointDeletionIntegrityContract,
    LoweredSymmetryContract, LoweredUniquenessContract, SymmetryMode, UniquenessScope,
};
use crate::transactions::data::EntityReference;
use crate::validation::data::{InvariantClass, InvariantViolation, InvariantViolationFields};

use super::super::context::InvariantExecutionContext;
use super::common::{canonicalize_violations, relation_violation};

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

pub(super) fn evaluate_endpoint_deletion_integrity_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredEndpointDeletionIntegrityContract,
) -> Vec<InvariantViolation> {
    let scope = match context.required_relation_integrity_scope(contract.relation_kind_id, class) {
        Ok(scope) => scope,
        Err(violation) => return vec![violation],
    };
    if scope.deleted_entities.is_empty() {
        return Vec::new();
    }
    context.metrics().count_relation_contracts_evaluated(1);
    let mut violations = Vec::new();
    for entity_id in &scope.deleted_entities {
        if !context.checkpoint(1) {
            return Vec::new();
        }
        let endpoint_key = super::super::request::PreparedRelationEndpointKey {
            entity_id: EntityReference::Existing(*entity_id),
        };
        let live_relations = scope
            .source_counts
            .get(&endpoint_key)
            .copied()
            .unwrap_or_default()
            + scope
                .target_counts
                .get(&endpoint_key)
                .copied()
                .unwrap_or_default();
        context.metrics().count_relation_endpoint_deletion_checks(1);
        if live_relations > 0 {
            match contract.mode {
                EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations => {
                    if !context.claim_result(4096) {
                        return Vec::new();
                    }
                    violations.push(relation_violation(
                        class,
                        DiagnosticCode::RelationEndpointDeletionIntegrityViolation,
                        format!(
                            "relation contract '{}' forbids deleting endpoint {:?} while {} relation endpoints remain live",
                            contract.contract_id, entity_id, live_relations
                        ),
                        InvariantViolationFields::RelationEndpointDeletionIntegrity {
                            contract_id: contract.contract_id.clone(),
                            relation_kind_id: contract.relation_kind_id,
                            entity_id: *entity_id,
                            remaining_relation_endpoint_count: live_relations,
                            mode: contract.mode,
                            cascade_delete_policy: None,
                        },
                    ));
                }
                EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit => {
                    if contract.cascade_delete_policy
                        != crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations
                    {
                        if !context.claim_result(4096) {
                            return Vec::new();
                        }
                        violations.push(relation_violation(
                            class,
                            DiagnosticCode::RelationEndpointDeletionIntegrityViolation,
                            format!(
                                "relation contract '{}' requires deleting dependent relations in the same commit before deleting endpoint {:?}",
                                contract.contract_id, entity_id
                            ),
                            InvariantViolationFields::RelationEndpointDeletionIntegrity {
                                contract_id: contract.contract_id.clone(),
                                relation_kind_id: contract.relation_kind_id,
                                entity_id: *entity_id,
                                remaining_relation_endpoint_count: live_relations,
                                mode: contract.mode,
                                cascade_delete_policy: Some(contract.cascade_delete_policy),
                            },
                        ));
                    }
                }
                EndpointDeletionIntegrityMode::RequireRelationRetirement => {
                    if contract.cascade_delete_policy
                        != crate::config::data::CascadeDeletePolicy::RetainDanglingForAudit
                    {
                        if !context.claim_result(4096) {
                            return Vec::new();
                        }
                        violations.push(relation_violation(
                            class,
                            DiagnosticCode::RelationEndpointDeletionIntegrityViolation,
                            format!(
                                "relation contract '{}' requires audit-retained relation retirement before deleting endpoint {:?}",
                                contract.contract_id, entity_id
                            ),
                            InvariantViolationFields::RelationEndpointDeletionIntegrity {
                                contract_id: contract.contract_id.clone(),
                                relation_kind_id: contract.relation_kind_id,
                                entity_id: *entity_id,
                                remaining_relation_endpoint_count: live_relations,
                                mode: contract.mode,
                                cascade_delete_policy: Some(contract.cascade_delete_policy),
                            },
                        ));
                    }
                }
            }
        }
    }
    canonicalize_violations(violations)
}
