use super::UiNativePresentationSource;
use super::*;

impl UiNativeApplicationProgramProgress {
    pub(super) fn admit_prepared_superseding_pair(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), ()> {
        let predecessor_index = self.next_frame;
        let successor_index = predecessor_index.checked_add(1).ok_or(())?;
        if !self.program.frames()[successor_index].starts_by_superseding_pending() {
            return Err(());
        }

        let predecessor = match self.staged_superseding_predecessor.take() {
            Some(predecessor) => predecessor,
            None => {
                if !self.apply_staged_frame_changes(shell, predecessor_index)? {
                    return Ok(());
                }
                layout::complete_program_layout(shell)?;
                shell.prepare_frame().map_err(|_| ())?
            }
        };
        if !self.apply_staged_frame_changes(shell, successor_index)? {
            self.staged_superseding_predecessor = Some(predecessor);
            return Ok(());
        }
        layout::complete_program_layout(shell)?;
        let successor = shell
            .prepare_superseding_frame(&predecessor)
            .map_err(|_| ())?;

        self.present_staged_frame(shell, predecessor_index, predecessor, None)?;
        self.staged_superseding_successor = Some(UiNativeStagedSupersedingSuccessor {
            program_frame: successor_index,
            frame: successor,
        });
        Ok(())
    }

    pub(super) fn present_staged_superseding_successor(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), ()> {
        let staged = self.staged_superseding_successor.take().ok_or(())?;
        let successor_index = staged.program_frame;
        let predecessor = self
            .pending
            .iter()
            .find(|pending| pending.source == UiNativePresentationSource::Program(self.next_frame))
            .map(|pending| pending.presentation.superseding_basis())
            .ok_or(())?;
        self.present_staged_frame(shell, successor_index, staged.frame, Some(predecessor))?;
        self.next_frame = successor_index.checked_add(1).ok_or(())?;
        Ok(())
    }

    fn apply_staged_frame_changes(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        frame_index: usize,
    ) -> Result<bool, ()> {
        if frame_index != self.next_change_frame {
            return Err(());
        }
        let frame = &self.program.frames()[frame_index];
        let presence = shell
            .apply_component_presence(frame.component_presence())
            .map_err(|_| ())?;
        if presence
            == crate::facade::entry::UiNativeComponentPresenceProgress::AwaitingPortalDismissal
        {
            return Ok(false);
        }
        shell
            .apply_component_semantic_text(frame.semantic_text())
            .map_err(|_| ())?;
        shell
            .apply_theme_token_values(frame.theme_values())
            .map_err(|_| ())?;
        self.next_change_frame = self.next_change_frame.checked_add(1).ok_or(())?;
        Ok(true)
    }

    fn present_staged_frame(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        frame_index: usize,
        frame: crate::mounting::UiPreparedMountedFrame,
        predecessor: Option<crate::mounting::UiMountedSupersedingPresentationBasis>,
    ) -> Result<(), ()> {
        let tick = self.next_present_tick;
        self.next_present_tick = self.next_present_tick.checked_add(1).ok_or(())?;
        let program_frame = &self.program.frames()[frame_index];
        let outcome = match predecessor {
            Some(predecessor) => {
                shell.present_prepared_superseding_frame(frame, predecessor, u64::MAX, tick)
            }
            None => shell
                .present_prepared_frame(frame, u64::MAX, tick)
                .map_err(|_| ())?,
        };
        let successor = predecessor.is_some();
        let progress = self.retain_or_attribute(
            shell,
            outcome,
            UiNativePresentationSource::Program(frame_index),
            None,
            None,
            program_frame.cancels_after_external_submission(),
        )?;
        if successor {
            matches!(progress, FrameProgress::Retained | FrameProgress::Settled)
                .then_some(())
                .ok_or(())
        } else {
            (progress == FrameProgress::Retained)
                .then_some(())
                .ok_or(())
        }
    }
}
