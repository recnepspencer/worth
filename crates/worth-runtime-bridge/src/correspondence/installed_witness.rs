use std::sync::Arc;

use worth_proof::{
    AssumptionBasis, AuthorityMarker, AuthorityWitness, CanonicalOrder, CanonicalVec,
    CurrentValidity, ExecutionReadyRecipe, FreshnessScopedBasis, LoweredRecipeDxExt, NonEmpty,
    Proof, Recipe, Resolved, ResolvedRecipeDxExt, StructuralProofAuthority, UniqueVec, Uniqueness,
    Unresolved, UnresolvedRecipeDxExt,
};
use worth_signal::facade::{
    Aspect, InstalledSignalAspectSetCapability, InstalledSignalGraphCapability, NodeId,
    PartitionToken,
};

use super::{
    BridgeCorrespondenceDenialKind, BridgeSemanticDependencyCandidate,
    CorrespondenceAdmissionCounters,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InstalledCorrespondenceTarget {
    pub(crate) mapping_identity: Arc<str>,
    pub(crate) signal_graph_instance_id: u64,
    pub(crate) partition: PartitionToken,
    pub(crate) node: NodeId,
    pub(crate) aspect: Aspect,
    pub(crate) precision: BridgeCorrespondencePrecision,
    pub(crate) admitted_source_widening:
        Option<crate::input::envelope::BridgeAspectChangeWideningCause>,
    pub(crate) allocation_sources: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ProvenCorrespondenceTargets {
    items: NonEmpty<InstalledCorrespondenceTarget>,
    _canonical: Proof<CanonicalOrder, StructuralProofAuthority>,
    _unique: Proof<Uniqueness, StructuralProofAuthority>,
}

impl ProvenCorrespondenceTargets {
    pub(crate) fn admit(
        mut items: Vec<InstalledCorrespondenceTarget>,
    ) -> Result<Self, BridgeCorrespondenceDenialKind> {
        items.sort();
        let (_, canonical) = CanonicalVec::try_from_sorted(items.clone())
            .expect("Bridge sorted correspondence targets")
            .into_parts();
        let (_, unique) = UniqueVec::try_from_unique(items.clone())
            .map_err(|_| BridgeCorrespondenceDenialKind::DuplicateTarget)?
            .into_parts();
        let items = NonEmpty::try_from_vec(items)
            .map_err(|_| BridgeCorrespondenceDenialKind::EmptyTargetSet)?;
        Ok(Self {
            items,
            _canonical: canonical,
            _unique: unique,
        })
    }

    pub(crate) fn as_slice(&self) -> &[InstalledCorrespondenceTarget] {
        self.items.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCorrespondenceBasis {
    pub(crate) source_installation_identity: Arc<str>,
    pub(crate) source_basis: Arc<str>,
    pub(crate) source_runtime_authority: u64,
    pub(crate) source_installation_generation: u64,
    pub(crate) source_authority_binding_identity: Arc<str>,
    pub(crate) declared_graph_role: Arc<str>,
    pub(crate) graph_participation_identity: Arc<str>,
    pub(crate) graph_adapter_identity: Arc<str>,
    pub(crate) authoritative_source_profile:
        Option<crate::input::envelope::BridgeAuthoritativeSourceProfile>,
    pub(crate) bridge_runtime_key: u64,
    pub(crate) signal_graph_instance_id: u64,
    pub(crate) signal_partitions: Vec<PartitionToken>,
}

impl BridgeCorrespondenceBasis {
    pub fn source_basis(&self) -> &str {
        &self.source_basis
    }

    pub const fn source_runtime_authority(&self) -> u64 {
        self.source_runtime_authority
    }

    pub const fn source_installation_generation(&self) -> u64 {
        self.source_installation_generation
    }

    pub fn declared_graph_role(&self) -> &str {
        &self.declared_graph_role
    }

    pub fn graph_participation_identity(&self) -> &str {
        &self.graph_participation_identity
    }

    pub fn graph_adapter_identity(&self) -> &str {
        &self.graph_adapter_identity
    }

    pub fn authoritative_source_profile(
        &self,
    ) -> Option<&crate::input::envelope::BridgeAuthoritativeSourceProfile> {
        self.authoritative_source_profile.as_ref()
    }

    /// Read-only installed Signal partition identities retained by this
    /// correspondence. These describe the performed binding; they grant no
    /// Signal mutation or Query admission authority.
    #[doc(hidden)]
    pub fn signal_partitions(&self) -> &[PartitionToken] {
        &self.signal_partitions
    }
}

struct BridgeCorrespondenceResolutionAuthority {
    _private: (),
}

impl AuthorityMarker for BridgeCorrespondenceResolutionAuthority {}

struct BridgeCorrespondenceReadinessAuthority {
    _private: (),
}

impl AuthorityMarker for BridgeCorrespondenceReadinessAuthority {}

type CorrespondenceReadyRecipe = ExecutionReadyRecipe<
    BridgeSemanticDependencyCandidate,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<BridgeCorrespondenceBasis>>,
>;

pub(crate) type CorrespondenceResolvedRecipe = Recipe<
    Resolved,
    BridgeSemanticDependencyCandidate,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<BridgeCorrespondenceBasis>>,
>;

#[derive(Debug)]
pub struct BridgeInstalledSemanticCorrespondence {
    pub(crate) ready: CorrespondenceReadyRecipe,
    pub(crate) targets: ProvenCorrespondenceTargets,
    admission_identity: super::BridgeCorrespondenceAdmissionIdentity,
    admission_counters: CorrespondenceAdmissionCounters,
}

impl BridgeInstalledSemanticCorrespondence {
    pub(crate) fn begin(
        dependency: BridgeSemanticDependencyCandidate,
    ) -> Recipe<Unresolved, BridgeSemanticDependencyCandidate> {
        Recipe::new(dependency)
    }

    pub(crate) fn resolve(
        unresolved: Recipe<Unresolved, BridgeSemanticDependencyCandidate>,
        basis: BridgeCorrespondenceBasis,
    ) -> CorrespondenceResolvedRecipe {
        unresolved.resolve_with(
            AuthorityWitness::from_authority_marker(BridgeCorrespondenceResolutionAuthority {
                _private: (),
            }),
            basis,
        )
    }

    pub(crate) fn admit_ready(
        resolved: CorrespondenceResolvedRecipe,
        targets: ProvenCorrespondenceTargets,
        admission_counters: CorrespondenceAdmissionCounters,
        signal_graph: &InstalledSignalGraphCapability,
        signal_targets: &InstalledSignalAspectSetCapability,
    ) -> Self {
        debug_assert_eq!(
            signal_graph.graph_instance_id(),
            signal_targets.graph_instance_id()
        );
        let ready = resolved
            .lower_with(signal_targets.lowering_witness())
            .ready_with(
                AuthorityWitness::from_authority_marker(BridgeCorrespondenceReadinessAuthority {
                    _private: (),
                }),
                signal_graph.graph_instance_id(),
            );
        Self {
            ready,
            targets,
            admission_identity: super::BridgeCorrespondenceAdmissionIdentity::issue(),
            admission_counters,
        }
    }

    pub fn dependency(&self) -> &BridgeSemanticDependencyCandidate {
        self.ready.payload()
    }

    pub fn target_count(&self) -> usize {
        self.targets.as_slice().len()
    }

    #[cfg(test)]
    pub(crate) fn targets(
        &self,
    ) -> impl ExactSizeIterator<Item = BridgeInstalledCorrespondenceTarget<'_>> {
        self.targets
            .as_slice()
            .iter()
            .map(BridgeInstalledCorrespondenceTarget)
    }

    pub fn basis(&self) -> &BridgeCorrespondenceBasis {
        self.ready.strong_basis().value()
    }

    pub fn admission_counters(&self) -> CorrespondenceAdmissionCounters {
        self.admission_counters
    }

    pub(crate) fn admission_identity(&self) -> &super::BridgeCorrespondenceAdmissionIdentity {
        &self.admission_identity
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg(test)]
pub(crate) struct BridgeInstalledCorrespondenceTarget<'a>(&'a InstalledCorrespondenceTarget);

#[cfg(test)]
impl BridgeInstalledCorrespondenceTarget<'_> {
    pub(crate) const fn signal_graph_instance_id(&self) -> u64 {
        self.0.signal_graph_instance_id
    }

    pub(crate) fn partition(&self) -> &PartitionToken {
        &self.0.partition
    }

    pub(crate) const fn node(&self) -> NodeId {
        self.0.node
    }

    pub(crate) const fn aspect(&self) -> Aspect {
        self.0.aspect
    }

    pub(crate) fn allocation_sources(&self) -> impl ExactSizeIterator<Item = &str> {
        self.0.allocation_sources.iter().map(String::as_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BridgeCorrespondencePrecision {
    Exact,
    DeclaredWidening,
}
