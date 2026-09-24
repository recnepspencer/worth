use worth_ui::facade::app::{UiMountedFramePublicationReceipt, WorthUiNativeApplicationShell};
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseQueryProjectionEvidence, PlatformPulseQueryProjectionPosture,
};

use super::{
    projection::{begin_projection, PlatformPulseProjectionPublication},
    PlatformPulseApplicationRuntime, PlatformPulseProjectionRebindDenial,
    PlatformPulseTerminalError,
};

pub(super) struct PlatformPulsePendingProjection {
    evidence: PlatformPulseQueryProjectionEvidence,
    first: bool,
}

impl PlatformPulseApplicationRuntime {
    pub(super) fn publish_initial_projection(&mut self) {
        let observation = match self
            .query_lifecycle
            .as_mut()
            .expect("prepared Pulse retains its Query lifecycle")
            .issue_initial()
        {
            Ok(observation) => observation,
            Err(denial) => {
                self.fail(PlatformPulseTerminalError::QueryLifecycle(denial), Ok(()));
                return;
            }
        };
        self.publish_query_observation(observation, self.initial_source.is_some());
    }

    pub(super) fn poll_query(&mut self) {
        while self.terminal.is_running() {
            let Some(event) = self
                .query_watch
                .as_ref()
                .and_then(crate::query_source::PlatformPulseExternalValueWatch::try_next)
            else {
                return;
            };
            match event {
                crate::query_source::PlatformPulseExternalValueEvent::Record(record) => {
                    let observation = match self
                        .query_lifecycle
                        .as_mut()
                        .expect("prepared Pulse retains its Query lifecycle")
                        .advance(record)
                    {
                        Ok(observation) => observation,
                        Err(denial) => {
                            self.fail(PlatformPulseTerminalError::QueryLifecycle(denial), Ok(()));
                            return;
                        }
                    };
                    self.publish_query_observation(observation, false);
                }
                crate::query_source::PlatformPulseExternalValueEvent::Failed(denial) => {
                    self.fail(PlatformPulseTerminalError::QueryWatch(denial), Ok(()));
                }
            }
        }
    }

    fn publish_query_observation(
        &mut self,
        observation: worth_ui::facade::query_binding::UiProjectionObservation,
        first: bool,
    ) {
        let evidence = match self.publisher.query_projection_issued(&observation) {
            Ok(evidence) => evidence,
            Err(error) => {
                self.fail(
                    PlatformPulseTerminalError::ObservationPublication,
                    Err(error),
                );
                return;
            }
        };
        let Some(mut shell) = self.shell.take() else {
            return;
        };
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let publication = match begin_projection(&mut shell, observation, self.presentation_tick) {
            Ok(publication) => publication,
            Err(denial) => {
                self.shell = Some(shell);
                self.fail(PlatformPulseTerminalError::NativeProjection(denial), Ok(()));
                return;
            }
        };
        let receipt = match publication {
            PlatformPulseProjectionPublication::Published(receipt) => receipt,
            PlatformPulseProjectionPublication::Pending => {
                self.pending_managed_rebind =
                    Some(super::PlatformPulsePendingManagedRebind::Projection(
                        PlatformPulsePendingProjection { evidence, first },
                    ));
                self.shell = Some(shell);
                return;
            }
        };
        self.settle_query_projection(&mut shell, evidence, first, receipt);
        self.shell = Some(shell);
    }

    pub(super) fn settle_pending_projection(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        pending: PlatformPulsePendingProjection,
        receipt: worth_ui::facade::rebind::UiRebindReceipt,
    ) {
        self.settle_query_projection(shell, pending.evidence, pending.first, receipt);
    }

    fn settle_query_projection(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        evidence: PlatformPulseQueryProjectionEvidence,
        first: bool,
        receipt: worth_ui::facade::rebind::UiRebindReceipt,
    ) {
        if first && !self.publish_query_first_frame(&receipt) {
            return;
        }
        let Some(mounted) = receipt.mounted_publication() else {
            self.fail(
                PlatformPulseTerminalError::NativeProjection(
                    PlatformPulseProjectionRebindDenial::ReceiptContract(
                        "mounted publication absent from receipt",
                    ),
                ),
                Ok(()),
            );
            return;
        };
        if !self.publish_query_projection_evidence(&evidence, mounted) {
            return;
        }
        if let Err(denial) = self.refresh_query_visual_identity(shell, &evidence) {
            self.fail_visual_identity(denial);
            return;
        }
        self.admit_query_publication(receipt);
    }

    fn publish_query_first_frame(
        &mut self,
        receipt: &worth_ui::facade::rebind::UiRebindReceipt,
    ) -> bool {
        let Some(source) = self.initial_source.take() else {
            self.fail(PlatformPulseTerminalError::UnexpectedInitialFrame, Ok(()));
            return false;
        };
        let Some(mounted) = receipt.mounted_publication() else {
            self.fail(PlatformPulseTerminalError::UnexpectedInitialFrame, Ok(()));
            return false;
        };
        if let Err(error) = self.publish_first_frame(&source, mounted) {
            self.fail(
                PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        true
    }

    pub(super) fn publish_query_projection_evidence(
        &mut self,
        evidence: &PlatformPulseQueryProjectionEvidence,
        mounted: &UiMountedFramePublicationReceipt,
    ) -> bool {
        match self.publisher.query_projection_published(evidence, mounted) {
            Ok(()) => true,
            Err(error) => {
                self.fail(
                    PlatformPulseTerminalError::ObservationPublication,
                    Err(error),
                );
                false
            }
        }
    }

    pub(super) fn refresh_query_visual_identity(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        evidence: &PlatformPulseQueryProjectionEvidence,
    ) -> Result<(), crate::visual_identity_execution::PlatformPulseVisualExecutionDenial> {
        if evidence.posture() != PlatformPulseQueryProjectionPosture::Current {
            return Ok(());
        }
        self.visual_identity
            .refresh_after_presentation_replacement(
                shell,
                self.presentation_tick,
                std::time::Instant::now(),
            )?;
        if evidence.owner_order() == 2 {
            self.visual_identity
                .arm_after_first_frame(std::time::Instant::now())?;
        }
        Ok(())
    }

    fn admit_query_publication(&mut self, receipt: worth_ui::facade::rebind::UiRebindReceipt) {
        let observation = match receipt.release_application_scalar_projection_observation() {
            Ok(observation) => observation,
            Err(_) => {
                self.fail(
                    PlatformPulseTerminalError::NativeProjection(
                        PlatformPulseProjectionRebindDenial::ReceiptContract(
                            "application scalar predecessor absent from receipt",
                        ),
                    ),
                    Ok(()),
                );
                return;
            }
        };
        let admission = self
            .query_lifecycle
            .as_mut()
            .expect("prepared Pulse retains its Query lifecycle")
            .admit_publication(observation);
        if let Err(denial) = admission {
            self.fail(PlatformPulseTerminalError::QueryLifecycle(denial), Ok(()));
        }
    }
}
