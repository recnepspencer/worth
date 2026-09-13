mod installation;
mod lowering;
mod retained_charge;
static NEXT_CONDITIONAL_CONTRACT_OCCURRENCE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);
use crate::data::aspect::{AspectMask, InstalledSignalNodeCapability, SignalAspectLoweringOwner};
use crate::data::comparator::{
    InstalledSignalComparatorIdentity, InstalledSignalComparatorRole, VersionComparatorPolicy,
};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{EvaluationCondition, InstalledSignalConditionIdentity};
use crate::data::output_equivalence::OutputEquivalencePolicy;
use lowering::{
    install_node_evaluation_config, installed_comparator_identity,
    installed_output_equivalence_identity, lower_artifact_reuse, lower_comparator, lower_condition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalConditionalCondition {
    Always,
    AspectFilter(AspectMask),
    DeltaThreshold(SignalDeltaThresholdContract),
    OnDemand,
    RuntimePredicate,
    TemporalWake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalThresholdValueFamily {
    Integer,
    Float32,
    Float64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalThresholdComparisonDomain {
    AbsoluteDifference,
    RelativeRatio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalThresholdBoundary {
    Inclusive,
    Exclusive,
}

/// Signal-owned, lossless threshold meaning installed by the lowering owner.
/// The runtime predicate identity remains opaque, while this contract retains
/// the typed semantic parameters that identity is authorized to evaluate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDeltaThresholdContract {
    threshold: worth_foundational::facade::AspectValue,
    unit_identity: String,
    value_family: SignalThresholdValueFamily,
    comparison_domain: SignalThresholdComparisonDomain,
    boundary: SignalThresholdBoundary,
}

impl SignalDeltaThresholdContract {
    pub fn new(
        threshold: worth_foundational::facade::AspectValue,
        unit_identity: impl Into<String>,
        value_family: SignalThresholdValueFamily,
        comparison_domain: SignalThresholdComparisonDomain,
        boundary: SignalThresholdBoundary,
    ) -> Self {
        Self {
            threshold,
            unit_identity: unit_identity.into(),
            value_family,
            comparison_domain,
            boundary,
        }
    }

    pub fn threshold(&self) -> &worth_foundational::facade::AspectValue {
        &self.threshold
    }

    pub fn unit_identity(&self) -> &str {
        &self.unit_identity
    }

    pub const fn value_family(&self) -> SignalThresholdValueFamily {
        self.value_family
    }

    pub const fn comparison_domain(&self) -> SignalThresholdComparisonDomain {
        self.comparison_domain
    }

    pub const fn boundary(&self) -> SignalThresholdBoundary {
        self.boundary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalConditionalVersionComparator {
    Exact,
    Tolerance(u64),
    OutputIdentity,
    RuntimeResolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalConditionalArtifactReuse {
    NotReusable,
    DependencyAndOutputEquivalent,
    OutputEquivalent,
    RuntimeResolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalConditionalContractDefinition {
    pub condition: SignalConditionalCondition,
    pub dependency_aspects: AspectMask,
    pub trigger_aspects: AspectMask,
    pub dependency_comparator: SignalConditionalVersionComparator,
    pub output_comparator: SignalConditionalVersionComparator,
    pub artifact_reuse: SignalConditionalArtifactReuse,
}

#[derive(Debug, Clone)]
pub enum SignalConditionalArtifactReusePolicy {
    NotReusable,
    DependencyAndOutputEquivalent,
    OutputEquivalent,
    Installed(InstalledSignalComparatorIdentity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalConditionalContractDenial {
    ForeignGraph,
    ForeignLoweringOwner,
    StaleNode,
    GenerationExhausted,
    OccurrenceExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstalledSignalComparatorUse {
    DependencyVersion,
    OutputEquivalence,
    ArtifactReuse,
}

pub(super) struct InstalledSignalConditionalAuthority {
    _owner_seal: (),
}

pub(crate) struct PreparedSignalConditionalContract {
    node: NodeId,
    predecessor_generation: u64,
    predecessor_occurrence: u64,
    config: crate::data::node::NodeEvaluationConfig,
    contract: InstalledSignalConditionalContract,
}

impl PreparedSignalConditionalContract {
    pub(crate) const fn predecessor_generation(&self) -> u64 {
        self.predecessor_generation
    }

    pub(crate) const fn predecessor_occurrence(&self) -> u64 {
        self.predecessor_occurrence
    }

    pub(crate) fn contract(&self) -> &InstalledSignalConditionalContract {
        &self.contract
    }
}

/// Opaque installed contract. Construction requires both the exact graph-local
/// node capability and the graph's admitted lowering owner.
#[derive(Clone)]
pub struct InstalledSignalConditionalContract {
    pub(super) authority: std::sync::Arc<InstalledSignalConditionalAuthority>,
    graph_instance_id: u64,
    node: NodeId,
    generation: u64,
    occurrence: u64,
    condition: EvaluationCondition,
    semantic_condition: SignalConditionalCondition,
    dependency_aspects: AspectMask,
    trigger_aspects: AspectMask,
    dependency_comparator: VersionComparatorPolicy,
    output_comparator: VersionComparatorPolicy,
    output_equivalence: OutputEquivalencePolicy,
    artifact_reuse: SignalConditionalArtifactReusePolicy,
    // Projection only: prepared once by installation, never admission authority.
    projection_contract: String,
    service_retained_bytes: usize,
}

#[derive(Clone)]
pub(crate) struct SignalConditionalServiceContractBinding {
    authority: std::sync::Arc<InstalledSignalConditionalAuthority>,
    graph_instance_id: u64,
    node: NodeId,
    generation: u64,
    occurrence: u64,
}

impl InstalledSignalConditionalContract {
    pub(super) fn projection_contract(&self) -> &str {
        &self.projection_contract
    }

    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub const fn node(&self) -> NodeId {
        self.node
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    pub fn condition(&self) -> &EvaluationCondition {
        &self.condition
    }

    pub fn semantic_condition(&self) -> &SignalConditionalCondition {
        &self.semantic_condition
    }

    pub fn dependency_comparator(&self) -> &VersionComparatorPolicy {
        &self.dependency_comparator
    }

    pub const fn dependency_aspects(&self) -> AspectMask {
        self.dependency_aspects
    }

    pub const fn trigger_aspects(&self) -> AspectMask {
        self.trigger_aspects
    }

    pub fn output_comparator(&self) -> &VersionComparatorPolicy {
        &self.output_comparator
    }

    pub fn output_equivalence(&self) -> &OutputEquivalencePolicy {
        &self.output_equivalence
    }

    pub fn artifact_reuse(&self) -> &SignalConditionalArtifactReusePolicy {
        &self.artifact_reuse
    }

    pub fn accepts_condition_identity(&self, candidate: &InstalledSignalConditionIdentity) -> bool {
        matches!(
            &self.condition,
            EvaluationCondition::Installed(installed)
                if installed.is_same_installed_identity(candidate)
        )
    }

    pub fn classify_comparator_identity(
        &self,
        candidate: &InstalledSignalComparatorIdentity,
    ) -> Option<InstalledSignalComparatorUse> {
        if installed_comparator_identity(&self.dependency_comparator)
            .is_some_and(|installed| installed.is_same_installed_identity(candidate))
        {
            return Some(InstalledSignalComparatorUse::DependencyVersion);
        }
        if installed_output_equivalence_identity(&self.output_equivalence)
            .is_some_and(|installed| installed.is_same_installed_identity(candidate))
        {
            return Some(InstalledSignalComparatorUse::OutputEquivalence);
        }
        if matches!(
            &self.artifact_reuse,
            SignalConditionalArtifactReusePolicy::Installed(installed)
                if installed.is_same_installed_identity(candidate)
        ) {
            return Some(InstalledSignalComparatorUse::ArtifactReuse);
        }
        None
    }

    pub fn retains_decision(&self, evidence: &super::SignalConditionalDecisionEvidence) -> bool {
        std::sync::Arc::ptr_eq(&self.authority, &evidence.contract_authority)
    }

    pub(crate) fn bind_for_conditional_service(
        &self,
        graph: &SignalGraph,
    ) -> Option<SignalConditionalServiceContractBinding> {
        (self.graph_instance_id == graph.runtime_instance_id()
            && graph.get_entry(self.node).is_ok_and(|entry| {
                entry.conditional_contract_generation() == self.generation
                    && entry.conditional_contract_occurrence() == self.occurrence
            }))
        .then(|| SignalConditionalServiceContractBinding {
            authority: std::sync::Arc::clone(&self.authority),
            graph_instance_id: self.graph_instance_id,
            node: self.node,
            generation: self.generation,
            occurrence: self.occurrence,
        })
    }
}

impl SignalConditionalServiceContractBinding {
    pub(crate) fn is_current_for(&self, graph: &SignalGraph) -> bool {
        self.graph_instance_id == graph.runtime_instance_id()
            && graph.get_entry(self.node).is_ok_and(|entry| {
                entry.conditional_contract_generation() == self.generation
                    && entry.conditional_contract_occurrence() == self.occurrence
            })
    }

    pub(crate) fn matches(&self, contract: &InstalledSignalConditionalContract) -> bool {
        self.graph_instance_id == contract.graph_instance_id
            && self.node == contract.node
            && self.generation == contract.generation
            && self.occurrence == contract.occurrence
            && std::sync::Arc::ptr_eq(&self.authority, &contract.authority)
    }
}
