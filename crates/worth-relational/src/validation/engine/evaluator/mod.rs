mod common;
mod partition_isolation;
mod record_surface_rules;
mod relation_cardinality;
mod relation_edge_rules;
mod relation_endpoint_kind;
mod relation_traversal;
mod unique_entity_fields;
pub(crate) use relation_cardinality::CurrentVersionMinimumIndex;

use crate::validation::data::{InvariantClass, InvariantRule, InvariantViolation};

use super::context::InvariantExecutionContext;
use partition_isolation::evaluate_partition_isolation_contract;
use record_surface_rules::{evaluate_record_surface_rule, evaluate_snapshot_entity_limit_rule};
use relation_cardinality::{
    evaluate_cardinality_maximum_contract, evaluate_cardinality_minimum_contract,
};
use relation_edge_rules::{
    evaluate_endpoint_deletion_integrity_contract, evaluate_symmetry_contract,
    evaluate_uniqueness_contract,
};
use relation_endpoint_kind::evaluate_endpoint_kind_contract;
use relation_traversal::{evaluate_acyclicity_contract, evaluate_connectivity_minimum_contract};
use unique_entity_fields::evaluate_unique_entity_aspect_field;

pub(crate) fn evaluate_rule(
    context: &InvariantExecutionContext<'_>,
    class: InvariantClass,
    rule: &InvariantRule,
) -> Vec<InvariantViolation> {
    match rule {
        InvariantRule::LiveRecordRequiresSidecar(kind) => {
            evaluate_record_surface_rule(context, class, kind)
        }
        InvariantRule::MaxMergedIntents(limit) => {
            let merged_len = context
                .merged_plan()
                .map(|plan| plan.merged_intents.len())
                .unwrap_or(0);
            if merged_len > *limit {
                vec![InvariantViolation {
                    class,
                    code: crate::diagnostics::data::DiagnosticCode::InvariantViolation,
                    detail: format!(
                        "merged commit plan has {} intents, limit is {}",
                        merged_len, limit
                    ),
                    fields: crate::validation::data::InvariantViolationFields::MergedIntentLimit {
                        merged_intent_count: merged_len,
                        limit: *limit,
                    },
                }]
            } else {
                Vec::new()
            }
        }
        InvariantRule::RelationIntegrityScopeBudget(_) => Vec::new(),
        InvariantRule::MaxSnapshotEntities(limit) => {
            single_violation(evaluate_snapshot_entity_limit_rule(context, class, *limit))
        }
        InvariantRule::UniqueEntityAspectField { field_locator } => single_violation(
            evaluate_unique_entity_aspect_field(context, class, field_locator),
        ),
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
