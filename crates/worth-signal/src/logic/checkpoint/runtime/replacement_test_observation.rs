use crate::data::checkpoint::CheckpointBarrier;
use crate::data::dirty_set::DomainImpact;
use crate::data::telemetry::RuntimeTelemetry;

use super::CheckpointRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointReplacementObservation<D: Copy + Ord, I: Copy + Ord> {
    pub(crate) dirty: Vec<(D, DomainImpact<I>)>,
    pub(crate) barriers: Vec<(D, CheckpointBarrier)>,
    pub(crate) default_barrier: CheckpointBarrier,
    pub(crate) telemetry: RuntimeTelemetry,
}

impl<D: Copy + Ord, I: Copy + Ord> CheckpointRuntime<D, I> {
    pub(crate) fn replacement_observation(
        &self,
        domains: &[D],
    ) -> CheckpointReplacementObservation<D, I> {
        CheckpointReplacementObservation {
            dirty: self
                .dirty
                .dirty_domains()
                .filter_map(|domain| {
                    self.dirty
                        .impact_for(domain)
                        .cloned()
                        .map(|impact| (domain, impact))
                })
                .collect(),
            barriers: domains
                .iter()
                .map(|domain| (*domain, self.policy.barrier_for(*domain)))
                .collect(),
            default_barrier: self.policy.barrier_for_default(),
            telemetry: self.telemetry,
        }
    }
}
