use crate::application::{
    WorthQueryDeclarationInput, WorthQueryDeclarationSupportsNeighborhoodGrouping,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::grouped_authoring::{
    ordinary_outcome_from_grouped_orchestration_checked,
    worth_query_grouped_declaration_checked_on_handle,
    worth_query_grouped_orchestration_checked_on_handle,
    worth_query_grouped_orchestration_proof_on_handle, WorthQueryGroupedContributionComposition,
    WorthQueryGroupedContributionInput, WorthQueryGroupedContributionStop,
    WorthQueryGroupedDeclarationArtifact, WorthQueryGroupedDeclarationChecked,
    WorthQueryGroupedDeclarationInput, WorthQueryGroupedDeclarationStop,
    WorthQueryGroupedEnvelopeChecked, WorthQueryGroupedEnvelopeTranscript,
    WorthQueryGroupedOrchestration, WorthQueryGroupedOrchestrationChecked,
    WorthQueryGroupedOrchestrationStop, WorthQueryGroupedOrchestrationTranscript,
    WorthQueryGroupedReceiptChecked, WorthQueryGroupedReceiptTranscript,
    WorthQueryGroupedRouteChecked, WorthQueryGroupedRouteTranscript,
};
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;

use super::{WorthQueryGeometryFamilyHelpers, WorthQueryGeometryNeighborhoodHelperFamily};

impl<'a, D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryGeometryFamilyHelpers<'a, D, C>
{
    pub fn local_neighborhood_for_active_face_selection<I>(
        &self,
        seed_member: I,
    ) -> WorthQueryGroupedDeclarationInput<D, I>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        local_neighborhood_for_active_face_selection(seed_member)
    }

    pub fn declare_local_neighborhood_for_active_face_selection<I>(
        &self,
        input: WorthQueryGroupedDeclarationInput<D, I>,
    ) -> Result<WorthQueryGroupedDeclarationArtifact<D, I>, WorthQueryGroupedDeclarationStop>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        declare_local_neighborhood_for_active_face_selection(self.handle, input)
    }

    pub fn declare_local_neighborhood_for_active_face_selection_checked<I>(
        &self,
        input: WorthQueryGroupedDeclarationInput<D, I>,
    ) -> WorthQueryGroupedDeclarationChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        declare_local_neighborhood_for_active_face_selection_checked(self.handle, input)
    }

    pub fn orchestrate_local_neighborhood_for_active_face_selection<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> Result<WorthQueryGroupedOrchestration<D, I>, WorthQueryGroupedOrchestrationStop<D, I>>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        orchestrate_local_neighborhood_for_active_face_selection(self.handle, declaration)
    }

    pub fn orchestrate_local_neighborhood_for_active_face_selection_outcome<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryOrdinaryOutcome<WorthQueryGroupedOrchestration<D, I>>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        orchestrate_local_neighborhood_for_active_face_selection_outcome(self.handle, declaration)
    }

    pub fn orchestrate_local_neighborhood_for_active_face_selection_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedOrchestrationChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        orchestrate_local_neighborhood_for_active_face_selection_checked(self.handle, declaration)
    }

    pub fn orchestrate_local_neighborhood_for_active_face_selection_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedOrchestrationTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        orchestrate_local_neighborhood_for_active_face_selection_proof(self.handle, declaration)
    }

    pub fn grouped_routes_for_active_face_selection_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedRouteChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        self.handle.grouped_route_checked(declaration)
    }

    pub fn grouped_routes_for_active_face_selection_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedRouteTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        self.handle.grouped_route_proof(declaration)
    }

    pub fn grouped_receipt_for_active_face_selection_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedReceiptChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        self.handle.grouped_receipt_checked(declaration)
    }

    pub fn grouped_receipt_for_active_face_selection_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedReceiptTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        self.handle.grouped_receipt_proof(declaration)
    }

    pub fn grouped_envelope_for_active_face_selection_checked<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedEnvelopeChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        self.handle.grouped_envelope_checked(declaration)
    }

    pub fn grouped_envelope_for_active_face_selection_proof<I>(
        &self,
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    ) -> WorthQueryGroupedEnvelopeTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        self.handle.grouped_envelope_proof(declaration)
    }

    pub fn grouped_contributions_for_active_face_selection_checked<I>(
        &self,
        input: WorthQueryGroupedContributionInput<D, I>,
    ) -> Result<
        WorthQueryGroupedContributionComposition<D, I>,
        WorthQueryGroupedContributionStop<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D> + Clone,
        I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
            + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
    {
        self.handle.grouped_contributions_checked(input)
    }
}

pub(super) fn local_neighborhood_for_active_face_selection<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    seed_member: I,
) -> WorthQueryGroupedDeclarationInput<D, I>
where
    I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
        + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
{
    WorthQueryGroupedDeclarationInput::local_neighborhood(seed_member)
}

pub(super) fn declare_local_neighborhood_for_active_face_selection<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    input: WorthQueryGroupedDeclarationInput<D, I>,
) -> Result<WorthQueryGroupedDeclarationArtifact<D, I>, WorthQueryGroupedDeclarationStop>
where
    I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
        + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
{
    match declare_local_neighborhood_for_active_face_selection_checked(handle, input) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => Ok(value),
        WorthQueryGroupedDeclarationChecked::MemberStopped(stop) => Err(stop),
    }
}

pub(super) fn declare_local_neighborhood_for_active_face_selection_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    input: WorthQueryGroupedDeclarationInput<D, I>,
) -> WorthQueryGroupedDeclarationChecked<D, I>
where
    I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
        + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
{
    worth_query_grouped_declaration_checked_on_handle(handle, input)
}

pub(super) fn orchestrate_local_neighborhood_for_active_face_selection<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
) -> Result<WorthQueryGroupedOrchestration<D, I>, WorthQueryGroupedOrchestrationStop<D, I>>
where
    I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
        + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
{
    match orchestrate_local_neighborhood_for_active_face_selection_checked(handle, declaration) {
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

pub(super) fn orchestrate_local_neighborhood_for_active_face_selection_outcome<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
) -> WorthQueryOrdinaryOutcome<WorthQueryGroupedOrchestration<D, I>>
where
    I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
        + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
{
    ordinary_outcome_from_grouped_orchestration_checked(
        orchestrate_local_neighborhood_for_active_face_selection_checked(handle, declaration),
    )
}

pub(super) fn orchestrate_local_neighborhood_for_active_face_selection_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
) -> WorthQueryGroupedOrchestrationChecked<D, I>
where
    I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
        + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
{
    worth_query_grouped_orchestration_checked_on_handle(handle, declaration)
}

pub(super) fn orchestrate_local_neighborhood_for_active_face_selection_proof<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
) -> WorthQueryGroupedOrchestrationTranscript<D, I>
where
    I::Family: WorthQueryGeometryNeighborhoodHelperFamily<D>
        + WorthQueryDeclarationSupportsNeighborhoodGrouping<D>,
{
    worth_query_grouped_orchestration_proof_on_handle(handle, declaration)
}
