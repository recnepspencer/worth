use crate::application::{WorthQueryDeclarationInput, WorthQueryDomainEntryMarker};
use crate::contribution_composed_orchestration::{
    orchestrate_declaration_with_contributions_on_handle,
    ordinary_outcome_from_contribution_composed_checked,
    WorthQueryContributionComposedOrchestration,
    WorthQueryContributionComposedOrchestrationChecked,
    WorthQueryContributionComposedOrchestrationInput,
    WorthQueryContributionComposedOrchestrationOutcome,
    WorthQueryContributionComposedOrchestrationTranscript,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;

use super::WorthQueryInstalledDomainDeclarationContext;
use crate::application::WorthQueryDomainOperatingContext;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn orchestrate_declaration_with_contributions<I>(
        &self,
        input: WorthQueryContributionComposedOrchestrationInput<D, I>,
    ) -> Result<
        WorthQueryContributionComposedOrchestration<D, I>,
        WorthQueryContributionComposedOrchestrationOutcome<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        match orchestrate_declaration_with_contributions_on_handle(self, input)?.into_checked() {
            WorthQueryContributionComposedOrchestrationOutcome::Bound(value) => Ok(value),
            other => Err(other),
        }
    }

    pub fn orchestrate_declaration_with_contributions_outcome<I>(
        &self,
        input: WorthQueryContributionComposedOrchestrationInput<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQueryContributionComposedOrchestration<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(ordinary_outcome_from_contribution_composed_checked(
            orchestrate_declaration_with_contributions_on_handle(self, input)?.into_checked(),
        )?)
    }

    pub fn orchestrate_declaration_with_contributions_checked<I>(
        &self,
        input: WorthQueryContributionComposedOrchestrationInput<D, I>,
    ) -> Result<WorthQueryContributionComposedOrchestrationChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(orchestrate_declaration_with_contributions_on_handle(self, input)?.into_checked())
    }

    pub fn orchestrate_declaration_with_contributions_proof<I>(
        &self,
        input: WorthQueryContributionComposedOrchestrationInput<D, I>,
    ) -> Result<WorthQueryContributionComposedOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(orchestrate_declaration_with_contributions_on_handle(
            self, input,
        )?)
    }
}
