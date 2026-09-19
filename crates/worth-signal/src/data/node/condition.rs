mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::aspect::AspectMask;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::temporal::TemporalCondition;
use crate::logic::transaction::{
    AspectMergePolicyBinding, ConflictIsolationPolicyName, ConflictPolicyName, DeletionPolicyName,
    IdentityMatcherName, MergeStrategyName, SourceOnlyPolicyName,
};
use crate::schema::data::SignalSchemaBinding;

use super::contract::NodeContract;

/// Runtime-affine identity for a condition installed by the one graph-lowering
/// owner. Unlike `Custom(String)`, this is not portable semantic text and
/// cannot be reconstructed from a Query family name or reporting digest.
#[derive(Clone)]
pub struct InstalledSignalConditionIdentity {
    graph_instance_id: u64,
    node: crate::data::handle::NodeId,
    role: InstalledSignalConditionRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum InstalledSignalConditionRole {
    Predicate,
    TemporalWake,
}

impl InstalledSignalConditionIdentity {
    pub(crate) const fn new(
        graph_instance_id: u64,
        node: crate::data::handle::NodeId,
        role: InstalledSignalConditionRole,
    ) -> Self {
        Self {
            graph_instance_id,
            node,
            role,
        }
    }

    pub(crate) const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub(crate) const fn role(&self) -> InstalledSignalConditionRole {
        self.role
    }

    pub(crate) fn is_same_installed_identity(&self, candidate: &Self) -> bool {
        self.graph_instance_id == candidate.graph_instance_id
            && self.node == candidate.node
            && self.role == candidate.role
    }
}

impl std::fmt::Debug for InstalledSignalConditionIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InstalledSignalConditionIdentity")
            .finish_non_exhaustive()
    }
}

/// Evaluation condition descriptor for a node.
///
/// This is a policy declaration. Runtime gating integration is tier/runtime
/// specific and intentionally decoupled from node storage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EvaluationCondition {
    /// Always evaluate when dirty.
    #[default]
    Always,
    /// Evaluate only when dirtying touches one of these aspects.
    AspectFilter(AspectMask),
    /// Evaluate only when change magnitude exceeds this threshold.
    DeltaThreshold(f64),
    /// Evaluate only when explicitly requested.
    OnDemand,
    /// Evaluate according to a first-class temporal policy.
    Temporal(TemporalCondition),
    /// Named custom condition handled by embedding runtime.
    Custom(String),
    /// Opaque condition installed by an admitted runtime lowering owner.
    #[serde(skip)]
    Installed(InstalledSignalConditionIdentity),
}

impl PartialEq for EvaluationCondition {
    fn eq(&self, candidate: &Self) -> bool {
        match (self, candidate) {
            (Self::Always, Self::Always) | (Self::OnDemand, Self::OnDemand) => true,
            (Self::AspectFilter(current), Self::AspectFilter(candidate)) => current == candidate,
            (Self::DeltaThreshold(current), Self::DeltaThreshold(candidate)) => {
                current == candidate
            }
            (Self::Temporal(current), Self::Temporal(candidate)) => current == candidate,
            (Self::Custom(current), Self::Custom(candidate)) => current == candidate,
            (Self::Installed(current), Self::Installed(candidate)) => {
                current.is_same_installed_identity(candidate)
            }
            _ => false,
        }
    }
}

/// Per-node evaluation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeEvaluationConfig {
    /// Schema registry binding that owns the node's default semantics.
    #[serde(default)]
    pub schema_binding: Option<SignalSchemaBinding>,
    /// Optional per-node merge strategy override resolved against the runtime registry.
    #[serde(default)]
    pub merge_strategy_name: Option<MergeStrategyName>,
    /// Optional per-node conflict policy override resolved against the runtime registry.
    #[serde(default)]
    pub conflict_policy_name: Option<ConflictPolicyName>,
    /// Optional per-node identity matcher override resolved against the runtime registry.
    #[serde(default)]
    pub identity_matcher_name: Option<IdentityMatcherName>,
    /// Optional per-node source-only merge policy override resolved against the runtime registry.
    #[serde(default)]
    pub source_only_policy_name: Option<SourceOnlyPolicyName>,
    /// Optional per-node deletion policy override resolved against the runtime registry.
    #[serde(default)]
    pub deletion_policy_name: Option<DeletionPolicyName>,
    /// Optional per-node conflict isolation override resolved against the runtime registry.
    #[serde(default)]
    pub conflict_isolation_policy_name: Option<ConflictIsolationPolicyName>,
    /// Optional per-aspect merge policy overrides resolved against the runtime registry.
    #[serde(default)]
    pub aspect_merge_policy_bindings: Vec<AspectMergePolicyBinding>,
    /// Declarative contract for this node's read/write and context behavior.
    #[serde(default)]
    pub contract: NodeContract,
    /// Condition used to gate node evaluation.
    pub condition: EvaluationCondition,
    /// Comparator policy used to decide whether dependency version changes
    /// are meaningful for this node. `None` means inherit from tier policy.
    #[serde(default)]
    pub comparator: Option<VersionComparatorPolicy>,
    /// Producer-owned semantic equivalence policy. This never governs a
    /// consumer dependency edge.
    #[serde(default)]
    pub output_equivalence: OutputEquivalencePolicy,
    /// Whether this node reports partition-aware output changes.
    #[serde(default)]
    pub partitioned_output: bool,
}

impl Default for NodeEvaluationConfig {
    fn default() -> Self {
        Self {
            schema_binding: None,
            merge_strategy_name: None,
            conflict_policy_name: None,
            identity_matcher_name: None,
            source_only_policy_name: None,
            deletion_policy_name: None,
            conflict_isolation_policy_name: None,
            aspect_merge_policy_bindings: Vec::new(),
            contract: NodeContract::default(),
            condition: EvaluationCondition::Always,
            comparator: None,
            output_equivalence: OutputEquivalencePolicy::default(),
            partitioned_output: false,
        }
    }
}

impl NodeEvaluationConfig {
    pub(crate) fn upgrade_legacy_output_equivalence(mut self) -> Self {
        use crate::data::performance::{ComparatorBasis, IdentityBasis, SuppressionBasis};

        let legacy = matches!(
            self.comparator,
            Some(VersionComparatorPolicy::OutputIdentity)
        ) && self.output_equivalence == OutputEquivalencePolicy::ExactAspectVersion
            && self.contract.execution.equivalence.identity_basis == IdentityBasis::OutputIdentity
            && self.contract.execution.equivalence.suppression_basis
                == SuppressionBasis::OutputIdentityAndComparator
            && self.contract.execution.equivalence.comparator_basis
                == ComparatorBasis::OutputIdentity;
        if legacy {
            self.output_equivalence = OutputEquivalencePolicy::OutputIdentity;
        }
        self
    }
}
