use super::super::UiNativePresentationSource;
use super::{
    super::{
        FrameProgress, UiNativeApplicationProgramProgress, UiNativeProgramReconstructionAuthority,
        WorthUiNativeApplicationShell,
    },
    pending_completion::CompletedPhysicalProgramFrame,
};

impl UiNativeApplicationProgramProgress {
    pub(super) fn settle_completed_physical_progress(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        completed: CompletedPhysicalProgramFrame,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
        duplicate_presentation_observed: bool,
    ) -> Result<(), ()> {
        self.settle_staged_successor(shell, &completed)?;
        settle_duplicate_observation(
            shell,
            completed.progress,
            presentation,
            duplicate_presentation_observed,
        )?;
        if let Some(reconstruction) = completed.reconstruction_authority {
            return self.settle_reconstruction_progress(shell, completed, reconstruction);
        }
        self.settle_ordinary_progress(shell, completed)
    }

    fn settle_staged_successor(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        completed: &CompletedPhysicalProgramFrame,
    ) -> Result<(), ()> {
        if self.staged_superseding_successor.is_none() {
            return Ok(());
        }
        if completed.source != UiNativePresentationSource::Program(self.next_frame)
            || completed.progress != FrameProgress::Retained
        {
            return Err(());
        }
        self.present_staged_superseding_successor(shell)
    }

    fn settle_reconstruction_progress(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        completed: CompletedPhysicalProgramFrame,
        reconstruction: UiNativeProgramReconstructionAuthority,
    ) -> Result<(), ()> {
        match (reconstruction, completed.progress) {
            (_, FrameProgress::Retained) => Ok(()),
            (
                UiNativeProgramReconstructionAuthority::Physical(reconstruction),
                FrameProgress::Settled,
            ) => {
                self.settle_physical_reconstruction(reconstruction)?;
                self.advance(shell)
            }
            (UiNativeProgramReconstructionAuthority::HostRequired, FrameProgress::Settled) => {
                self.advance(shell)
            }
            (_, FrameProgress::RetryRequired(_)) if self.pending_retry.is_some() => Ok(()),
            (_, FrameProgress::RetryRequired(_)) => Err(()),
            (_, FrameProgress::Failed) => Err(()),
        }
    }

    fn settle_ordinary_progress(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        completed: CompletedPhysicalProgramFrame,
    ) -> Result<(), ()> {
        if matches!(completed.progress, FrameProgress::RetryRequired(_)) {
            if let UiNativePresentationSource::Program(index) = completed.source {
                self.next_frame = self.next_frame.min(index);
            }
        } else if completed.progress == FrameProgress::Failed {
            return Err(());
        }
        self.advance(shell)
    }
}

fn settle_duplicate_observation(
    shell: &mut WorthUiNativeApplicationShell,
    progress: FrameProgress,
    presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    duplicate_presentation_observed: bool,
) -> Result<(), ()> {
    if !duplicate_presentation_observed {
        return Ok(());
    }
    let presentation = presentation.ok_or(())?;
    if progress != FrameProgress::Settled {
        return Err(());
    }
    shell
        .admit_duplicate_native_presentation_observation(presentation)
        .map_err(|_| ())
}
