use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::LoweredPartitionIsolationContract;
use crate::validation::data::{InvariantClass, InvariantViolation, InvariantViolationFields};

use super::super::context::InvariantExecutionContext;
use super::common::canonicalize_violations;

pub(super) fn evaluate_partition_isolation_contract(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    contract: &LoweredPartitionIsolationContract,
) -> Vec<InvariantViolation> {
    let scope = match context.required_relation_integrity_scope(contract.relation_kind_id, class) {
        Ok(scope) => scope,
        Err(violation) => return vec![violation],
    };
    context.metrics().count_relation_contracts_evaluated(1);
    let mut violations = Vec::new();
    for edge in &scope.planned_edges {
        if !context.checkpoint(1) {
            return Vec::new();
        }
        context.metrics().count_relation_slot_scans(1);
        if edge.source.partition_id() != edge.target.partition_id() {
            if !context.claim_result(4096) {
                return Vec::new();
            }
            violations.push(partition_isolation_violation(
                class,
                contract,
                None,
                edge.source.partition_id(),
                edge.target.partition_id(),
            ));
        }
    }
    for edge in &scope.visible_edges {
        if !context.checkpoint(1) {
            return Vec::new();
        }
        context.metrics().count_relation_slot_scans(1);
        if edge.source.partition_id != edge.target.partition_id {
            if !context.claim_result(4096) {
                return Vec::new();
            }
            violations.push(partition_isolation_violation(
                class,
                contract,
                Some(edge.relation_id),
                edge.source.partition_id,
                edge.target.partition_id,
            ));
        }
    }
    canonicalize_violations(violations)
}

fn partition_isolation_violation(
    class: crate::validation::data::InvariantClass,
    contract: &LoweredPartitionIsolationContract,
    relation_id: Option<crate::identity::data::RelationId>,
    source_partition_id: crate::identity::data::PartitionId,
    target_partition_id: crate::identity::data::PartitionId,
) -> InvariantViolation {
    InvariantViolation {
        class,
        code: DiagnosticCode::InvariantViolation,
        detail: format!(
            "relation contract '{}' requires same-partition endpoints for relation kind {:?}",
            contract.contract_id, contract.relation_kind_id
        ),
        fields: InvariantViolationFields::PartitionIsolation {
            contract_id: contract.contract_id.clone(),
            relation_kind_id: contract.relation_kind_id,
            relation_id,
            source_partition_id,
            target_partition_id,
        },
    }
}
