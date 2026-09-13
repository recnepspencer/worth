use super::{
    rebind::{normalize_rebind, PlatformPulseRebindAction},
    PlatformPulseApplicationRuntime, PlatformPulseTerminalError,
};
use crate::source_watch::{PlatformPulseSourceEvent, PlatformPulseSourceWatch};

impl PlatformPulseApplicationRuntime {
    pub(super) fn poll_source(&mut self) {
        while source_poll_admitted(
            self.terminal_error.is_some(),
            self.pending_managed_rebind.is_some(),
        ) {
            if !self
                .source_watch
                .as_mut()
                .is_some_and(PlatformPulseSourceWatch::has_pending)
            {
                return;
            }
            let Some(shell) = self.shell.as_mut() else {
                return;
            };
            match self.visual_identity.prepare_source_rebind(
                shell,
                self.presentation_tick,
                std::time::Instant::now(),
            ) {
                Ok(true) => {}
                Ok(false) => return,
                Err(denial) => {
                    self.fail_visual_identity(denial);
                    return;
                }
            }
            let Some(event) = self
                .source_watch
                .as_mut()
                .and_then(PlatformPulseSourceWatch::try_next)
            else {
                return;
            };
            match event {
                PlatformPulseSourceEvent::Settled(snapshot) => self.replace_from(snapshot),
                PlatformPulseSourceEvent::Failed(denial) => {
                    let observation = self.publisher.filesystem_watcher_failure(&denial);
                    self.fail(
                        PlatformPulseTerminalError::SourceWatcher(denial),
                        observation,
                    );
                }
            }
        }
    }

    fn replace_from(
        &mut self,
        snapshot: Box<worth_ui::facade::source::WorthUiSettledSourceSnapshot>,
    ) {
        let Some(mut shell) = self.shell.take() else {
            return;
        };
        let source = snapshot.source_revision().clone();
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let deadline = self.presentation_tick.saturating_add(1);
        let action = normalize_rebind(&mut shell, *snapshot, deadline, self.presentation_tick);
        self.shell = Some(shell);
        match action {
            PlatformPulseRebindAction::SourceDenied(denial) => {
                let failure = denial
                    .source_failure()
                    .expect("source-denied action retains exact source failure");
                let observation = self.publisher.preserved_predecessor(&source, failure);
                if let Err(error) = observation {
                    self.fail(
                        PlatformPulseTerminalError::ObservationPublication,
                        Err(error),
                    );
                } else {
                    eprintln!(
                        "WORTH UI platform pulse kept its predecessor after source denial: {failure:?}"
                    );
                }
            }
            PlatformPulseRebindAction::Published(receipt) => {
                let shell = self
                    .shell
                    .as_mut()
                    .expect("normalized rebind restores the native shell");
                publish_source_rebind(
                    &self.publisher,
                    &mut self.visual_identity,
                    shell,
                    source,
                    receipt,
                    self.presentation_tick,
                )
                .unwrap_or_else(|failure| match failure {
                    SourceRebindPublicationFailure::Observation(error) => self.fail(
                        PlatformPulseTerminalError::ObservationPublication,
                        Err(error),
                    ),
                    SourceRebindPublicationFailure::Visual(denial) => {
                        self.fail_visual_identity(denial)
                    }
                });
            }
            PlatformPulseRebindAction::Pending => {
                self.pending_managed_rebind =
                    Some(super::PlatformPulsePendingManagedRebind::Source(source));
            }
            PlatformPulseRebindAction::Stopped(
                worth_ui::facade::app::WorthUiNativeManagedRebindStop::Duplicate
                | worth_ui::facade::app::WorthUiNativeManagedRebindStop::ObservedNoChange,
            ) => {}
            PlatformPulseRebindAction::Stopped(stop) => {
                self.fail(
                    PlatformPulseTerminalError::NativeManagedSourceRebind(stop),
                    self.publisher.native_rebind_outcome_failure(),
                );
            }
            PlatformPulseRebindAction::Denied(denial) => {
                let observation = self.publisher.native_rebind_failure(&denial);
                self.fail(
                    PlatformPulseTerminalError::NativeRebind(denial),
                    observation,
                );
            }
        }
    }

    pub(super) fn settle_pending_source_rebind(
        &mut self,
        shell: &mut worth_ui::facade::app::WorthUiNativeApplicationShell,
        source: worth_ui::facade::source::WorthUiSourcePackageRevision,
        receipt: worth_ui::facade::rebind::UiRebindReceipt,
    ) {
        if let Err(failure) = publish_source_rebind(
            &self.publisher,
            &mut self.visual_identity,
            shell,
            source,
            receipt,
            self.presentation_tick,
        ) {
            match failure {
                SourceRebindPublicationFailure::Observation(error) => self.fail(
                    PlatformPulseTerminalError::ObservationPublication,
                    Err(error),
                ),
                SourceRebindPublicationFailure::Visual(denial) => self.fail_visual_identity(denial),
            }
        }
    }
}

const fn source_poll_admitted(terminal: bool, managed_rebind_pending: bool) -> bool {
    !terminal && !managed_rebind_pending
}

enum SourceRebindPublicationFailure {
    Observation(
        crate::lifecycle_observation_publication::PlatformPulseObservationPublicationDenial,
    ),
    Visual(crate::visual_identity_execution::PlatformPulseVisualExecutionDenial),
}

fn publish_source_rebind(
    publisher: &crate::lifecycle_observation_publication::PlatformPulseObservationPublisher,
    visual_identity: &mut crate::visual_identity_execution::PlatformPulseVisualIdentityExecution,
    shell: &mut worth_ui::facade::app::WorthUiNativeApplicationShell,
    source: worth_ui::facade::source::WorthUiSourcePackageRevision,
    receipt: worth_ui::facade::rebind::UiRebindReceipt,
    presentation_tick: u64,
) -> Result<(), SourceRebindPublicationFailure> {
    let latest_focus_transition = shell.why_focus_moved().filter(|summary| {
        matches!(
            summary.cause(),
            worth_ui::facade::inspection::UiFocusMoveInspectionCause::RebindPreserved
                | worth_ui::facade::inspection::UiFocusMoveInspectionCause::RebindFallback
        )
    });
    publisher
        .replacement(&source, &receipt, latest_focus_transition)
        .map_err(SourceRebindPublicationFailure::Observation)?;
    visual_identity
        .compare_after_rebind(shell, receipt, presentation_tick, std::time::Instant::now())
        .map_err(SourceRebindPublicationFailure::Visual)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn queued_source_events_wait_behind_managed_rebind_authority() {
        assert!(super::source_poll_admitted(false, false));
        assert!(!super::source_poll_admitted(false, true));
        assert!(!super::source_poll_admitted(true, false));
    }
}
