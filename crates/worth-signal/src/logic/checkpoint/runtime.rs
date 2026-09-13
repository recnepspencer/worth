//! Tier-0 checkpoint runtime for batched domain refresh.

use crate::clock::RuntimeInstant;
use crate::data::checkpoint::CheckpointBarrier;
use crate::data::checkpoint_policy::CheckpointPolicy;
use crate::data::dirty_set::{BatchedDirtySet, DomainImpact};
use crate::data::effect_mapping::EffectMapping;
use crate::data::error::SignalError;
use crate::data::evaluator::CheckpointEvaluator;
use crate::data::telemetry::RuntimeTelemetry;

#[cfg(test)]
#[path = "runtime/replacement_test_observation.rs"]
mod replacement_test_observation;
#[cfg(test)]
pub(crate) use replacement_test_observation::CheckpointReplacementObservation;

/// Runtime state for batched Tier-0 signal scheduling.
#[derive(Debug, Clone)]
pub struct CheckpointRuntime<D: Copy + Ord, I: Copy + Ord> {
    dirty: BatchedDirtySet<D, I>,
    policy: CheckpointPolicy<D>,
    telemetry: RuntimeTelemetry,
}

impl<D: Copy + Ord, I: Copy + Ord> CheckpointRuntime<D, I> {
    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            dirty: self.dirty.fork_persistent(),
            policy: self.policy.fork_persistent(),
            telemetry: self.telemetry,
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            dirty: self.dirty.fork_storage_identity(),
            policy: self.policy.fork_storage_identity(),
            telemetry: self.telemetry,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.dirty.shares_storage_with(&other.dirty)
            && self.policy.shares_storage_with(&other.policy)
    }

    /// Create a new runtime with a per-domain checkpoint policy.
    pub fn new(policy: CheckpointPolicy<D>) -> Self {
        Self {
            dirty: BatchedDirtySet::new(),
            policy,
            telemetry: RuntimeTelemetry::default(),
        }
    }

    /// Read-only dirty state.
    pub fn dirty(&self) -> &BatchedDirtySet<D, I> {
        &self.dirty
    }

    /// Mutable dirty state.
    pub fn dirty_mut(&mut self) -> &mut BatchedDirtySet<D, I> {
        &mut self.dirty
    }

    /// Read-only checkpoint policy.
    pub fn policy(&self) -> &CheckpointPolicy<D> {
        &self.policy
    }

    /// Mutable checkpoint policy.
    pub fn policy_mut(&mut self) -> &mut CheckpointPolicy<D> {
        &mut self.policy
    }

    /// Runtime telemetry snapshot.
    pub fn telemetry(&self) -> &RuntimeTelemetry {
        &self.telemetry
    }

    pub(crate) fn telemetry_mut(&mut self) -> &mut RuntimeTelemetry {
        &mut self.telemetry
    }

    /// Reset runtime telemetry counters.
    pub fn reset_telemetry(&mut self) {
        self.telemetry = RuntimeTelemetry::default();
    }

    /// Route one effect into this runtime's dirty set.
    pub fn record_effect<M>(&mut self, effect: &M::Effect)
    where
        M: EffectMapping<Domain = D, Impact = I>,
    {
        M::route(effect, &mut self.dirty);
    }

    /// Mark one domain globally dirty.
    pub fn mark_domain_global(&mut self, domain: D) {
        self.dirty.mark_domain_global(domain);
    }

    /// Mark one scoped impact dirty.
    pub fn mark_domain_scoped(&mut self, domain: D, impact: I) {
        self.dirty.mark_domain_scoped(domain, impact);
    }

    /// Flush domains scheduled for this barrier.
    pub fn flush<E>(
        &mut self,
        barrier: CheckpointBarrier,
        evaluator: &mut E,
        ctx: &mut E::Context,
    ) -> Result<usize, SignalError>
    where
        E: CheckpointEvaluator<Domain = D, Impact = I>,
    {
        self.flush_with_capture(barrier, evaluator, ctx, true)
    }

    pub(crate) fn flush_with_capture<E>(
        &mut self,
        barrier: CheckpointBarrier,
        evaluator: &mut E,
        ctx: &mut E::Context,
        capture_telemetry: bool,
    ) -> Result<usize, SignalError>
    where
        E: CheckpointEvaluator<Domain = D, Impact = I>,
    {
        let flush_start = RuntimeInstant::now();
        let domains: Vec<D> = self
            .dirty
            .dirty_domains()
            .filter(|domain| self.policy.barrier_for(*domain) == barrier)
            .collect();

        for domain in &domains {
            let impact = self
                .dirty
                .take_domain_impact(*domain)
                .unwrap_or_else(DomainImpact::empty);
            evaluator.refresh(*domain, impact, ctx)?;
        }

        if capture_telemetry {
            self.telemetry.checkpoint.checkpoint_flushes += 1;
            self.telemetry.checkpoint.checkpoint_flush_nanos += flush_start.elapsed().as_nanos();
        }

        Ok(domains.len())
    }

    /// Ensure one domain is refreshed immediately regardless of barrier policy.
    pub fn ensure_fresh<E>(
        &mut self,
        domain: D,
        evaluator: &mut E,
        ctx: &mut E::Context,
    ) -> Result<bool, SignalError>
    where
        E: CheckpointEvaluator<Domain = D, Impact = I>,
    {
        let Some(impact) = self.dirty.take_domain_impact(domain) else {
            return Ok(false);
        };
        evaluator.refresh(domain, impact, ctx)?;
        Ok(true)
    }
}
