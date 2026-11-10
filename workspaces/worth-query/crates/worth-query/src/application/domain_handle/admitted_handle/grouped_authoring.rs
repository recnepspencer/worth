use crate::application::{WorthQueryDeclarationInput, WorthQueryDomainEntryMarker};
use crate::grouped_authoring::{
    ordinary_outcome_from_grouped_orchestration_checked,
    worth_query_grouped_contribution_checked_on_handle,
    worth_query_grouped_declaration_checked_on_handle,
    worth_query_grouped_envelope_checked_on_handle, worth_query_grouped_envelope_proof_on_handle,
    worth_query_grouped_orchestration_checked_on_handle,
    worth_query_grouped_orchestration_proof_on_handle,
    worth_query_grouped_receipt_checked_on_handle, worth_query_grouped_receipt_proof_on_handle,
    worth_query_grouped_route_checked_on_handle, worth_query_grouped_route_proof_on_handle,
    worth_query_grouped_support_report, WorthQueryGroupedContributionComposition,
    WorthQueryGroupedContributionInput, WorthQueryGroupedContributionStop,
    WorthQueryGroupedDeclarationArtifact, WorthQueryGroupedDeclarationChecked,
    WorthQueryGroupedDeclarationInput, WorthQueryGroupedDeclarationStop,
    WorthQueryGroupedEnvelopeChecked, WorthQueryGroupedEnvelopeTranscript,
    WorthQueryGroupedOrchestration, WorthQueryGroupedOrchestrationChecked,
    WorthQueryGroupedOrchestrationStop, WorthQueryGroupedOrchestrationTranscript,
    WorthQueryGroupedReceiptChecked, WorthQueryGroupedReceiptTranscript,
    WorthQueryGroupedRouteChecked, WorthQueryGroupedRouteTranscript,
    WorthQueryGroupedSupportReport,
};
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;

use super::WorthQueryInstalledDomainDeclarationContext;
use crate::application::WorthQueryDomainOperatingContext;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn declare_grouped_checked<I>(
        &self,
        input: WorthQueryGroupedDeclarationInput<D, I>,
    ) -> WorthQueryGroupedDeclarationChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_declaration_checked_on_handle(self, input)
    }

    pub fn declare_grouped<I>(
        &self,
        input: WorthQueryGroupedDeclarationInput<D, I>,
    ) -> Result<WorthQueryGroupedDeclarationArtifact<D, I>, WorthQueryGroupedDeclarationStop>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        match self.declare_grouped_checked(input) {
            WorthQueryGroupedDeclarationChecked::Bound(value) => Ok(value),
            WorthQueryGroupedDeclarationChecked::MemberStopped(stop) => Err(stop),
        }
    }

    pub fn orchestrate_grouped_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedOrchestrationChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_orchestration_checked_on_handle(self, declaration)
    }

    pub fn orchestrate_grouped<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> Result<WorthQueryGroupedOrchestration<D, I>, WorthQueryGroupedOrchestrationStop<D, I>>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        match self.orchestrate_grouped_checked(declaration) {
            WorthQueryGroupedOrchestrationChecked::Bound(value) => Ok(value),
            WorthQueryGroupedOrchestrationChecked::MemberStopped(stop) => {
                Err(WorthQueryGroupedOrchestrationStop::MemberStopped(stop))
            }
            WorthQueryGroupedOrchestrationChecked::WrongWorld(stop) => {
                Err(WorthQueryGroupedOrchestrationStop::WrongWorld(stop))
            }
            WorthQueryGroupedOrchestrationChecked::WrongHandle(stop) => {
                Err(WorthQueryGroupedOrchestrationStop::WrongHandle(stop))
            }
        }
    }

    pub fn orchestrate_grouped_outcome<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryOrdinaryOutcome<WorthQueryGroupedOrchestration<D, I>>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        ordinary_outcome_from_grouped_orchestration_checked(
            self.orchestrate_grouped_checked(declaration),
        )
    }

    pub fn orchestrate_grouped_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedOrchestrationTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_orchestration_proof_on_handle(self, declaration)
    }

    pub fn grouped_route_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedRouteChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_route_checked_on_handle(self, declaration)
    }

    pub fn grouped_route_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedRouteTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_route_proof_on_handle(self, declaration)
    }

    pub fn grouped_receipt_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedReceiptChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_receipt_checked_on_handle(self, declaration)
    }

    pub fn grouped_receipt_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedReceiptTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_receipt_proof_on_handle(self, declaration)
    }

    pub fn grouped_envelope_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedEnvelopeChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_envelope_checked_on_handle(self, declaration)
    }

    pub fn grouped_envelope_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedEnvelopeTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_envelope_proof_on_handle(self, declaration)
    }

    pub fn grouped_contributions_checked<I>(
        &self,
        input: WorthQueryGroupedContributionInput<D, I>,
    ) -> Result<
        WorthQueryGroupedContributionComposition<D, I>,
        WorthQueryGroupedContributionStop<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D> + Clone,
    {
        worth_query_grouped_contribution_checked_on_handle(self, input)
    }

    pub fn grouped_support_report<I>(
        &self,
        declaration: &WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedSupportReport
    where
        I: WorthQueryDeclarationInput<D>,
    {
        let _ = self;
        worth_query_grouped_support_report(declaration)
    }
}
