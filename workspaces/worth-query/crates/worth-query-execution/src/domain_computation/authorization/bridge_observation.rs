//! Runtime Bridge observations derived from neutral Relational evidence.

use worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence;
use worth_runtime_bridge::facade::{
    BridgeAuthorizationClauseObservation, BridgeAuthorizationDependencyCardinality,
    BridgeAuthorizationObservation, BridgeAuthorizationRequirementObservation,
    BridgeAuthorizationRuleObservation,
};

use super::installed_policy::WorthQueryInstalledAuthorizationPolicy;
use super::{WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind};

mod admitted;
pub(super) use admitted::{lower_bridge_observation_admitted, BridgeObservationPreparationStop};

pub(super) fn lower_bridge_observation(
    installed: &WorthQueryInstalledAuthorizationPolicy,
    evidence: &RelationalAuthorizationObservationEvidence,
    dependency_identity: [u8; 32],
    policy: &str,
) -> Result<BridgeAuthorizationObservation, WorthQueryOperationAuthorizationDenial> {
    if evidence.paths().len() != installed.bridge_path_bindings().len() {
        return Err(invalid_shape(policy));
    }
    let mut observed_rules = Vec::with_capacity(installed.bridge_rule_bindings().len());
    let mut observed_paths = vec![false; evidence.paths().len()];
    let mut binding = ObservationBinding {
        installed,
        evidence,
        observed_paths: &mut observed_paths,
        policy,
    };
    for rule_binding in installed.bridge_rule_bindings() {
        observed_rules
            .push(binding.observe_rule(rule_binding.rule(), rule_binding.path_indices())?);
    }
    if observed_paths.iter().any(|observed| !observed) {
        return Err(invalid_shape(policy));
    }
    Ok(BridgeAuthorizationObservation::new(
        installed.correspondence(),
        dependency_identity,
        observed_rules,
    ))
}

struct ObservationBinding<'a> {
    installed: &'a WorthQueryInstalledAuthorizationPolicy,
    evidence: &'a RelationalAuthorizationObservationEvidence,
    observed_paths: &'a mut [bool],
    policy: &'a str,
}

impl ObservationBinding<'_> {
    fn observe_rule(
        &mut self,
        rule: &worth_runtime_bridge::facade::BridgeAuthorizationRuleContract,
        path_indices: &[usize],
    ) -> Result<BridgeAuthorizationRuleObservation, WorthQueryOperationAuthorizationDenial> {
        let [requirement] = rule.requirements() else {
            return Err(invalid_shape(self.policy));
        };
        if requirement.clauses().len() != path_indices.len() {
            return Err(invalid_shape(self.policy));
        }
        let mut clauses = Vec::with_capacity(path_indices.len());
        for (contract, path_index) in requirement.clauses().iter().zip(path_indices) {
            clauses.push(self.observe_clause(contract, *path_index)?);
        }
        Ok(BridgeAuthorizationRuleObservation::all(
            rule.effect(),
            [BridgeAuthorizationRequirementObservation::any(clauses)],
        ))
    }

    fn observe_clause(
        &mut self,
        contract: &worth_runtime_bridge::facade::BridgeAuthorizationClauseContract,
        path_index: usize,
    ) -> Result<BridgeAuthorizationClauseObservation, WorthQueryOperationAuthorizationDenial> {
        let Some(binding) = self.installed.bridge_path_bindings().get(path_index) else {
            return Err(invalid_shape(self.policy));
        };
        let Some(path) = self.evidence.paths().get(path_index) else {
            return Err(invalid_shape(self.policy));
        };
        let Some(was_observed) = self.observed_paths.get_mut(path_index) else {
            return Err(invalid_shape(self.policy));
        };
        if *was_observed || contract.identity() != &binding.identity {
            return Err(invalid_shape(self.policy));
        }
        *was_observed = true;
        Ok(BridgeAuthorizationClauseObservation::new(
            binding.identity,
            path.matched(),
            path.exhaustive(),
            BridgeAuthorizationDependencyCardinality {
                entities: path.entities().len(),
                relations: path.relations().len(),
                adjacency_lists: path.adjacency_lists().len(),
                fields: path.fields().len(),
            },
        ))
    }
}

fn invalid_shape(policy: &str) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::InvalidInstalledPolicy,
        policy,
    )
}
