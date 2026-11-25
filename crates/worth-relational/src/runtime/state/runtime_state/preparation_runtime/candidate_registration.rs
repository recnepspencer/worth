//! Lend the original slot, receiver pair, and accounting to candidate registration.
use super::RelationalPreparationRuntime;
use crate::authority::commit::pipeline::candidate_preparation::registration::CandidateRegistrationInputs;

impl RelationalPreparationRuntime {
    pub(crate) fn candidate_registration_inputs<'a>(
        &'a self,
        symbols: &'a crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> CandidateRegistrationInputs<'a> {
        CandidateRegistrationInputs {
            snapshot_capacity: &self.snapshot_capacity,
            receiver_epoch: &self.configuration_epoch,
            invariant_generation: self.configuration_generation,
            maximum_candidates: self.config.publication.policy.max_prepared_candidates,
            symbols,
            instrumentation: &self.services.instrumentation,
            history: &self.history,
        }
    }
}
