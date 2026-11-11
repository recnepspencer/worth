//! Endpoint deletion classifies the prepared live incidence before rendering a violation.
use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::EntityId;

#[derive(Clone, Copy)]
pub(super) enum EndpointDeletionRefusal {
    LiveIncidence,
    RelationDeletionRequired,
    RelationRetirementRequired,
}

pub(super) fn classify(
    contract: &LoweredEndpointDeletionIntegrityContract,
    live_relations: usize,
) -> Option<EndpointDeletionRefusal> {
    if live_relations == 0 {
        return None;
    }
    match contract.mode {
        EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations => {
            Some(EndpointDeletionRefusal::LiveIncidence)
        }
        EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit => {
            (contract.cascade_delete_policy != CascadeDeletePolicy::CascadeDeleteRelations)
                .then_some(EndpointDeletionRefusal::RelationDeletionRequired)
        }
        EndpointDeletionIntegrityMode::RequireRelationRetirement => {
            (contract.cascade_delete_policy != CascadeDeletePolicy::RetainDanglingForAudit)
                .then_some(EndpointDeletionRefusal::RelationRetirementRequired)
        }
    }
}

pub(super) fn violation(
    class: InvariantClass,
    contract: &LoweredEndpointDeletionIntegrityContract,
    entity_id: EntityId,
    live_relations: usize,
    refusal: EndpointDeletionRefusal,
) -> InvariantViolation {
    let (message, policy) = match refusal {
        EndpointDeletionRefusal::LiveIncidence => (
            format!("relation contract '{}' forbids deleting endpoint {:?} while {} relation endpoints remain live", contract.contract_id, entity_id, live_relations),
            None,
        ),
        EndpointDeletionRefusal::RelationDeletionRequired => (
            format!("relation contract '{}' requires deleting dependent relations in the same commit before deleting endpoint {:?}", contract.contract_id, entity_id),
            Some(contract.cascade_delete_policy),
        ),
        EndpointDeletionRefusal::RelationRetirementRequired => (
            format!("relation contract '{}' requires audit-retained relation retirement before deleting endpoint {:?}", contract.contract_id, entity_id),
            Some(contract.cascade_delete_policy),
        ),
    };
    relation_violation(
        class,
        DiagnosticCode::RelationEndpointDeletionIntegrityViolation,
        message,
        InvariantViolationFields::RelationEndpointDeletionIntegrity {
            contract_id: contract.contract_id.clone(),
            relation_kind_id: contract.relation_kind_id,
            entity_id,
            remaining_relation_endpoint_count: live_relations,
            mode: contract.mode,
            cascade_delete_policy: policy,
        },
    )
}

pub(in crate::validation::engine::evaluator) fn evaluate_endpoint_deletion_integrity_contract(
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
        let endpoint_key = crate::validation::engine::request::PreparedRelationEndpointKey {
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
        if let Some(refusal) = classify(contract, live_relations) {
            if !context.claim_result(4096) {
                return Vec::new();
            }
            violations.push(violation(
                class,
                contract,
                *entity_id,
                live_relations,
                refusal,
            ));
        }
    }
    canonicalize_violations(violations)
}
