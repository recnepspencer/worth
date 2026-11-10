use crate::application::{
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;
use crate::signal_compatibility_orchestration::{
    orchestrate_signal_compatibility_on_handle,
    ordinary_outcome_from_signal_compatibility_orchestration_checked,
    WorthQuerySignalCompatibilityOrchestration, WorthQuerySignalCompatibilityOrchestrationChecked,
    WorthQuerySignalCompatibilityOrchestrationInput,
    WorthQuerySignalCompatibilityOrchestrationOutcome,
    WorthQuerySignalCompatibilityOrchestrationTranscript,
};

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn orchestrate_signal_compatibility<I: WorthQueryDeclarationInput<D>>(
        &self,
        input: WorthQuerySignalCompatibilityOrchestrationInput<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial>
    {
        Ok(self
            .orchestrate_signal_compatibility_checked(input)?
            .into_outcome())
    }

    pub fn orchestrate_signal_compatibility_outcome<I: WorthQueryDeclarationInput<D>>(
        &self,
        input: WorthQuerySignalCompatibilityOrchestrationInput<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
        IdentityDerivationDenial,
    > {
        Ok(
            ordinary_outcome_from_signal_compatibility_orchestration_checked(
                self.orchestrate_signal_compatibility_checked(input)?,
            ),
        )
    }

    pub fn orchestrate_signal_compatibility_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        input: WorthQuerySignalCompatibilityOrchestrationInput<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial>
    {
        Ok(self
            .orchestrate_signal_compatibility_proof(input)?
            .into_checked())
    }

    pub fn orchestrate_signal_compatibility_proof<I: WorthQueryDeclarationInput<D>>(
        &self,
        input: WorthQuerySignalCompatibilityOrchestrationInput<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    {
        orchestrate_signal_compatibility_on_handle(self, input)
    }
}
