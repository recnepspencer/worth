mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::node_meta::NodeMetaStore;
use crate::data::tier_policy_table::TierPolicyTable;

/// Version-change comparator policy for dependency snapshot checks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum VersionComparatorPolicy {
    /// Treat any numeric difference as a meaningful change.
    #[default]
    Exact,
    /// Ignore differences up to and including `epsilon`.
    Tolerance { epsilon: u64 },
    /// Compare downstream propagation using host-supplied output identity.
    OutputIdentity,
    /// Delegate comparison to embedding runtime callback by stable key.
    Custom { key: String },
    /// Runtime-affine comparator installed by an admitted graph-lowering owner.
    #[serde(skip)]
    Installed {
        identity: InstalledSignalComparatorIdentity,
    },
}

impl PartialEq for VersionComparatorPolicy {
    fn eq(&self, candidate: &Self) -> bool {
        match (self, candidate) {
            (Self::Exact, Self::Exact) | (Self::OutputIdentity, Self::OutputIdentity) => true,
            (Self::Tolerance { epsilon: current }, Self::Tolerance { epsilon: candidate }) => {
                current == candidate
            }
            (Self::Custom { key: current }, Self::Custom { key: candidate }) => {
                current == candidate
            }
            (
                Self::Installed { identity: current },
                Self::Installed {
                    identity: candidate,
                },
            ) => current.is_same_installed_identity(candidate),
            _ => false,
        }
    }
}

impl Eq for VersionComparatorPolicy {}

impl std::hash::Hash for VersionComparatorPolicy {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Tolerance { epsilon } => epsilon.hash(state),
            Self::Custom { key } => key.hash(state),
            Self::Installed { identity } => {
                identity.graph_instance_id.hash(state);
                identity.node.hash(state);
                identity.role.hash(state);
            }
            Self::Exact | Self::OutputIdentity => {}
        }
    }
}

#[derive(Clone)]
pub struct InstalledSignalComparatorIdentity {
    graph_instance_id: u64,
    node: NodeId,
    role: InstalledSignalComparatorRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum InstalledSignalComparatorRole {
    DependencyVersion,
    OutputEquivalence,
    ArtifactReuse,
}

impl InstalledSignalComparatorIdentity {
    pub(crate) const fn new(
        graph_instance_id: u64,
        node: NodeId,
        role: InstalledSignalComparatorRole,
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

    pub(crate) const fn role(&self) -> InstalledSignalComparatorRole {
        self.role
    }

    pub(crate) fn is_same_installed_identity(&self, candidate: &Self) -> bool {
        self.graph_instance_id == candidate.graph_instance_id
            && self.node == candidate.node
            && self.role == candidate.role
    }
}

impl std::fmt::Debug for InstalledSignalComparatorIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InstalledSignalComparatorIdentity")
            .finish_non_exhaustive()
    }
}

impl VersionComparatorPolicy {
    /// Return true when `current` should be treated as changed vs `cached`.
    pub fn has_meaningful_change<R: VersionComparatorResolver>(
        &self,
        aspect: Aspect,
        cached: u64,
        current: u64,
        resolver: &mut R,
    ) -> Result<bool, SignalError> {
        Ok(match self {
            Self::Exact => current != cached,
            Self::Tolerance { epsilon } => current.abs_diff(cached) > *epsilon,
            Self::OutputIdentity => current != cached,
            Self::Custom { key } => resolver.resolve(key, aspect, cached, current)?,
            Self::Installed { identity } => {
                resolver.resolve_installed(identity, aspect, cached, current)?
            }
        })
    }
}

