//! Lend the original slot, receiver pair, and accounting to candidate registration.
use super::RelationalPreparationRuntime;
use crate::authority::commit::pipeline::candidate_preparation::registration::CandidateRegistrationInputs;

impl<'a> From<&'a RelationalPreparationRuntime> for CandidateRegistrationInputs<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            snapshot_capacity: &runtime.snapshot_capacity,
            receiver_epoch: &runtime.configuration_epoch,
            invariant_generation: runtime.configuration_generation,
            maximum_candidates: runtime.config.publication.policy.max_prepared_candidates,
            symbols: &runtime.services.symbols,
            instrumentation: &runtime.services.instrumentation,
            history: &runtime.history,
        }
    }
}
