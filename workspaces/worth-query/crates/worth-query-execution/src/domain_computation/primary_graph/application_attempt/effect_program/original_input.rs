//! Retain exactly the input from which the installed handler built this candidate.
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding,
};
use worth_query_installation::facade::{ApplicationSchema, InstalledCorrectionAuthority};

use super::WorthQueryApplicationEffectProgram;
use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};
mod correction;
mod program;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    /// Whether the installed operation retains original meaning for correction.
    pub fn requires_original_input_retention(
        &self,
        _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> bool {
        self.read_set
            .admission
            .allowed_graph_contract()
            .aftermath()
            .is_some_and(|aftermath| {
                aftermath.authority() != InstalledCorrectionAuthority::NotCorrectable
            })
    }

    /// Keeps native immutable payload custody after the original request ends.
    /// The sealed handler proof prevents substitution of another input or binding.
    pub fn retain_original_mutation_input<Binding>(
        self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: Send + Sync + 'static,
    {
        if !self.requires_original_input_retention(access) {
            return Ok(self);
        }
        self.retain_checked_input::<Binding>(access, runtime, input)
    }

    /// Keeps the performed inverse's actual input for retry, including an irreversible inverse.
    /// This does not change its installed correction or disclosure contract.
    pub fn retain_performed_correction_input<Binding>(
        self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: Send + Sync + 'static,
    {
        self.retain_checked_input::<Binding>(access, runtime, input)
    }

    fn retain_checked_input<Binding>(
        mut self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: Send + Sync + 'static,
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
        if self.read_set.admission.runtime_authority() != runtime.runtime.authority_identity()
            || (self.requires_original_input_retention(access)
                && self.read_set.admission.original_scope().is_none())
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::CorrectionCaptureDenied,
                "original correction scope was not retained by this admission owner",
            ));
        }
        let correction = correction::capture::<Schema, Binding>(runtime, input.input())?;
        self.read_set
            .admission
            .retain_original_input::<Binding>(input, correction);
        Ok(self)
    }
}
