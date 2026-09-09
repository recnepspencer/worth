use std::collections::VecDeque;

use super::physical_recovery_tracker::{
    UiNativePhysicalRecoverySettlement, UiNativePhysicalRecoveryTracker,
};
use super::program_reconstruction::{is_text_atlas_deferred, retry_text_atlas_deferred};
use crate::facade::WorthUiNativeApplicationShell;

#[path = "program_progress/physical_progress.rs"]
mod physical_progress;
#[path = "program_progress/pointer_refresh.rs"]
mod pointer_refresh;
#[path = "program_progress/presentation_outcome.rs"]
mod presentation_outcome;
#[path = "program_progress/superseding_pair.rs"]
mod superseding_pair;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiNativePresentationSource {
    Program(usize),
    PointerRefresh,
}

pub(super) struct UiNativeApplicationProgramProgress {
    program: crate::facade::entry::UiNativeApplicationProgram,
    pub(super) next_frame: usize,
    next_change_frame: usize,
    next_present_tick: u64,
    pub(super) pending: VecDeque<UiNativePendingProgramFrame>,
    pub(super) next_completion_tick: u64,
    pub(super) recovery_source: Option<UiNativePresentationSource>,
    pub(super) physical_recovery: UiNativePhysicalRecoveryTracker,
    pub(super) pending_retry: Option<UiNativePendingProgramRetry>,
    pub(super) readiness_generation: u64,
    surface_basis_generation: u64,
    surface_basis_barrier: Option<(usize, u64)>,
    runtime_qualification: super::runtime_qualification::UiNativeRuntimeQualificationState,
    pub(super) staged_superseding_successor: Option<UiNativeStagedSupersedingSuccessor>,
    pub(super) staged_superseding_predecessor: Option<crate::mounting::UiPreparedMountedFrame>,
    external_close_requested: bool,
    visual_snapshot: Option<worth_ui_host_native::UiNativeClientVisualSnapshotObservation>,
}

pub(super) struct UiNativePendingProgramFrame {
    pub(super) source: UiNativePresentationSource,
    pub(super) presentation: crate::mounting::UiMountedPresentationInFlight,
    pub(super) reconstruction_authority: Option<UiNativeProgramReconstructionAuthority>,
    pub(super) cancel_after_external_submission: bool,
}

pub(super) struct UiNativeStagedSupersedingSuccessor {
    pub(super) program_frame: usize,
    pub(super) frame: crate::mounting::UiPreparedMountedFrame,
}

pub(super) struct UiNativePendingProgramRetry {
    pub(super) source: UiNativePresentationSource,
    pub(super) rejected: crate::mounting::UiMountedRejectedFrame,
    pub(super) reconstruction_authority: Option<UiNativeProgramReconstructionAuthority>,
    pub(super) cancel_after_external_submission: bool,
    pub(super) after_readiness_generation: u64,
    pub(super) readiness: UiNativeProgramRetryReadiness,
}

