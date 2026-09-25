use crate::mounting::{
    UiMountedFrameOutcome, UiMountedFramePreparationDenial, UiMountedFrameRequest,
    UiMountedFrameReuse, UiMountedPublicationLeaseDenial,
};
use crate::runtime::WorthUiFrameworkTurn;
use worth_ui_host_contract::UiPresentationDeadline;

use super::{
    mounted_publication::finish_mounted_transition, WorthUiActiveApplicationSession,
    WorthUiActiveFrameworkTurnCompletion,
};

/// A typed stop before one ordinary mounted-frame request can publish.
pub enum WorthUiMountedFrameExecutionStop<'session> {
    PublicationLease(UiMountedPublicationLeaseDenial),
    HostMeasurement(Box<crate::facade::host::UiHostMeasurementEvidenceDenial>),
    HostMeasurementTransition(Box<super::UiMountedHostMeasurementTransitionDenial>),
    OccurrenceGeometry(crate::mounting::UiMountedOccurrenceGeometryDenial),
    FrameworkTransition(WorthUiMountedFrameFrameworkTransitionStop<'session>),
    Preparation(Box<UiMountedFramePreparationDenial>),
}

/// Opaque framework-transition state retained by an ordinary mounted-frame stop.
///
/// The raw completion remains available to runtime internals, but ordinary callers
/// cannot recover lane-execution authority from a mounted-frame failure.
pub struct WorthUiMountedFrameFrameworkTransitionStop<'session> {
    pub(super) completion: Box<WorthUiActiveFrameworkTurnCompletion<'session>>,
}

pub(crate) struct UiPreparedMountedReconstructionFrame {
    frame: crate::mounting::UiPreparedMountedFrame,
    owner_receipts:
        super::mounted_owner_receipt_succession::UiPreparedMountedOwnerReceiptSuccession,
}

impl std::ops::Deref for UiPreparedMountedReconstructionFrame {
    type Target = crate::mounting::UiPreparedMountedFrame;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl UiPreparedMountedReconstructionFrame {
    pub(super) fn into_parts(
        self,
    ) -> (
        crate::mounting::UiPreparedMountedFrame,
        super::mounted_owner_receipt_succession::UiPreparedMountedOwnerReceiptSuccession,
    ) {
        (self.frame, self.owner_receipts)
    }
}

impl WorthUiMountedFrameFrameworkTransitionStop<'_> {
    pub fn generation_identity(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        self.completion.generation_identity()
    }
}

impl WorthUiActiveApplicationSession {
    /// Executes, assembles, presents, and publishes one ordinary mounted frame.
    pub fn execute_mounted_frame(
        &mut self,
        request: UiMountedFrameRequest,
        deadline: UiPresentationDeadline,
        now: u64,
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> Result<UiMountedFrameOutcome, WorthUiMountedFrameExecutionStop<'_>> {
        let completion = self
            .execute_framework_turn(collect_sources)
            .map_err(WorthUiMountedFrameExecutionStop::PublicationLease)?;
        completion.execute_mounted_frame(request, deadline, now)
    }

    pub(crate) fn prepare_mounted_reconciliation_frame_with_application_presentation(
        &mut self,
        request: UiMountedFrameRequest,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> Result<crate::mounting::UiPreparedMountedFrame, WorthUiMountedFrameExecutionStop<'_>> {
        let projection = self
            .presentation
            .project()
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        let completion = self
            .execute_framework_turn(collect_sources)
            .map_err(WorthUiMountedFrameExecutionStop::PublicationLease)?;
        let mut execution = completion.into_execution().map_err(|completion| {
            WorthUiMountedFrameExecutionStop::FrameworkTransition(
                WorthUiMountedFrameFrameworkTransitionStop { completion },
            )
        })?;
        let frame = execution
            .prepare_mounted_reconciliation_frame_with_content_internal(
                request,
                crate::mounting::UiMountedSemanticContentInput::empty(),
                projection,
                replacements,
            )
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        Ok(frame)
    }

    pub(crate) fn prepare_mounted_reconstruction_frame_with_application_presentation(
        &mut self,
        request: UiMountedFrameRequest,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> Result<UiPreparedMountedReconstructionFrame, WorthUiMountedFrameExecutionStop<'_>> {
        let projection = self
            .presentation
            .project_complete()
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        let completion = self
            .execute_framework_turn(collect_sources)
            .map_err(WorthUiMountedFrameExecutionStop::PublicationLease)?;
        let mut execution = completion.into_execution().map_err(|completion| {
            WorthUiMountedFrameExecutionStop::FrameworkTransition(
                WorthUiMountedFrameFrameworkTransitionStop { completion },
            )
        })?;
        let (frame, owner_receipts) = execution
            .prepare_mounted_reconstruction_frame_with_content_internal(
                request,
                crate::mounting::UiMountedSemanticContentInput::empty(),
                projection,
                replacements,
            )
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        Ok(UiPreparedMountedReconstructionFrame {
            frame,
            owner_receipts,
        })
    }

