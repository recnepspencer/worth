use crate::data::checkpoint::CheckpointBarrier;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::handle::NodeId;
use crate::data::tier::TierPolicy;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::runtime_policy::SignalRuntimePolicy;
use crate::runtime_policy::SignalRuntimePolicyCompilationDenial;

#[cfg(test)]
use super::RawRuntimeMerge;
use super::{
    runtime_observation::{
        MatchingObserverSet, ObservationHandle, ObservationListener, ObservationPolicy,
        ObservationRegistrySummary, ObservedNodeSet,
    },
    runtime_state::SignalRuntime,
    RuntimeHistory, RuntimeMerge,
};

pub use crate::observation::session::{
    SignalObservationAdmissionDenial, SignalObservationCompletion, SignalObservationRequest,
    SignalObservationSession, SignalObservationSurface,
};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn prepare_for_observation(&mut self) {
        self.graph.prepare_for_observation();
    }

    pub fn runtime_policy(&self) -> SignalRuntimePolicy {
        self.graph.runtime_policy()
    }

    pub fn diagnostics(&self) -> crate::diagnostics::Diagnostics<'_> {
        self.observe().diagnostics()
    }

    pub fn history(&mut self) -> RuntimeHistory<'_, D, I, E, Ctx, T> {
        RuntimeHistory::new(self)
    }

    pub fn merge(&mut self) -> RuntimeMerge<'_, D, I, E, Ctx, T> {
        RuntimeMerge::new(self)
    }

    #[cfg(test)]
    pub(crate) fn merge_raw(&mut self) -> RawRuntimeMerge<'_, D, I, E, Ctx, T> {
        RawRuntimeMerge::new(self)
    }

    pub fn observe_nodes(
        &mut self,
        policy: ObservationPolicy,
        nodes: impl IntoIterator<Item = NodeId>,
        listener: Box<dyn ObservationListener<D, I, E, Ctx, T>>,
    ) -> ObservationHandle {
        let observed_nodes = ObservedNodeSet::from_nodes(nodes);
        self.observations_mut()
            .register_nodes(policy, observed_nodes, listener)
    }

    pub fn unobserve(&mut self, handle: ObservationHandle) -> bool {
        self.observations_mut().unsubscribe(handle)
    }

    pub fn observation_summary(&self) -> ObservationRegistrySummary {
        self.observations().summary()
    }

    pub fn matching_observers_for_node(&self, node: NodeId) -> MatchingObserverSet {
        self.observations().matching_observers_for_node(node)
    }

    /// Every node with at least one live observer, in node order.
    pub fn observed_node_ids(&self) -> Vec<NodeId> {
        self.observations().observed_node_ids().collect()
    }

    /// Reset the runtime to the named diagnostics tier preset.
    ///
    /// This is a convenience for switching back to a stock posture.
    /// If you already have narrower retention or replay overrides, prefer
    /// `set_runtime_policy(...)` so you do not accidentally throw them away.
    pub fn reset_runtime_policy_to_tier(&mut self, profile: DiagnosticsTier) {
        self.graph.reset_runtime_policy_to_tier(profile);
    }

    /// Apply the full runtime policy bundle.
    ///
    /// This is the canonical owner for runtime posture and diagnostics
    /// richness once you move past the stock presets.
    pub fn set_runtime_policy(&mut self, policy: SignalRuntimePolicy) {
        self.graph.set_runtime_policy(policy);
    }

    /// Apply a full runtime policy while preserving the compiler's typed
    /// admission denial for caller-provided configuration.
    pub fn try_set_runtime_policy(
        &mut self,
        policy: SignalRuntimePolicy,
    ) -> Result<(), SignalRuntimePolicyCompilationDenial> {
        self.graph.try_set_runtime_policy(policy)
    }

    /// Adjust the current runtime policy while preserving typed admission denial.
    pub fn try_adjust_runtime_policy<F>(
        &mut self,
        adjust: F,
    ) -> Result<(), SignalRuntimePolicyCompilationDenial>
    where
        F: FnOnce(SignalRuntimePolicy) -> SignalRuntimePolicy,
    {
        let next = adjust(self.runtime_policy());
        self.try_set_runtime_policy(next)
    }

    /// Adjust the current runtime policy in one place when the result is known valid.
    ///
    /// Callers that construct policy values dynamically should prefer
    /// [`try_adjust_runtime_policy`](Self::try_adjust_runtime_policy), which
    /// returns the compiler's typed admission denial instead of panicking.
    pub fn adjust_runtime_policy<F>(&mut self, adjust: F)
    where
        F: FnOnce(SignalRuntimePolicy) -> SignalRuntimePolicy,
    {
        let next = adjust(self.runtime_policy());
        self.set_runtime_policy(next);
    }

    pub fn set_node_tier(&mut self, node: NodeId, tier: T) {
        self.config.set_node_tier(&self.graph, node, tier);
    }

    pub fn set_tier_policy(&mut self, policy: TierPolicy<T>) {
        self.config.set_tier_policy(policy);
    }

    /// Adjust the policy for one existing tier.
    ///
    /// Returns `true` when the tier existed and was updated.
    pub fn adjust_tier_policy<F>(&mut self, tier: T, adjust: F) -> bool
    where
        F: FnOnce(TierPolicy<T>) -> TierPolicy<T>,
    {
        let Some(current) = self.config.tier_policies().get(tier).cloned() else {
            return false;
        };
        self.set_tier_policy(adjust(current));
        true
    }

    pub fn set_fallback_comparator(&mut self, policy: VersionComparatorPolicy) {
        self.config.set_fallback_comparator(policy);
    }

    /// Adjust the fallback comparator in one place.
    pub fn adjust_fallback_comparator<F>(&mut self, adjust: F)
    where
        F: FnOnce(VersionComparatorPolicy) -> VersionComparatorPolicy,
    {
        let next = adjust(self.config.fallback_comparator().clone());
        self.set_fallback_comparator(next);
    }

    /// Set one domain-specific checkpoint barrier.
    pub fn set_domain_checkpoint_barrier(&mut self, domain: D, barrier: CheckpointBarrier) {
        self.checkpoint.policy_mut().set_barrier(domain, barrier);
    }

    /// Adjust the full checkpoint policy in one place.
    pub fn adjust_checkpoint_policy<F>(&mut self, adjust: F)
    where
        F: FnOnce(&mut crate::data::checkpoint_policy::CheckpointPolicy<D>),
    {
        adjust(self.checkpoint.policy_mut());
    }
}
