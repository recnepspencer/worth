use super::UiNativePresentationSource;
use super::{
    FrameProgress, UiNativeApplicationProgramProgress, UiNativePendingProgramFrame,
    UiNativeProgramReconstructionAuthority, UiNativeProgramRetryReadiness,
};
use crate::facade::WorthUiNativeApplicationShell;

impl UiNativeApplicationProgramProgress {
    pub(in crate::native_platform::application_driver) fn retain_or_attribute(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        outcome: crate::mounting::UiMountedFrameOutcome,
        source: UiNativePresentationSource,
        physical_presentation: Option<
            worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
        >,
        reconstruction_authority: Option<UiNativeProgramReconstructionAuthority>,
        cancel_after_external_submission: bool,
    ) -> Result<FrameProgress, ()> {
        let outcome = apply_completion_intent(shell, outcome, cancel_after_external_submission)?;
        match outcome {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending.push_back(UiNativePendingProgramFrame {
                    source,
                    presentation: in_flight,
                    reconstruction_authority,
                    cancel_after_external_submission,
                });
                Ok(FrameProgress::Retained)
            }
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
                match rejection_requirements(&rejected) {
                    UiNativeRejectedFrameRequirements::Retry(readiness) => {
                        self.retain_retry(
                            source,
                            rejected,
                            reconstruction_authority,
                            cancel_after_external_submission,
                            readiness,
                        )?;
                        return Ok(FrameProgress::RetryRequired(readiness));
                    }
                    UiNativeRejectedFrameRequirements::Reconstruct => {
                        return self.reconstruct_for_owner(shell, source, reconstruction_authority);
                    }
                    UiNativeRejectedFrameRequirements::Terminal => {}
                }
                Ok(FrameProgress::Failed)
            }
            crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(indeterminate) => {
                self.settle_indeterminate(shell, indeterminate, source, physical_presentation)
            }
            crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
                self.settle_attribution(shell, source, reconstruction_authority.is_some())
            }
            crate::mounting::UiMountedFrameOutcome::Superseded(_) => Ok(FrameProgress::Settled),
            crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
                Ok(FrameProgress::Failed)
            }
        }
    }

    fn reconstruct_for_owner(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        source: UiNativePresentationSource,
        reconstruction_authority: Option<UiNativeProgramReconstructionAuthority>,
    ) -> Result<FrameProgress, ()> {
        self.next_completion_tick = self.next_completion_tick.saturating_add(1);
        let reconstruction = shell
            .reconstruct_current_presentation(u64::MAX, self.next_completion_tick)
            .map_err(|_| ())?;
        if owner_reconstruction_required(&reconstruction) {
            return Ok(FrameProgress::Failed);
        }
        self.retain_or_attribute(
            shell,
            reconstruction,
            source,
            None,
            Some(
                reconstruction_authority
                    .unwrap_or(UiNativeProgramReconstructionAuthority::HostRequired),
            ),
            false,
        )
    }

    fn settle_indeterminate(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        indeterminate: crate::mounting::UiMountedIndeterminateFrame,
        source: UiNativePresentationSource,
        physical_presentation: Option<
            worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
        >,
    ) -> Result<FrameProgress, ()> {
        if indeterminate.report().awaits_physical_recovery() {
            if physical_presentation.is_some_and(|presentation| {
                presentation.attempt() != indeterminate.report().attempt()
            }) {
                return Err(());
            }
            if self.physical_recovery.has_pending() && self.recovery_source != Some(source) {
                return Err(());
            }
            self.recovery_source = Some(source);
            for binding in indeterminate.report().physical_recovery_bindings() {
                self.physical_recovery
                    .expect(indeterminate.report().attempt(), *binding)
                    .map_err(|_| ())?;
            }
            if let Some(presentation) = physical_presentation.filter(|presentation| {
                indeterminate
                    .report()
                    .physical_recovery_bindings()
                    .contains(&presentation.binding())
            }) {
                self.physical_recovery
                    .observe_scheduled(presentation)
                    .map_err(|_| ())?;
            }
            return Ok(FrameProgress::Settled);
        }
        self.next_completion_tick = self.next_completion_tick.saturating_add(1);
        let recovery = shell
            .reconstruct_current_presentation(u64::MAX, self.next_completion_tick)
            .map_err(|_| ())?;
        if owner_reconstruction_required(&recovery) {
            return Ok(FrameProgress::Failed);
        }
        self.retain_or_attribute(shell, recovery, source, None, None, false)
    }

    fn settle_attribution(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        source: UiNativePresentationSource,
        reconstructed: bool,
    ) -> Result<FrameProgress, ()> {
        let UiNativePresentationSource::Program(source) = source else {
            return Ok(FrameProgress::Settled);
        };
        self.runtime_qualification
            .observe_settled_presentation(shell, reconstructed)?;
        if self.program.frames()[source].captures_presented_source_pixels() {
            if self.visual_snapshot.is_some() {
                return Err(());
            }
            self.visual_snapshot = Some(super::super::visual_snapshot::capture_presented_source(
                shell,
                &mut self.next_completion_tick,
            )?);
        }
        Ok(FrameProgress::Settled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiNativeRejectedFrameRequirements {
    Retry(UiNativeProgramRetryReadiness),
    Reconstruct,
    Terminal,
}

fn rejection_requirements(
    rejected: &crate::mounting::UiMountedRejectedFrame,
) -> UiNativeRejectedFrameRequirements {
    rejection_requirements_for(
        rejected
            .rejections()
            .iter()
            .map(|rejection| rejection.denial()),
    )
}

fn rejection_requirements_for(
    denials: impl IntoIterator<Item = worth_ui_host_contract::UiHostSurfacePresentationDenial>,
) -> UiNativeRejectedFrameRequirements {
    let mut retry = None;
    let mut reconstruct = false;
    let mut observed = false;
    for denial in denials {
        observed = true;
        match denial {
            worth_ui_host_contract::UiHostSurfacePresentationDenial::ExternalTimeout => {
                retry = Some(dominant_retry(
                    retry,
                    UiNativeProgramRetryReadiness::Timeout,
                ));
            }
            worth_ui_host_contract::UiHostSurfacePresentationDenial::SurfaceOccluded => {
                retry = Some(dominant_retry(
                    retry,
                    UiNativeProgramRetryReadiness::Visibility,
                ));
            }
            worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred => {
                retry = Some(dominant_retry(
                    retry,
                    UiNativeProgramRetryReadiness::TextAtlas,
                ));
            }
            worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired => {
                reconstruct = true;
            }
            worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::ExternalValidationFailed
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::CancelledBeforeEffects
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::UnsupportedPresentationMode(_)
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::UnsupportedEffect(_)
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::Protocol(_)
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::ProtocolChanged
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::CapabilityGenerationChanged
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::CapabilityProfileChanged
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::SurfaceBindingChanged
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::StalePredecessor
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::MalformedProjection
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::DeadlineExpired
            | worth_ui_host_contract::UiHostSurfacePresentationDenial::CapacityExceeded => {
                return UiNativeRejectedFrameRequirements::Terminal;
            }
        }
    }
    if !observed {
        UiNativeRejectedFrameRequirements::Terminal
    } else if reconstruct {
        UiNativeRejectedFrameRequirements::Reconstruct
    } else if let Some(retry) = retry {
        UiNativeRejectedFrameRequirements::Retry(retry)
    } else {
        UiNativeRejectedFrameRequirements::Terminal
    }
}

fn dominant_retry(
    current: Option<UiNativeProgramRetryReadiness>,
    candidate: UiNativeProgramRetryReadiness,
) -> UiNativeProgramRetryReadiness {
    current.map_or(candidate, |current| {
        if retry_rank(candidate) > retry_rank(current) {
            candidate
        } else {
            current
        }
    })
}

const fn retry_rank(readiness: UiNativeProgramRetryReadiness) -> u8 {
    match readiness {
        UiNativeProgramRetryReadiness::Timeout => 1,
        UiNativeProgramRetryReadiness::TextAtlas => 2,
        UiNativeProgramRetryReadiness::Visibility => 3,
    }
}

fn owner_reconstruction_required(outcome: &crate::mounting::UiMountedFrameOutcome) -> bool {
    matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected)
            if rejected.rejections().iter().all(|rejection| {
                rejection.denial()
                    == worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired
            })
    )
}