    pub(crate) fn prepare_mounted_frame_with_application_presentation(
        &mut self,
        request: UiMountedFrameRequest,
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> Result<crate::mounting::UiPreparedMountedFrame, WorthUiMountedFrameExecutionStop<'_>> {
        let projection = self
            .presentation
            .project()
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        let completion = self
            .execute_framework_turn(collect_sources)
            .map_err(WorthUiMountedFrameExecutionStop::PublicationLease)?;
        let mut execution = completion.into_execution().map_err(|completion| {
            WorthUiMountedFrameExecutionStop::FrameworkTransition(
                WorthUiMountedFrameFrameworkTransitionStop { completion },
            )
        })?;
        let frame = execution
            .prepare_mounted_frame_with_content_internal(
                request,
                crate::mounting::UiMountedSemanticContentInput::empty(),
                projection,
            )
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        Ok(frame)
    }

    pub(crate) fn prepare_mounted_superseding_frame_with_application_presentation(
        &mut self,
        request: UiMountedFrameRequest,
        predecessor: &crate::mounting::UiPreparedMountedFrame,
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> Result<crate::mounting::UiPreparedMountedFrame, WorthUiMountedFrameExecutionStop<'_>> {
        let projection = self
            .presentation
            .project()
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        let completion = self
            .execute_framework_turn(collect_sources)
            .map_err(WorthUiMountedFrameExecutionStop::PublicationLease)?;
        let mut execution = completion.into_execution().map_err(|completion| {
            WorthUiMountedFrameExecutionStop::FrameworkTransition(
                WorthUiMountedFrameFrameworkTransitionStop { completion },
            )
        })?;
        let frame = execution
            .prepare_mounted_superseding_frame_with_content_internal(
                request,
                crate::mounting::UiMountedSemanticContentInput::empty(),
                projection,
                predecessor,
            )
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        Ok(frame)
    }
}

impl<'session> WorthUiActiveFrameworkTurnCompletion<'session> {
    fn execute_mounted_frame(
        self,
        request: UiMountedFrameRequest,
        deadline: UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, WorthUiMountedFrameExecutionStop<'session>> {
        let mut execution = self.into_execution().map_err(|completion| {
            WorthUiMountedFrameExecutionStop::FrameworkTransition(
                WorthUiMountedFrameFrameworkTransitionStop { completion },
            )
        })?;

        match execution.classify_mounted_frame_reuse_internal(&request) {
            UiMountedFrameReuse::Exact(witness)
                if !execution.presentation.requires_mounted_projection() =>
            {
                execution
                    .mounted
                    .admit_pointer_reuse_observation(
                        execution.pointer_affordance_snapshot.as_ref(),
                        &request,
                    )
                    .map_err(|denial| {
                        WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial))
                    })?;
                Ok(UiMountedFrameOutcome::Unchanged(
                    witness.publication().clone(),
                ))
            }
            UiMountedFrameReuse::ComparisonRequired(_) | UiMountedFrameReuse::Exact(_) => {
                let presentation = execution.presentation.project().map_err(|denial| {
                    WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial))
                })?;
                let frame = execution
                    .prepare_mounted_frame_with_content_internal(
                        request,
                        crate::mounting::UiMountedSemanticContentInput::empty(),
                        presentation,
                    )
                    .map_err(|denial| {
                        WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial))
                    })?;
                let owner_receipts = execution.prepare_mounted_owner_receipts(&frame);
                let transition =
                    execution.present_prepared_frame_with_appearance(frame, deadline, now);
                let outcome = finish_mounted_transition(
                    &mut *execution.mounted,
                    execution.focus.as_deref_mut(),
                    execution.portal.as_deref_mut(),
                    &mut *execution.interaction,
                    execution.host_session,
                    execution.application_session_identity,
                    &execution.generation_identity,
                    &mut *execution.host_exchange,
                    transition,
                    Some(&mut *execution.appearance_inspection),
                    Some(&mut *execution.presentation),
                    Some(&mut *execution.overlay_composition_owners),
                );
                execution.settle_new_mounted_owner_receipts(owner_receipts, &outcome);
                super::active_application_session::settle_presented_scroll_extent(
                    execution.scroll.as_deref_mut(),
                    execution.motion.as_deref_mut(),
                    execution.mounted,
                    &*execution.interaction,
                    execution.owed_scroll_settles,
                    &outcome,
                    now,
                );
                Ok(outcome)
            }
        }
    }
}
