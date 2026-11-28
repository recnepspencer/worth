use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::authorization::{
    RelationalAuthorizationObservationEvidence, RelationalAuthorizationObservationIdentity,
    RelationalAuthorizationPathObservation, RelationalAuthorizationRecordedActorObservation,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeAuthorizationBindingIdentity {
    runtime_ordinal: u64,
    generation: u64,
    package_identity: CanonicalDigestId,
    schema_identity: CanonicalDigestId,
}

impl BridgeAuthorizationBindingIdentity {
    pub const fn new(
        runtime_ordinal: u64,
        generation: u64,
        package_identity: CanonicalDigestId,
        schema_identity: CanonicalDigestId,
    ) -> Self {
        Self {
            runtime_ordinal,
            generation,
            package_identity,
            schema_identity,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BridgeAuthorizationCorrespondenceIdentity(pub(crate) [u8; 32]);

impl BridgeAuthorizationCorrespondenceIdentity {
    pub const fn from_installed_policy(identity: &CanonicalDigestId) -> Self {
        Self(*identity.bytes())
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeAuthorizationRuleEffect {
    Required,
    Prohibited,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeAuthorizationClauseContract {
    identity: [u8; 32],
}

impl BridgeAuthorizationClauseContract {
    pub const fn new(identity: [u8; 32]) -> Self {
        Self { identity }
    }

    pub const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeAuthorizationRequirementContract {
    clauses: Vec<BridgeAuthorizationClauseContract>,
}

impl BridgeAuthorizationRequirementContract {
    pub fn any(clauses: impl IntoIterator<Item = BridgeAuthorizationClauseContract>) -> Self {
        Self {
            clauses: clauses.into_iter().collect(),
        }
    }

    pub fn clauses(&self) -> &[BridgeAuthorizationClauseContract] {
        &self.clauses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeAuthorizationRuleContract {
    effect: BridgeAuthorizationRuleEffect,
    requirements: Vec<BridgeAuthorizationRequirementContract>,
}

impl BridgeAuthorizationRuleContract {
    pub fn all(
        effect: BridgeAuthorizationRuleEffect,
        requirements: impl IntoIterator<Item = BridgeAuthorizationRequirementContract>,
    ) -> Self {
        Self {
            effect,
            requirements: requirements.into_iter().collect(),
        }
    }

    pub const fn effect(&self) -> BridgeAuthorizationRuleEffect {
        self.effect
    }

    pub fn requirements(&self) -> &[BridgeAuthorizationRequirementContract] {
        &self.requirements
    }
}

pub struct BridgeAuthorizationInstallationRequest {
    pub(crate) correspondence: BridgeAuthorizationCorrespondenceIdentity,
    pub(crate) binding_identity: BridgeAuthorizationBindingIdentity,
    pub(crate) ability: String,
    pub(crate) scope_entity: String,
    pub(crate) policy: String,
    pub(crate) rules: Vec<BridgeAuthorizationRuleContract>,
}

impl BridgeAuthorizationInstallationRequest {
    pub fn new(
        installed_policy_identity: &CanonicalDigestId,
        binding_identity: BridgeAuthorizationBindingIdentity,
        ability: impl Into<String>,
        scope_entity: impl Into<String>,
        policy: impl Into<String>,
        rules: impl IntoIterator<Item = BridgeAuthorizationRuleContract>,
    ) -> Self {
        Self {
            correspondence: BridgeAuthorizationCorrespondenceIdentity::from_installed_policy(
                installed_policy_identity,
            ),
            binding_identity,
            ability: ability.into(),
            scope_entity: scope_entity.into(),
            policy: policy.into(),
            rules: rules.into_iter().collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeAuthorizationDependencyCardinality {
    pub entities: usize,
    pub relations: usize,
    pub adjacency_lists: usize,
    pub fields: usize,
}

/// Authorization input derived from a Relational observation.
///
/// ```compile_fail,E0624
/// use worth_runtime_bridge::facade::{
///     BridgeAuthorizationClauseObservation, BridgeAuthorizationDependencyCardinality,
/// };
/// let _ = BridgeAuthorizationClauseObservation::new(
///     [1; 32], true, true,
///     BridgeAuthorizationDependencyCardinality { entities: 0, relations: 0, adjacency_lists: 0, fields: 0 },
/// );
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeAuthorizationClauseObservation {
    identity: [u8; 32],
    source: Option<RelationalAuthorizationObservationIdentity>,
    matched: bool,
    exhaustive: bool,
    dependencies: BridgeAuthorizationDependencyCardinality,
}

impl BridgeAuthorizationClauseObservation {
    pub(crate) const fn new(
        identity: [u8; 32],
        matched: bool,
        exhaustive: bool,
        dependencies: BridgeAuthorizationDependencyCardinality,
    ) -> Self {
        Self {
            identity,
            source: None,
            matched,
            exhaustive,
            dependencies,
        }
    }

    pub fn from_relational_path(
        identity: [u8; 32],
        path: &RelationalAuthorizationPathObservation,
    ) -> Self {
        Self::from_guarded_relational_path(identity, path, true)
    }

    /// A guard may narrow the underlying match.
    pub fn from_guarded_relational_path(
        identity: [u8; 32],
        path: &RelationalAuthorizationPathObservation,
        guard_admitted: bool,
    ) -> Self {
        let mut observation = Self::new(
            identity,
            path.matched() && guard_admitted,
            path.exhaustive(),
            BridgeAuthorizationDependencyCardinality {
                entities: path.entities().len(),
                relations: path.relations().len(),
                adjacency_lists: path.adjacency_lists().len(),
                fields: path.fields().len(),
            },
        );
        observation.source = path.observation_identity();
        observation
    }

    /// Evaluates a required recorded-actor clause using a live actor read
    /// from the exact snapshot and principal of the policy observation.
    pub fn from_recorded_actor(
        identity: [u8; 32],
        evidence: &RelationalAuthorizationObservationEvidence,
        recorded: Option<&RelationalAuthorizationRecordedActorObservation>,
        comparison: BridgeRecordedActorComparison,
    ) -> Self {
        let recorded = recorded.filter(|actor| actor.snapshot() == evidence.snapshot());
        let matched = recorded
            .and_then(|recorded| recorded.actor())
            .is_some_and(|actor| match comparison {
                BridgeRecordedActorComparison::SamePrincipal => actor == evidence.principal(),
                BridgeRecordedActorComparison::DifferentPrincipal => actor != evidence.principal(),
            });
        let mut observation = Self::new(
            identity,
            matched,
            true,
            BridgeAuthorizationDependencyCardinality {
                entities: usize::from(recorded.is_some()),
                relations: 0,
                adjacency_lists: 0,
                fields: if recorded.is_some() { 3 } else { 0 },
            },
        );
        observation.source = Some(evidence.observation_identity());
        observation
    }

    pub(crate) const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    pub(crate) const fn matched(&self) -> bool {
        self.matched
    }

    pub(crate) const fn exhaustive(&self) -> bool {
        self.exhaustive
    }

    pub(crate) const fn dependencies(&self) -> BridgeAuthorizationDependencyCardinality {
        self.dependencies
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeRecordedActorComparison {
    SamePrincipal,
    DifferentPrincipal,
}

pub struct BridgeAuthorizationRequirementObservation {
    pub(crate) clauses: Vec<BridgeAuthorizationClauseObservation>,
}

impl BridgeAuthorizationRequirementObservation {
    pub fn any(clauses: impl IntoIterator<Item = BridgeAuthorizationClauseObservation>) -> Self {
        Self {
            clauses: clauses.into_iter().collect(),
        }
    }
}

pub struct BridgeAuthorizationRuleObservation {
    pub(crate) effect: BridgeAuthorizationRuleEffect,
    pub(crate) requirements: Vec<BridgeAuthorizationRequirementObservation>,
}

impl BridgeAuthorizationRuleObservation {
    pub fn all(
        effect: BridgeAuthorizationRuleEffect,
        requirements: impl IntoIterator<Item = BridgeAuthorizationRequirementObservation>,
    ) -> Self {
        Self {
            effect,
            requirements: requirements.into_iter().collect(),
        }
    }
}

pub struct BridgeAuthorizationObservation {
    pub(crate) correspondence: BridgeAuthorizationCorrespondenceIdentity,
    pub(crate) dependency_identity: [u8; 32],
    pub(crate) rules: Vec<BridgeAuthorizationRuleObservation>,
    pub(crate) source_matches: bool,
}

impl BridgeAuthorizationObservation {
    pub fn new(
        correspondence: BridgeAuthorizationCorrespondenceIdentity,
        evidence: &RelationalAuthorizationObservationEvidence,
        rules: impl IntoIterator<Item = BridgeAuthorizationRuleObservation>,
    ) -> Self {
        let identity = evidence.observation_identity();
        let rules: Vec<_> = rules.into_iter().collect();
        let source_matches = rules
            .iter()
            .flat_map(|rule| &rule.requirements)
            .flat_map(|requirement| &requirement.clauses)
            .all(|clause| clause.source == Some(identity));
        Self {
            correspondence,
            dependency_identity: *identity.bytes(),
            rules,
            source_matches,
        }
    }
}
