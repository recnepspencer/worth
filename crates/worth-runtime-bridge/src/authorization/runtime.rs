use std::collections::BTreeMap;
use std::sync::Arc;

use worth_signal::facade::{
    InstalledSignalAuthorizationPolicy, SignalAuthorizationClauseContract,
    SignalAuthorizationClauseObservation, SignalAuthorizationDependencyCardinality,
    SignalAuthorizationObservation, SignalAuthorizationRequirementContract,
    SignalAuthorizationRequirementObservation, SignalAuthorizationRuleContract,
    SignalAuthorizationRuleEffect, SignalAuthorizationRuleObservation, SignalGraph,
};

use super::evidence::BridgeAuthorizationCorrespondenceAuthority;
use super::{
    BridgeAuthorizationCorrespondenceIdentity, BridgeAuthorizationDecisionEvidence,
    BridgeAuthorizationDenial, BridgeAuthorizationDenialKind,
    BridgeAuthorizationInstallationRequest, BridgeAuthorizationObservation,
    BridgeAuthorizationRuleContract, BridgeAuthorizationRuleEffect,
};

mod retention;
pub use retention::BridgeAuthorizationRetentionStop;

pub(super) struct BridgeInstalledAuthorizationCorrespondence {
    identity: BridgeAuthorizationCorrespondenceIdentity,
    binding_identity: super::BridgeAuthorizationBindingIdentity,
    ability: String,
    scope_entity: String,
    policy: String,
    rules: Vec<BridgeAuthorizationRuleContract>,
    signal_policy: InstalledSignalAuthorizationPolicy,
    authority: Arc<BridgeAuthorizationCorrespondenceAuthority>,
}

impl BridgeInstalledAuthorizationCorrespondence {
    pub(super) fn from_installation(
        request: BridgeAuthorizationInstallationRequest,
        signal_policy: InstalledSignalAuthorizationPolicy,
    ) -> Self {
        Self {
            identity: request.correspondence,
            binding_identity: request.binding_identity,
            ability: request.ability,
            scope_entity: request.scope_entity,
            policy: request.policy,
            rules: request.rules,
            signal_policy,
            authority: Arc::new(BridgeAuthorizationCorrespondenceAuthority { _seal: () }),
        }
    }
}

pub struct BridgeAuthorizationRuntime {
    pub(super) graph: SignalGraph,
    pub(super) correspondences: BTreeMap<
        BridgeAuthorizationCorrespondenceIdentity,
        BridgeInstalledAuthorizationCorrespondence,
    >,
}

impl BridgeAuthorizationRuntime {
    /// Installed correspondences bound the two fixed-identity ordered reads
    /// performed during one authorization evaluation.
    pub fn correspondence_count(&self) -> usize {
        self.correspondences.len()
    }

    pub fn new() -> Self {
        Self {
            graph: SignalGraph::new(),
            correspondences: BTreeMap::new(),
        }
    }

    pub fn matches_installed_policy<'rule>(
        &self,
        correspondence: BridgeAuthorizationCorrespondenceIdentity,
        binding_identity: &super::BridgeAuthorizationBindingIdentity,
        ability: &str,
        scope_entity: &str,
        policy: &str,
        rules: impl IntoIterator<Item = &'rule BridgeAuthorizationRuleContract>,
    ) -> bool {
        self.correspondences
            .get(&correspondence)
            .is_some_and(|installed| {
                installed.identity == correspondence
                    && &installed.binding_identity == binding_identity
                    && installed.ability == ability
                    && installed.scope_entity == scope_entity
                    && installed.policy == policy
                    && installed.rules.iter().eq(rules)
            })
    }

    pub fn evaluate(
        &self,
        observation: BridgeAuthorizationObservation,
    ) -> Result<BridgeAuthorizationDecisionEvidence, BridgeAuthorizationDenial> {
        let installed = self
            .correspondences
            .get(&observation.correspondence)
            .ok_or_else(|| {
                denial(
                    BridgeAuthorizationDenialKind::UnknownCorrespondence,
                    "unknown installed authorization correspondence",
                )
            })?;
        if !same_shape(&installed.rules, &observation.rules) {
            return Err(denial(
                BridgeAuthorizationDenialKind::ObservationShapeMismatch,
                &installed.policy,
            ));
        }
        let signal_observation = lower_signal_observation(&observation);
        let signal = self
            .graph
            .evaluate_authorization(&installed.signal_policy, signal_observation)
            .map_err(|_| {
                denial(
                    BridgeAuthorizationDenialKind::SignalEvaluationRejected,
                    &installed.policy,
                )
            })?;
        let rule_decisions = retain_rule_decisions(installed, &signal)?;
        Ok(BridgeAuthorizationDecisionEvidence::mint(
            installed.identity,
            observation.dependency_identity,
            signal,
            rule_decisions,
            Arc::clone(&installed.authority),
        ))
    }
}