impl UiNativePendingProgramRetry {
    fn admits_readiness(&self, generation: u64) -> bool {
        generation > self.after_readiness_generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiNativeProgramReconstructionAuthority {
    Physical(worth_ui_host_native::UiNativePhysicalPresentationCorrelation),
    HostRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiNativeProgramRetryReadiness {
    Timeout,
    Visibility,
    TextAtlas,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum FrameProgress {
    Retained,
    Settled,
    RetryRequired(UiNativeProgramRetryReadiness),
    Failed,
}

impl UiNativeApplicationProgramProgress {
    pub(super) fn new(
        program: crate::facade::entry::UiNativeApplicationProgram,
        runtime_qualification: Option<
            super::super::runtime_qualification::UiNativeRuntimeQualificationPlan,
        >,
    ) -> Self {
        Self {
            program,
            next_frame: 0,
            next_change_frame: 0,
            next_present_tick: 1,
            pending: VecDeque::new(),
            next_completion_tick: 1,
            recovery_source: None,
            physical_recovery: UiNativePhysicalRecoveryTracker::default(),
            pending_retry: None,
            readiness_generation: 0,
            surface_basis_generation: 0,
            surface_basis_barrier: None,
            runtime_qualification:
                super::runtime_qualification::UiNativeRuntimeQualificationState::new(
                    runtime_qualification,
                ),
            staged_superseding_successor: None,
            staged_superseding_predecessor: None,
            external_close_requested: false,
            visual_snapshot: None,
        }
    }

    pub(super) fn observe_readiness(&mut self, generation: u64, surface_basis_generation: u64) {
        self.readiness_generation = self.readiness_generation.max(generation);
        self.surface_basis_generation = self.surface_basis_generation.max(surface_basis_generation);
    }

    pub(super) fn should_close(&self) -> bool {
        if self.external_close_requested {
            return true;
        }
        let program_finished =
            self.program.closes_after_program() && self.next_frame >= self.program.frames().len();
        program_finished
            && self.pending.is_empty()
            && self.staged_superseding_successor.is_none()
            && self.staged_superseding_predecessor.is_none()
            && self.pending_retry.is_none()
            && self.physical_recovery.is_empty()
    }

    pub(super) fn request_external_close(&mut self) {
        self.external_close_requested = true;
        self.staged_superseding_successor = None;
        self.staged_superseding_predecessor = None;
        self.pending_retry = None;
    }

    pub(super) fn take_visual_snapshot(
        &mut self,
    ) -> Option<worth_ui_host_native::UiNativeClientVisualSnapshotObservation> {
        self.visual_snapshot.take()
    }

    pub(super) fn advance(&mut self, shell: &mut WorthUiNativeApplicationShell) -> Result<(), ()> {
        if self.external_close_requested {
            return Ok(());
        }
        if !self.progress_pending_retry(shell)? {
            return Ok(());
        }
        while self.next_frame < self.program.frames().len() {
            if self.physical_recovery.has_pending() {
                break;
            }
            if self.pending.is_empty()
                && self.staged_superseding_successor.is_none()
                && self.next_frame.saturating_add(1) < self.program.frames().len()
                && self.program.frames()[self.next_frame.saturating_add(1)]
                    .starts_by_superseding_pending()
            {
                return self.admit_prepared_superseding_pair(shell);
            }
            if !self.pending.is_empty() {
                let starts_by_superseding =
                    self.program.frames()[self.next_frame].starts_by_superseding_pending();
                let may_supersede = starts_by_superseding
                    && self.pending.iter().all(|pending| {
                        matches!(pending.source, UiNativePresentationSource::Program(_))
                            && pending.presentation.awaits_progress_class(
                        worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface,
                    )
                    });
                if !may_supersede {
                    break;
                }
            }
            let frame = &self.program.frames()[self.next_frame];
            let program_frame = self.next_frame;
            if frame.awaits_host_surface_basis_successor() {
                match self.surface_basis_barrier {
                    Some((barrier_frame, predecessor_generation))
                        if barrier_frame == program_frame
                            && self.surface_basis_generation > predecessor_generation =>
                    {
                        self.surface_basis_barrier = None;
                    }
                    Some((barrier_frame, _)) if barrier_frame == program_frame => break,
                    None => {
                        self.surface_basis_barrier =
                            Some((program_frame, self.surface_basis_generation));
                        break;
                    }
                    Some(_) => return Err(()),
                }
            }
            if program_frame == self.next_change_frame {
                let presence = shell
                    .apply_component_presence(frame.component_presence())
                    .map_err(|_| ())?;
                if presence
                    == crate::facade::entry::UiNativeComponentPresenceProgress::AwaitingPortalDismissal
                {
                    break;
                }
                shell
                    .apply_component_semantic_text(frame.semantic_text())
                    .map_err(|_| ())?;
                shell
                    .apply_theme_token_values(frame.theme_values())
                    .map_err(|_| ())?;
                self.next_change_frame = self.next_change_frame.saturating_add(1);
            } else if program_frame > self.next_change_frame {
                return Err(());
            }
            let tick = self.next_present_tick;
            self.next_present_tick = self.next_present_tick.checked_add(1).ok_or(())?;
            let reconstruction = self.runtime_qualification.reconstruction_required();
            let outcome = if reconstruction {
                shell.reconstruct_current_presentation(u64::MAX, tick)?
            } else {
                shell.present_frame(u64::MAX, tick).map_err(|_| ())?
            };
            let reconstruction_authority =
                reconstruction.then_some(UiNativeProgramReconstructionAuthority::HostRequired);
            let progress = self.retain_or_attribute(
                shell,
                outcome,
                UiNativePresentationSource::Program(program_frame),
                None,
                reconstruction_authority,
                frame.cancels_after_external_submission(),
            )?;
            match progress {
                FrameProgress::Retained | FrameProgress::Settled => {
                    self.next_frame = self.next_frame.saturating_add(1);
                }
                FrameProgress::RetryRequired(_) => break,
                FrameProgress::Failed => return Err(()),
            }
        }
        Ok(())
    }

    pub(super) const fn retry_readiness(&self) -> Option<UiNativeProgramRetryReadiness> {
        match &self.pending_retry {
            Some(pending) => Some(pending.readiness),
            None => None,
        }
    }

    pub(super) fn retain_retry(
        &mut self,
        source: UiNativePresentationSource,
        rejected: crate::mounting::UiMountedRejectedFrame,
        reconstruction_authority: Option<UiNativeProgramReconstructionAuthority>,
        cancel_after_external_submission: bool,
        readiness: UiNativeProgramRetryReadiness,
    ) -> Result<(), ()> {
        if self.pending_retry.is_some() {
            return Err(());
        }
        self.pending_retry = Some(UiNativePendingProgramRetry {
            source,
            rejected,
            reconstruction_authority,
            cancel_after_external_submission,
            after_readiness_generation: self.readiness_generation,
            readiness,
        });
        Ok(())
    }

    fn progress_pending_retry(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<bool, ()> {
        let Some(pending) = self.pending_retry.as_ref() else {
            return Ok(true);
        };
        if !pending.admits_readiness(self.readiness_generation) {
            return Ok(false);
        }
        let pending = self.pending_retry.take().expect("observed pending retry");
        self.next_completion_tick = self.next_completion_tick.checked_add(1).ok_or(())?;
        let outcome = shell.retry_rejected_frame_presentation(
            pending.rejected,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
            self.next_completion_tick,
        );
        let progress = self.retain_or_attribute(
            shell,
            outcome,
            pending.source,
            None,
            pending.reconstruction_authority,
            pending.cancel_after_external_submission,
        )?;
        match progress {
            FrameProgress::Retained => {
                if pending.source == UiNativePresentationSource::Program(self.next_frame) {
                    self.next_frame = self.next_frame.saturating_add(1);
                }
                Ok(false)
            }
            FrameProgress::Settled => {
                if let Some(UiNativeProgramReconstructionAuthority::Physical(correlation)) =
                    pending.reconstruction_authority
                {
                    self.settle_physical_reconstruction(correlation)?;
                }
                if pending.source == UiNativePresentationSource::Program(self.next_frame) {
                    self.next_frame = self.next_frame.saturating_add(1);
                }
                Ok(true)
            }
            FrameProgress::RetryRequired(_) => Ok(false),
            FrameProgress::Failed => Err(()),
        }
    }

    pub(super) fn progress_text_atlas_retry(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), ()> {
        if self.retry_readiness() != Some(UiNativeProgramRetryReadiness::TextAtlas) {
            return Err(());
        }
        let predecessor_generation = self.readiness_generation;
        self.readiness_generation = self.readiness_generation.checked_add(1).ok_or(())?;
        let progressed = self.progress_pending_retry(shell).map(|_| ());
        if progressed.is_err() {
            self.readiness_generation = predecessor_generation;
        }
        progressed
    }
}
