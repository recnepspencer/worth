//! Provisional telemetry and diagnostics for a candidate that may be abandoned.
use super::RelationalPreparationRuntime;
use crate::authority::commit::preparation::symbol_rounds::{
    PreparationRoundObservations, SymbolRoundInputs,
};

impl RelationalPreparationRuntime {
    pub(crate) fn preparation_round(&self) -> Self {
        let observation = PreparationRoundObservations::new(&self.services.instrumentation);
        let mut round = self.clone();
        round.services.instrumentation = observation.instrumentation;
        round.diagnostics = observation.diagnostics;
        round
    }

    /// Settle only a miss-free round. Identity and request-work owners were
    /// shared throughout; this never rewinds either issuance or actual work.
    pub(crate) fn settle_preparation_round(&self, round: &Self) {
        PreparationRoundObservations::settle(
            &self.services.instrumentation,
            &self.diagnostics,
            &round.services.instrumentation,
            &round.diagnostics,
        );
    }
}

impl<'a> From<&'a RelationalPreparationRuntime>
    for SymbolRoundInputs<'a, RelationalPreparationRuntime>
{
    fn from(owner: &'a RelationalPreparationRuntime) -> Self {
        Self {
            source: owner,
            symbols: &owner.services.symbols,
            instrumentation: &owner.services.instrumentation,
            diagnostics: &owner.diagnostics,
            bind: |owner, observations| {
                let mut round = owner.clone();
                round.services.instrumentation = observations.instrumentation.clone();
                round.diagnostics = observations.diagnostics.clone();
                round
            },
        }
    }
}
