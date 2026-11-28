pub mod common;
pub mod partition_isolation;
pub mod record_surface_rules;
pub mod relation_cardinality;
pub mod relation_edge_rules;
pub mod relation_endpoint_kind;
pub mod relation_traversal;
pub(crate) use relation_cardinality::CurrentVersionMinimumIndex;

use crate::contracts::validation::data::execution::InvariantClass;
use crate::contracts::validation::data::results::InvariantViolation;
use crate::contracts::validation::data::rules::InvariantRule;

use super::context::InvariantExecutionContext;
use partition_isolation::evaluate_partition_isolation_contract;
use record_surface_rules::evaluate_record_surface_rule;
use relation_cardinality::{
    evaluate_cardinality_maximum_contract, evaluate_cardinality_minimum_contract,
};
use relation_edge_rules::{
    evaluate_endpoint_deletion_integrity_contract, evaluate_symmetry_contract,
    evaluate_uniqueness_contract,
};
use relation_endpoint_kind::evaluate_endpoint_kind_contract;
use relation_traversal::{evaluate_acyclicity_contract, evaluate_connectivity_minimum_contract};
use crate::validation::unique_field_probes::UniqueFieldScope;

pub(crate) fn evaluate_rule(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    rule: &InvariantRule,
) -> Vec<InvariantViolation> {
    match rule {
        InvariantRule::EntityAspectExclusion => single_violation(
            record_surface_rules::aspect_exclusion::evaluate(context, class),
        ),
        InvariantRule::LiveRecordRequiresSidecar(kind) => {
            evaluate_record_surface_rule(context, class, kind)
        }
        InvariantRule::MaxMergedIntents(limit) => {
            let merged_len = context
                .semantic_plan()
                .map(|plan| plan.operation_count())
                .unwrap_or(0);
            if merged_len > *limit {
                vec![InvariantViolation {
                    class,
                    code: crate::contracts::diagnostics::data::artifacts::DiagnosticCode::InvariantViolation,
                    detail: format!(
                        "merged commit plan has {} intents, limit is {}",
                        merged_len, limit
                    ),
                    fields: crate::contracts::validation::data::results::InvariantViolationFields::MergedIntentLimit {
                        merged_intent_count: merged_len,
                        limit: *limit,
                    },
                }]
            } else {
                Vec::new()
            }
        }
        InvariantRule::UniqueEntityAspectField {
            entity_kind_id, backing_index_id, field_locator,
        } => single_violation(context.unique_field_duplicate(*backing_index_id).and_then(|value| {
            let bytes = 4096u64
                .saturating_add(field_locator.owned_allocation_capacity_bytes() as u64)
                .saturating_add(value.owned_allocation_capacity_bytes() as u64);
            context.claim_result(bytes).then(|| UniqueFieldScope {
                entity_kind_id: *entity_kind_id,
                backing_index_id: *backing_index_id,
                field_locator,
            }.violation(class, value.clone()))
        })),
        InvariantRule::EndpointKindContract(contract) => {
            evaluate_endpoint_kind_contract(context, class, contract)
        }
        InvariantRule::CardinalityMaximumContract(contract) => {
            evaluate_cardinality_maximum_contract(context, class, contract)
        }
        InvariantRule::CardinalityMinimumContract(contract) => {
            evaluate_cardinality_minimum_contract(context, class, contract)
        }
        InvariantRule::UniquenessContract(contract) => {
            evaluate_uniqueness_contract(context, class, contract)
        }
        InvariantRule::SymmetryContract(contract) => {
            evaluate_symmetry_contract(context, class, contract)
        }
        InvariantRule::EndpointDeletionIntegrityContract(contract) => {
            evaluate_endpoint_deletion_integrity_contract(context, class, contract)
        }
        InvariantRule::PartitionIsolationContract(contract) => {
            evaluate_partition_isolation_contract(context, class, contract)
        }
        InvariantRule::AcyclicityContract(contract) => {
            single_violation(evaluate_acyclicity_contract(context, class, contract))
        }
        InvariantRule::ConnectivityMinimumContract(contract) => single_violation(
            evaluate_connectivity_minimum_contract(context, class, contract),
        ),
    }
}

fn single_violation(violation: Option<InvariantViolation>) -> Vec<InvariantViolation> {
    violation.into_iter().collect()
}