fn lower_signal_observation(
    observation: &BridgeAuthorizationObservation,
) -> SignalAuthorizationObservation {
    SignalAuthorizationObservation::new(
        observation.dependency_identity,
        observation.rules.iter().map(|rule| {
            SignalAuthorizationRuleObservation::all(
                lower_effect(rule.effect),
                rule.requirements.iter().map(|requirement| {
                    SignalAuthorizationRequirementObservation::any(requirement.clauses.iter().map(
                        |clause| {
                            let dependencies = clause.dependencies();
                            SignalAuthorizationClauseObservation::new(
                                clause.matched(),
                                clause.exhaustive(),
                                SignalAuthorizationDependencyCardinality {
                                    entities: dependencies.entities,
                                    relations: dependencies.relations,
                                    adjacency_lists: dependencies.adjacency_lists,
                                    fields: dependencies.fields,
                                },
                            )
                        },
                    ))
                }),
            )
        }),
    )
}

fn retain_rule_decisions(
    installed: &BridgeInstalledAuthorizationCorrespondence,
    signal: &worth_signal::facade::SignalAuthorizationDecisionEvidence,
) -> Result<Vec<super::BridgeAuthorizationRuleDecisionEvidence>, BridgeAuthorizationDenial> {
    let mut decisions = Vec::with_capacity(installed.rules.len());
    for (rule, decision) in installed.rules.iter().zip(signal.rule_decisions()) {
        if lower_effect(rule.effect()) != decision.effect() {
            return Err(denial(
                BridgeAuthorizationDenialKind::SignalEvaluationRejected,
                &installed.policy,
            ));
        }
        decisions.push(super::BridgeAuthorizationRuleDecisionEvidence::new(
            rule.effect(),
            decision.matched(),
        ));
    }
    Ok(decisions)
}

impl Default for BridgeAuthorizationRuntime {
    fn default() -> Self {
        Self::new()
    }
}

fn same_shape(
    contracts: &[BridgeAuthorizationRuleContract],
    observations: &[super::BridgeAuthorizationRuleObservation],
) -> bool {
    contracts.len() == observations.len()
        && contracts
            .iter()
            .zip(observations)
            .all(|(contract, observed)| {
                contract.effect() == observed.effect
                    && contract.requirements().len() == observed.requirements.len()
                    && contract
                        .requirements()
                        .iter()
                        .zip(&observed.requirements)
                        .all(|(required, actual)| {
                            required.clauses().len() == actual.clauses.len()
                                && required.clauses().iter().zip(&actual.clauses).all(
                                    |(expected, clause)| expected.identity() == clause.identity(),
                                )
                        })
            })
}

pub(super) fn lower_rule_contract(
    rule: &BridgeAuthorizationRuleContract,
) -> SignalAuthorizationRuleContract {
    SignalAuthorizationRuleContract::all(
        lower_effect(rule.effect()),
        rule.requirements().iter().map(|requirement| {
            SignalAuthorizationRequirementContract::any(
                requirement
                    .clauses()
                    .iter()
                    .map(|_| SignalAuthorizationClauseContract::new()),
            )
        }),
    )
}

const fn lower_effect(effect: BridgeAuthorizationRuleEffect) -> SignalAuthorizationRuleEffect {
    match effect {
        BridgeAuthorizationRuleEffect::Required => SignalAuthorizationRuleEffect::Required,
        BridgeAuthorizationRuleEffect::Prohibited => SignalAuthorizationRuleEffect::Prohibited,
    }
}

fn denial(
    kind: BridgeAuthorizationDenialKind,
    subject: impl Into<String>,
) -> BridgeAuthorizationDenial {
    BridgeAuthorizationDenial::new(kind, subject)
}