fn apply_completion_intent(
    shell: &mut WorthUiNativeApplicationShell,
    outcome: crate::mounting::UiMountedFrameOutcome,
    cancel_after_external_submission: bool,
) -> Result<crate::mounting::UiMountedFrameOutcome, ()> {
    if !cancel_after_external_submission {
        return Ok(outcome);
    }
    match outcome {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight)
            if in_flight.awaits_progress_class(
                worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface,
            ) =>
        {
            Ok(shell.cancel_mounted_presentation(in_flight))
        }
        outcome @ crate::mounting::UiMountedFrameOutcome::InFlight(_) => Ok(outcome),
        crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_)
        | crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
        | crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_)
        | crate::mounting::UiMountedFrameOutcome::Superseded(_)
        | crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)
        | crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_)
        | crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        rejection_requirements_for, UiNativeProgramRetryReadiness,
        UiNativeRejectedFrameRequirements,
    };
    use worth_ui_host_contract::UiHostSurfacePresentationDenial::{
        ExternalTimeout, ReconstructionRequired, SurfaceOccluded,
    };

    #[test]
    fn reconstruction_dominates_retryable_multi_binding_denials_in_both_surface_orders() {
        for denials in [
            [ExternalTimeout, ReconstructionRequired],
            [ReconstructionRequired, ExternalTimeout],
            [SurfaceOccluded, ReconstructionRequired],
            [ReconstructionRequired, SurfaceOccluded],
        ] {
            assert_eq!(
                rejection_requirements_for(denials),
                UiNativeRejectedFrameRequirements::Reconstruct
            );
        }
    }

    #[test]
    fn visibility_dominates_timeout_when_no_binding_requires_reconstruction() {
        assert_eq!(
            rejection_requirements_for([ExternalTimeout, SurfaceOccluded]),
            UiNativeRejectedFrameRequirements::Retry(UiNativeProgramRetryReadiness::Visibility)
        );
        assert_eq!(
            rejection_requirements_for([SurfaceOccluded, ExternalTimeout]),
            UiNativeRejectedFrameRequirements::Retry(UiNativeProgramRetryReadiness::Visibility)
        );
    }
}
