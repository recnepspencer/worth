use std::sync::Arc;

mod evaluation;
mod installation;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SignalAuthorizationPolicyIdentity([u8; 32]);

impl SignalAuthorizationPolicyIdentity {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalAuthorizationRuleEffect {
    Required,
    Prohibited,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationClauseContract;

impl SignalAuthorizationClauseContract {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for SignalAuthorizationClauseContract {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationRequirementContract {
    clauses: Vec<SignalAuthorizationClauseContract>,
}

impl SignalAuthorizationRequirementContract {
    pub fn any(clauses: impl IntoIterator<Item = SignalAuthorizationClauseContract>) -> Self {
        Self {
            clauses: clauses.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationRuleContract {
    effect: SignalAuthorizationRuleEffect,
    requirements: Vec<SignalAuthorizationRequirementContract>,
}

impl SignalAuthorizationRuleContract {
    pub fn all(
        effect: SignalAuthorizationRuleEffect,
        requirements: impl IntoIterator<Item = SignalAuthorizationRequirementContract>,
    ) -> Self {
        Self {
            effect,
            requirements: requirements.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationPolicyDefinition {
    identity: SignalAuthorizationPolicyIdentity,
    rules: Vec<SignalAuthorizationRuleContract>,
}

impl SignalAuthorizationPolicyDefinition {
    pub fn new(
        identity: SignalAuthorizationPolicyIdentity,
        rules: impl IntoIterator<Item = SignalAuthorizationRuleContract>,
    ) -> Self {
        Self {
            identity,
            rules: rules.into_iter().collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SignalAuthorizationDependencyCardinality {
    pub entities: usize,
    pub relations: usize,
    pub adjacency_lists: usize,
    pub fields: usize,
}

/// A clause observation admitted under an installed Signal policy.
///
/// ```compile_fail,E0624
/// use worth_signal::facade::{
///     SignalAuthorizationClauseObservation, SignalAuthorizationDependencyCardinality,
/// };
/// use worth_signal::facade::InstalledSignalAuthorizationPolicy;
/// fn forge(policy: &InstalledSignalAuthorizationPolicy) {
///     let _ = SignalAuthorizationClauseObservation::new(
///         policy, true, true, SignalAuthorizationDependencyCardinality::default(),
///     );
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationClauseObservation {
    graph_instance_id: u64,
    policy_identity: SignalAuthorizationPolicyIdentity,
    matched: bool,
    exhaustive: bool,
    dependencies: SignalAuthorizationDependencyCardinality,
}

impl SignalAuthorizationClauseObservation {
    pub(crate) const fn new(
        policy: &InstalledSignalAuthorizationPolicy,
        matched: bool,
        exhaustive: bool,
        dependencies: SignalAuthorizationDependencyCardinality,
    ) -> Self {
        Self {
            graph_instance_id: policy.graph_instance_id,
            policy_identity: policy.identity,
            matched,
            exhaustive,
            dependencies,
        }
    }
}

impl InstalledSignalAuthorizationPolicy {
    /// Lowers an observed clause under this exact installed policy's custody.
    pub const fn admit_clause_observation(
        &self,
        matched: bool,
        exhaustive: bool,
        dependencies: SignalAuthorizationDependencyCardinality,
    ) -> SignalAuthorizationClauseObservation {
        SignalAuthorizationClauseObservation::new(self, matched, exhaustive, dependencies)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationRequirementObservation {
    clauses: Vec<SignalAuthorizationClauseObservation>,
}

impl SignalAuthorizationRequirementObservation {
    pub fn any(clauses: impl IntoIterator<Item = SignalAuthorizationClauseObservation>) -> Self {
        Self {
            clauses: clauses.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationRuleObservation {
    effect: SignalAuthorizationRuleEffect,
    requirements: Vec<SignalAuthorizationRequirementObservation>,
}

impl SignalAuthorizationRuleObservation {
    pub fn all(
        effect: SignalAuthorizationRuleEffect,
        requirements: impl IntoIterator<Item = SignalAuthorizationRequirementObservation>,
    ) -> Self {
        Self {
            effect,
            requirements: requirements.into_iter().collect(),
        }
    }
}

pub struct SignalAuthorizationObservation {
    dependency_identity: [u8; 32],
    rules: Vec<SignalAuthorizationRuleObservation>,
}

impl SignalAuthorizationObservation {
    pub fn new(
        dependency_identity: [u8; 32],
        rules: impl IntoIterator<Item = SignalAuthorizationRuleObservation>,
    ) -> Self {
        Self {
            dependency_identity,
            rules: rules.into_iter().collect(),
        }
    }
}

struct SignalAuthorizationAuthority {
    _seal: (),
}

pub struct InstalledSignalAuthorizationPolicy {
    graph_instance_id: u64,
    identity: SignalAuthorizationPolicyIdentity,
    rules: Vec<SignalAuthorizationRuleContract>,
    authority: Arc<SignalAuthorizationAuthority>,
}

impl InstalledSignalAuthorizationPolicy {
    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub const fn identity(&self) -> SignalAuthorizationPolicyIdentity {
        self.identity
    }

    pub fn retains(&self, evidence: &SignalAuthorizationDecisionEvidence) -> bool {
        self.graph_instance_id == evidence.graph_instance_id
            && self.identity == evidence.policy_identity
            && Arc::ptr_eq(&self.authority, &evidence.authority)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalAuthorizationDecision {
    Allowed,
    Denied,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SignalAuthorizationEvaluationCounters {
    pub rules_evaluated: usize,
    pub required_rules_matched: usize,
    pub prohibited_rules_matched: usize,
    pub requirements_evaluated: usize,
    pub requirements_matched: usize,
    pub clauses_evaluated: usize,
    pub clauses_matched: usize,
    pub entities_depended_on: usize,
    pub relations_depended_on: usize,
    pub adjacency_lists_depended_on: usize,
    pub fields_depended_on: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalAuthorizationRuleDecisionEvidence {
    effect: SignalAuthorizationRuleEffect,
    matched: bool,
}

impl SignalAuthorizationRuleDecisionEvidence {
    pub(super) const fn new(effect: SignalAuthorizationRuleEffect, matched: bool) -> Self {
        Self { effect, matched }
    }

    pub const fn effect(self) -> SignalAuthorizationRuleEffect {
        self.effect
    }

    pub const fn matched(self) -> bool {
        self.matched
    }
}

pub struct SignalAuthorizationDecisionEvidence {
    graph_instance_id: u64,
    policy_identity: SignalAuthorizationPolicyIdentity,
    dependency_identity: [u8; 32],
    decision: SignalAuthorizationDecision,
    rule_decisions: Vec<SignalAuthorizationRuleDecisionEvidence>,
    counters: SignalAuthorizationEvaluationCounters,
    authority: Arc<SignalAuthorizationAuthority>,
}

impl SignalAuthorizationDecisionEvidence {
    pub const fn policy_identity(&self) -> SignalAuthorizationPolicyIdentity {
        self.policy_identity
    }

    pub const fn dependency_identity(&self) -> &[u8; 32] {
        &self.dependency_identity
    }

    pub const fn decision(&self) -> SignalAuthorizationDecision {
        self.decision
    }

    pub fn rule_decisions(&self) -> &[SignalAuthorizationRuleDecisionEvidence] {
        &self.rule_decisions
    }

    pub const fn counters(&self) -> SignalAuthorizationEvaluationCounters {
        self.counters
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalAuthorizationDenial {
    ForeignGraph,
    EmptyPolicy,
    MissingRequiredRule,
    EmptyRule,
    EmptyRequirement,
    DuplicatePolicy,
    StalePolicy,
    ObservationShapeMismatch,
    NonExhaustiveObservation,
}

#[cfg(test)]
mod tests;
