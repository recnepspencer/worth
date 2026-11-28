use std::sync::Arc;

use crate::data::graph::SignalGraph;

use super::{
    InstalledSignalAuthorizationPolicy, SignalAuthorizationDecision,
    SignalAuthorizationDecisionEvidence, SignalAuthorizationDenial,
    SignalAuthorizationEvaluationCounters, SignalAuthorizationObservation,
    SignalAuthorizationRuleContract, SignalAuthorizationRuleEffect,
    SignalAuthorizationRuleObservation,
};

impl SignalGraph {
    pub fn evaluate_authorization(
        &self,
        policy: &InstalledSignalAuthorizationPolicy,
        observation: SignalAuthorizationObservation,
    ) -> Result<SignalAuthorizationDecisionEvidence, SignalAuthorizationDenial> {
        validate_observation(self, policy, &observation)?;
        let mut counters = SignalAuthorizationEvaluationCounters::default();
        let mut required_rules_match = true;
        let mut prohibited_rule_matches = false;
        let mut rule_decisions = Vec::with_capacity(observation.rules.len());
        for rule in &observation.rules {
            let rule_matches = evaluate_rule(rule, &mut counters);
            rule_decisions.push(super::SignalAuthorizationRuleDecisionEvidence::new(
                rule.effect,
                rule_matches,
            ));
            match rule.effect {
                SignalAuthorizationRuleEffect::Required => {
                    required_rules_match &= rule_matches;
                    counters.required_rules_matched += usize::from(rule_matches);
                }
                SignalAuthorizationRuleEffect::Prohibited => {
                    prohibited_rule_matches |= rule_matches;
                    counters.prohibited_rules_matched += usize::from(rule_matches);
                }
            }
        }
        let decision = if required_rules_match && !prohibited_rule_matches {
            SignalAuthorizationDecision::Allowed
        } else {
            SignalAuthorizationDecision::Denied
        };
        Ok(SignalAuthorizationDecisionEvidence {
            graph_instance_id: self.runtime_instance_id(),
            policy_identity: policy.identity,
            dependency_identity: observation.dependency_identity,
            decision,
            rule_decisions,
            counters,
            authority: Arc::clone(&policy.authority),
        })
    }
}

fn validate_observation(
    graph: &SignalGraph,
    policy: &InstalledSignalAuthorizationPolicy,
    observation: &SignalAuthorizationObservation,
) -> Result<(), SignalAuthorizationDenial> {
    if policy.graph_instance_id != graph.runtime_instance_id()
        || !graph
            .authorization_policy_identities
            .contains(policy.identity.bytes())
    {
        return Err(SignalAuthorizationDenial::StalePolicy);
    }
    if !same_shape(&policy.rules, &observation.rules) {
        return Err(SignalAuthorizationDenial::ObservationShapeMismatch);
    }
    for clause in observation
        .rules
        .iter()
        .flat_map(|rule| &rule.requirements)
        .flat_map(|requirement| &requirement.clauses)
    {
        if clause.graph_instance_id != policy.graph_instance_id
            || clause.policy_identity != policy.identity
        {
            return Err(SignalAuthorizationDenial::ObservationShapeMismatch);
        }
        if !clause.exhaustive {
            return Err(SignalAuthorizationDenial::NonExhaustiveObservation);
        }
    }
    Ok(())
}

fn same_shape(
    contracts: &[SignalAuthorizationRuleContract],
    observations: &[SignalAuthorizationRuleObservation],
) -> bool {
    contracts.len() == observations.len()
        && contracts
            .iter()
            .zip(observations)
            .all(|(contract, observed)| {
                contract.effect == observed.effect
                    && contract.requirements.len() == observed.requirements.len()
                    && contract
                        .requirements
                        .iter()
                        .zip(&observed.requirements)
                        .all(|(required, actual)| required.clauses.len() == actual.clauses.len())
            })
}

fn evaluate_rule(
    rule: &SignalAuthorizationRuleObservation,
    counters: &mut SignalAuthorizationEvaluationCounters,
) -> bool {
    counters.rules_evaluated += 1;
    let mut rule_matches = true;
    for requirement in &rule.requirements {
        counters.requirements_evaluated += 1;
        let requirement_matches = evaluate_requirement(requirement, counters);
        counters.requirements_matched += usize::from(requirement_matches);
        rule_matches &= requirement_matches;
    }
    rule_matches
}

fn evaluate_requirement(
    requirement: &super::SignalAuthorizationRequirementObservation,
    counters: &mut SignalAuthorizationEvaluationCounters,
) -> bool {
    let mut requirement_matches = false;
    for clause in &requirement.clauses {
        counters.clauses_evaluated += 1;
        counters.entities_depended_on += clause.dependencies.entities;
        counters.relations_depended_on += clause.dependencies.relations;
        counters.adjacency_lists_depended_on += clause.dependencies.adjacency_lists;
        counters.fields_depended_on += clause.dependencies.fields;
        counters.clauses_matched += usize::from(clause.matched);
        requirement_matches |= clause.matched;
    }
    requirement_matches
}
