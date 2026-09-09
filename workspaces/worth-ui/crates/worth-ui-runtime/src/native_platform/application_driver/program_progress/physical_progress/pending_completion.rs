use super::super::UiNativePresentationSource;
use super::super::{
    is_text_atlas_deferred, retry_text_atlas_deferred, FrameProgress,
    UiNativeApplicationProgramProgress, UiNativePendingProgramFrame,
    UiNativeProgramReconstructionAuthority, WorthUiNativeApplicationShell,
};

pub(super) struct CompletedPhysicalProgramFrame {
    pub(super) source: UiNativePresentationSource,
    pub(super) reconstruction_authority: Option<UiNativeProgramReconstructionAuthority>,
    pub(super) progress: FrameProgress,
}

impl UiNativeApplicationProgramProgress {
    pub(super) fn take_pending_for_physical_progress(
        &mut self,
        class: worth_ui_host_native::UiNativePhysicalProgressClass,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> Result<Option<UiNativePendingProgramFrame>, ()> {
        let index = match class {
            worth_ui_host_native::UiNativePhysicalProgressClass::Presentation => {
                let presentation = presentation.ok_or(())?;
                self.pending.iter().position(|pending| {
                    pending.presentation.awaits_progress_class(
                        worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface,
                    ) && pending.presentation.attempt() == presentation.attempt()
                        && pending
                            .presentation
                            .pending_bindings()
                            .any(|binding| binding == presentation.binding())
                })
            }
            worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas => {
                self.pending.iter().position(|pending| {
                    pending.presentation.awaits_progress_class(
                        worth_ui_host_contract::UiHostPresentationProgressClass::TextAtlas,
                    )
                })
            }
            worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery => {
                unreachable!("presentation recovery is handled before pending polling")
            }
        };
        Ok(index.and_then(|index| self.pending.remove(index)))
    }

    pub(super) fn complete_pending_physical_progress(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        pending: UiNativePendingProgramFrame,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> Result<CompletedPhysicalProgramFrame, ()> {
        let UiNativePendingProgramFrame {
            source,
            presentation: pending_presentation,
            reconstruction_authority,
            cancel_after_external_submission,
        } = pending;
        self.next_completion_tick = self.next_completion_tick.saturating_add(1);
        let deadline = pending_presentation.deadline();
        let outcome =
            shell.complete_frame_presentation(pending_presentation, self.next_completion_tick);
        let outcome = if is_text_atlas_deferred(&outcome)
            && matches!(
                reconstruction_authority,
                Some(UiNativeProgramReconstructionAuthority::HostRequired)
            ) {
            shell.reconstruct_current_presentation(deadline.tick(), self.next_completion_tick)?
        } else {
            retry_text_atlas_deferred(shell, outcome, deadline, self.next_completion_tick)
        };
        let progress = self.retain_or_attribute(
            shell,
            outcome,
            source,
            presentation,
            reconstruction_authority,
            cancel_after_external_submission,
        )?;
        Ok(CompletedPhysicalProgramFrame {
            source,
            reconstruction_authority,
            progress,
        })
    }
}
