//! Retain exactly the input from which the installed handler built this candidate.
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding,
};
use worth_query_installation::facade::{ApplicationSchema, InstalledCorrectionAuthority};

use super::super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};
pub use super::WorthQueryApplicationEffectProgram;
mod correction;
pub mod program;

impl<Schema, Operation, Input, Scope>
    crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
        Schema,
        Operation,
        Input,
        Scope,
    >
{
    pub fn requires_original_input_retention(
        &self,
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
    ) -> bool {
        self.captures_original_input()
    }

    pub(crate) fn captures_original_input(&self) -> bool {
        self.read_set
            .admission
            .allowed_graph_contract()
            .aftermath()
            .is_some_and(|aftermath| {
                aftermath.authority() != InstalledCorrectionAuthority::NotCorrectable
            })
    }

    pub fn requires_original_source_capture(
        &self,
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
    ) -> bool {
        self.captures_original_source()
    }

    pub(crate) fn captures_original_source(&self) -> bool {
        self.captures_original_input()
            || !self
                .read_set
                .admission
                .allowed_graph_contract()
                .source_inspection_disclosures()
                .is_empty()
    }

    /// Whether the installed operation retains original meaning for correction.
    /// Keeps native immutable payload custody after the original request ends.
    /// The sealed handler proof prevents substitution of another input or binding.
    pub fn retain_original_mutation_input<Binding>(
        self,
        _access: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::ports::EffectProgramOriginalInputRuntimePort<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: serde::Serialize + Send + Sync + 'static,
    {
        self.retain_admitted_source_input::<Binding>(phase, runtime, input)
    }

    /// The owned SDK stage uses the same checked retention without issuing a publication token.
    pub(crate) fn retain_admitted_source_input<Binding>(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::ports::EffectProgramOriginalInputRuntimePort<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: serde::Serialize + Send + Sync + 'static,
    {
        if !self.captures_original_source() {
            return Ok(self);
        }
        let retain_input = self.captures_original_input();
        self.retain_checked_input::<Binding>(phase, runtime, input, retain_input)
    }

    /// Keeps the performed inverse's actual input for retry, including an irreversible inverse.
    /// This does not change its installed correction or disclosure contract.
    pub fn retain_performed_correction_input<Binding>(
        self,
        _access: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::ports::EffectProgramOriginalInputRuntimePort<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: serde::Serialize + Send + Sync + 'static,
    {
        self.retain_checked_input::<Binding>(phase, runtime, input, true)
    }

    fn retain_checked_input<Binding>(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::ports::EffectProgramOriginalInputRuntimePort<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
        retain_input: bool,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: serde::Serialize + Send + Sync + 'static,
    {
        let matches_handler =
            self.read_set
                .mutation_handler_binding
                .as_ref()
                .is_some_and(|proof| {
                    proof.binding == Binding::IDENTITY && proof.input_identity == *input.identity()
                });
        if !matches_handler {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                "original input differs from the installed handler candidate",
            ));
        }
        if self.read_set.admission.runtime_authority() != runtime.authority_identity()
            || (self.captures_original_input()
                && self.read_set.admission.original_scope().is_none())
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::CorrectionCaptureDenied,
                "original correction scope was not retained by this admission owner",
            ));
        }
        let execution = runtime.execution_request_for_phase(phase)?;
        if self
            .read_set
            .admission
            .original_inspection_disclosure()
            .is_none()
        {
            let inspection =
                crate::domain_computation::primary_graph::source_disclosure::capture_inspection::<
                    Schema,
                    Binding,
                >(
                    &execution,
                    runtime.installed_schema(),
                    input.input(),
                    self.read_set.admission.original_scope(),
                )?;
            self.read_set
                .admission
                .retain_original_inspection_disclosure(inspection);
        }
        if retain_input {
            let correction = correction::capture::<Schema, Binding>(
                &execution,
                runtime.installed_schema(),
                input.input(),
            )?;
            self.read_set.admission.retain_original_input::<Binding>(
                input
                    .retain_canonical_input()
                    .map_err(WorthQueryApplicationAttemptDenial::retained_input)?,
                correction,
            );
        }
        Ok(self)
    }
}