/// Host callback used when a node declares `VersionComparatorPolicy::Custom`.
pub trait VersionComparatorResolver {
    /// Return true if `(cached -> current)` is a meaningful change.
    fn resolve(
        &mut self,
        key: &str,
        aspect: Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, SignalError>;

    fn resolve_installed(
        &mut self,
        _identity: &InstalledSignalComparatorIdentity,
        aspect: Aspect,
        _cached: u64,
        _current: u64,
    ) -> Result<bool, SignalError> {
        Err(SignalError::invalid_input(format!(
            "installed comparator requires its admitted runtime resolver for aspect {aspect:?}"
        )))
    }
}

impl<T: VersionComparatorResolver + ?Sized> VersionComparatorResolver for &mut T {
    fn resolve(
        &mut self,
        key: &str,
        aspect: Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, SignalError> {
        (**self).resolve(key, aspect, cached, current)
    }

    fn resolve_installed(
        &mut self,
        identity: &InstalledSignalComparatorIdentity,
        aspect: Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, SignalError> {
        (**self).resolve_installed(identity, aspect, cached, current)
    }
}

/// Default resolver used when callers do not provide custom comparator hooks.
pub struct DefaultComparatorResolver;

impl VersionComparatorResolver for DefaultComparatorResolver {
    fn resolve(
        &mut self,
        key: &str,
        aspect: Aspect,
        _cached: u64,
        _current: u64,
    ) -> Result<bool, SignalError> {
        Err(SignalError::InvalidInput {
            message: format!("Custom comparator '{key}' requires a resolver for aspect {aspect:?}"),
            context: None,
        })
    }
}

/// Comparator policy resolution contract used by evaluation runtime.
pub trait ComparatorPolicyResolver: VersionComparatorResolver {
    /// Resolve effective comparator for one node.
    fn policy_for_node(
        &self,
        node: NodeId,
        node_override: Option<&VersionComparatorPolicy>,
    ) -> VersionComparatorPolicy;
}

impl<T: ComparatorPolicyResolver + ?Sized> ComparatorPolicyResolver for &mut T {
    fn policy_for_node(
        &self,
        node: NodeId,
        node_override: Option<&VersionComparatorPolicy>,
    ) -> VersionComparatorPolicy {
        (**self).policy_for_node(node, node_override)
    }
}

/// Default policy resolver:
/// node override if present, otherwise global fallback `Exact`.
pub struct DefaultComparatorPolicyResolver<R = DefaultComparatorResolver> {
    pub fallback: VersionComparatorPolicy,
    pub custom: R,
}

impl Default for DefaultComparatorPolicyResolver<DefaultComparatorResolver> {
    fn default() -> Self {
        Self {
            fallback: VersionComparatorPolicy::Exact,
            custom: DefaultComparatorResolver,
        }
    }
}

impl<R: VersionComparatorResolver> VersionComparatorResolver
    for DefaultComparatorPolicyResolver<R>
{
    fn resolve(
        &mut self,
        key: &str,
        aspect: Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, SignalError> {
        self.custom.resolve(key, aspect, cached, current)
    }
}

impl<R: VersionComparatorResolver> ComparatorPolicyResolver for DefaultComparatorPolicyResolver<R> {
    fn policy_for_node(
        &self,
        _node: NodeId,
        node_override: Option<&VersionComparatorPolicy>,
    ) -> VersionComparatorPolicy {
        node_override
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }
}

/// Tier-aware policy resolver with per-node tier assignments.
pub struct TierPolicyResolver<'a, T: Copy + Ord, R = DefaultComparatorResolver> {
    node_meta: &'a NodeMetaStore<T>,
    tier_policies: &'a TierPolicyTable<T>,
    fallback: &'a VersionComparatorPolicy,
    custom: R,
}

impl<'a, T: Copy + Ord> TierPolicyResolver<'a, T, DefaultComparatorResolver> {
    /// Build resolver from explicit node-tier assignments and tier policies.
    pub fn new(
        node_meta: &'a NodeMetaStore<T>,
        tier_policies: &'a TierPolicyTable<T>,
        fallback: &'a VersionComparatorPolicy,
    ) -> Self {
        Self {
            node_meta,
            tier_policies,
            fallback,
            custom: DefaultComparatorResolver,
        }
    }
}

impl<'a, T: Copy + Ord, R> TierPolicyResolver<'a, T, R> {
    /// Attach custom comparator resolver for `Custom` comparator keys.
    pub fn with_custom_resolver<R2>(self, custom: R2) -> TierPolicyResolver<'a, T, R2> {
        TierPolicyResolver {
            node_meta: self.node_meta,
            tier_policies: self.tier_policies,
            fallback: self.fallback,
            custom,
        }
    }
}

impl<T: Copy + Ord, R: VersionComparatorResolver> VersionComparatorResolver
    for TierPolicyResolver<'_, T, R>
{
    fn resolve(
        &mut self,
        key: &str,
        aspect: Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, SignalError> {
        self.custom.resolve(key, aspect, cached, current)
    }
}

impl<T: Copy + Ord, R: VersionComparatorResolver> ComparatorPolicyResolver
    for TierPolicyResolver<'_, T, R>
{
    fn policy_for_node(
        &self,
        node: NodeId,
        node_override: Option<&VersionComparatorPolicy>,
    ) -> VersionComparatorPolicy {
        if let Some(override_policy) = node_override {
            return override_policy.clone();
        }
        if let Some(tier) = self.node_meta.tier_for_node(node) {
            if let Some(policy) = self.tier_policies.get(tier) {
                return policy.default_comparator.clone();
            }
        }
        self.fallback.clone()
    }
}
