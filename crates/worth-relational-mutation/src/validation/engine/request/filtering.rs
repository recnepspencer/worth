use crate::contracts::validation::data::rules::InvariantRule;

use super::InvariantExecutionRequest;

pub(super) fn rule_matches_plan_scope(
    request: &InvariantExecutionRequest<'_>,
    rule: &InvariantRule,
) -> bool {
    let Some(_semantic_plan) = request.semantic_plan() else {
        return true;
    };
    let Some(relation_kind_id) = super::relation_kind_scope(rule) else {
        return true;
    };
    request
        .relation_integrity_scopes()
        .and_then(|scopes| scopes.scope_for(relation_kind_id))
        .is_some_and(super::PreparedRelationIntegrityScope::should_execute)
}

#[cfg(test)]
pub(super) fn relation_rule_kind(
    rule: &crate::contracts::validation::data::rules::InvariantRule,
) -> Option<crate::contracts::identity::data::primitives::KindId> {
    match rule {
        crate::contracts::validation::data::rules::InvariantRule::EndpointKindContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::contracts::validation::data::rules::InvariantRule::CardinalityMaximumContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::contracts::validation::data::rules::InvariantRule::CardinalityMinimumContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::contracts::validation::data::rules::InvariantRule::UniquenessContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::contracts::validation::data::rules::InvariantRule::SymmetryContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::contracts::validation::data::rules::InvariantRule::EndpointDeletionIntegrityContract(
            contract,
        ) => Some(contract.relation_kind_id),
        crate::contracts::validation::data::rules::InvariantRule::AcyclicityContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::contracts::validation::data::rules::InvariantRule::PartitionIsolationContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::contracts::validation::data::rules::InvariantRule::ConnectivityMinimumContract(contract) => {
            Some(contract.relation_kind_id)
        }
        _ => None,
    }
}
