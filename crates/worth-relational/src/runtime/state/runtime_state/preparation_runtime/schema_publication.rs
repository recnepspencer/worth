//! Narrow the admitted preparation snapshot to the sources its two assembly phases read.
use super::RelationalPreparationRuntime;
use crate::authority::commit::phases::publication::CanonicalEnvelopeInputs;
use crate::authority::commit::phases::schema_continuity::inputs::{
    SchemaContinuityInputs, SchemaContinuityRequest,
};
use crate::mvcc::RelationalTransactionValidationInput;
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;

impl RelationalPreparationRuntime {
    pub(crate) fn preparation_diagnostics(&self) -> PreparationDiagnostics<'_> {
        PreparationDiagnostics {
            profile: &self.config.diagnostics.profile,
            store: &self.diagnostics,
            instrumentation: &self.services.instrumentation,
        }
    }
}

impl<'a> From<&'a RelationalPreparationRuntime> for CanonicalEnvelopeInputs<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            history: &runtime.history,
            diagnostics: runtime.preparation_diagnostics(),
            symbols: &runtime.services.symbols,
            symbol_references: &runtime.symbol_references,
        }
    }
}

impl<'a> From<&'a RelationalPreparationRuntime> for SchemaContinuityInputs<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            schema: &runtime.config.schema,
            history: &runtime.history,
            diagnostics: runtime.preparation_diagnostics(),
        }
    }
}

impl<'a> From<&'a RelationalTransactionValidationInput> for SchemaContinuityRequest<'a> {
    fn from(input: &'a RelationalTransactionValidationInput) -> Self {
        Self {
            basis: input.basis(),
            authority: input.schema_authority(),
            copied_authority: input.schema_authority_input(),
            transition: input.proposed_schema_transition(),
            reconciliation: input.schema_reconciliation_policy(),
        }
    }
}
