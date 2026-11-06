use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod continuation;
mod grouped;
mod material_attachment;

pub use material_attachment::WorthQueryGeometryMaterialAttachmentInput;

use crate::application::{
    WorthQueryAdmittedDeclarationProgression, WorthQueryDeclarationEntryProgressionError,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};
use crate::contribution_composed_orchestration::{
    WorthQueryContributionComposedOrchestration,
    WorthQueryContributionComposedOrchestrationChecked,
    WorthQueryContributionComposedOrchestrationOutcome,
    WorthQueryContributionComposedOrchestrationTranscript,
};
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;
use crate::signal_compatibility_orchestration::{
    WorthQuerySignalCompatibilityOrchestration, WorthQuerySignalCompatibilityOrchestrationChecked,
    WorthQuerySignalCompatibilityOrchestrationOutcome,
    WorthQuerySignalCompatibilityOrchestrationTranscript,
};

pub trait WorthQueryGeometryActiveFaceSelectionHelperFamily<D: WorthQueryDomainEntryMarker>:
    WorthQueryDeclarationFamilyMarker<D>
{
}

pub trait WorthQueryGeometryMaterialAttachmentHelperFamily<D: WorthQueryDomainEntryMarker>:
    WorthQueryDeclarationFamilyMarker<D>
{
}

pub trait WorthQueryGeometryNeighborhoodHelperFamily<D: WorthQueryDomainEntryMarker>:
    WorthQueryDeclarationFamilyMarker<D>
{
}

pub struct WorthQueryGeometryFamilyHelpers<
    'a,
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
> {
    handle: &'a WorthQueryInstalledDomainDeclarationContext<D, C>,
}

impl<'a, D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryGeometryFamilyHelpers<'a, D, C>
{
    pub(crate) fn new(handle: &'a WorthQueryInstalledDomainDeclarationContext<D, C>) -> Self {
        Self { handle }
    }

    pub fn progress_active_face_selection<I>(
        &self,
        input: I,
    ) -> Result<
        WorthQueryAdmittedDeclarationProgression<D, I>,
        WorthQueryDeclarationEntryProgressionError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        self.handle.declare_review_and_progress(input)
    }

    pub fn prepare_preview_for_active_face_selection<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(continuation::prepare_preview_for_active_face_selection(
            self.handle,
            progression,
        )?)
    }

    pub fn prepare_preview_for_active_face_selection_outcome<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_preview_for_active_face_selection_outcome(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_preview_for_active_face_selection_checked<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_preview_for_active_face_selection_checked(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_preview_for_active_face_selection_proof<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_preview_for_active_face_selection_proof(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_runtime_route_for_active_face_selection<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_runtime_route_for_active_face_selection(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_runtime_route_for_active_face_selection_outcome<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_runtime_route_for_active_face_selection_outcome(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_runtime_route_for_active_face_selection_checked<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_runtime_route_for_active_face_selection_checked(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_runtime_route_for_active_face_selection_proof<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_runtime_route_for_active_face_selection_proof(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_current_truth_view_for_active_face_selection<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_current_truth_view_for_active_face_selection(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_current_truth_view_for_active_face_selection_outcome<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_current_truth_view_for_active_face_selection_outcome(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_current_truth_view_for_active_face_selection_checked<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_current_truth_view_for_active_face_selection_checked(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_current_truth_view_for_active_face_selection_proof<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_current_truth_view_for_active_face_selection_proof(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_historical_truth_view_for_active_face_selection<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_historical_truth_view_for_active_face_selection(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_historical_truth_view_for_active_face_selection_outcome<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_historical_truth_view_for_active_face_selection_outcome(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_historical_truth_view_for_active_face_selection_checked<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_historical_truth_view_for_active_face_selection_checked(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn prepare_historical_truth_view_for_active_face_selection_proof<I>(
        &self,
        progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryActiveFaceSelectionHelperFamily<D>,
    {
        Ok(
            continuation::prepare_historical_truth_view_for_active_face_selection_proof(
                self.handle,
                progression,
            )?,
        )
    }

    pub fn orchestrate_material_attachment_for_active_face_selection<I>(
        &self,
        input: WorthQueryGeometryMaterialAttachmentInput<D, I>,
    ) -> Result<
        WorthQueryContributionComposedOrchestration<D, I>,
        WorthQueryContributionComposedOrchestrationOutcome<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryMaterialAttachmentHelperFamily<D>,
    {
        self.handle
            .orchestrate_declaration_with_contributions(input.into_composed_input())
    }

    pub fn orchestrate_material_attachment_for_active_face_selection_outcome<I>(
        &self,
        input: WorthQueryGeometryMaterialAttachmentInput<D, I>,
    ) -> WorthQueryOrdinaryOutcome<WorthQueryContributionComposedOrchestration<D, I>>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryMaterialAttachmentHelperFamily<D>,
    {
        self.handle
            .orchestrate_declaration_with_contributions_outcome(input.into_composed_input())
    }

    pub fn orchestrate_material_attachment_for_active_face_selection_checked<I>(
        &self,
        input: WorthQueryGeometryMaterialAttachmentInput<D, I>,
    ) -> WorthQueryContributionComposedOrchestrationChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryMaterialAttachmentHelperFamily<D>,
    {
        self.handle
            .orchestrate_declaration_with_contributions_checked(input.into_composed_input())
    }

    pub fn orchestrate_material_attachment_for_active_face_selection_proof<I>(
        &self,
        input: WorthQueryGeometryMaterialAttachmentInput<D, I>,
    ) -> WorthQueryContributionComposedOrchestrationTranscript<D, I>
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryGeometryMaterialAttachmentHelperFamily<D>,
    {
        self.handle
            .orchestrate_declaration_with_contributions_proof(input.into_composed_input())
    }
}
