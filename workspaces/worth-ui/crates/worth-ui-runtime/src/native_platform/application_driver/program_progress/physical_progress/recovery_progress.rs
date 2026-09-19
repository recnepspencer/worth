use super::super::{UiNativeApplicationProgramProgress, WorthUiNativeApplicationShell};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiNativeRecoveryProgressDirective {
    WaitForRemainingBindings,
    ResumeReconstruction,
}

impl UiNativeApplicationProgramProgress {
    pub(super) fn progress_presentation_recovery(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        presentation: worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
    ) -> Result<(), ()> {
        if progress_recovery_correlation(&mut self.physical_recovery, presentation)?
            == UiNativeRecoveryProgressDirective::WaitForRemainingBindings
        {
            return Ok(());
        }
        let source = self.recovery_source.ok_or(())?;
        self.resume_reconstruction(shell, source, presentation)
    }
}

fn progress_recovery_correlation(
    tracker: &mut super::super::UiNativePhysicalRecoveryTracker,
    presentation: worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
) -> Result<UiNativeRecoveryProgressDirective, ()> {
    tracker
        .observe_scheduled(presentation)
        .or_else(|denial| {
            (denial
                == super::super::super::physical_recovery_tracker::UiNativePhysicalRecoveryTrackingDenial::DuplicateCorrelation)
                .then_some(())
                .ok_or(denial)
        })
        .map_err(|_| ())?;
    match tracker.classify_settlement(presentation).map_err(|_| ())? {
        super::super::UiNativePhysicalRecoverySettlement::AttemptStillPending => {
            tracker.commit_settlement(presentation).map_err(|_| ())?;
            Ok(UiNativeRecoveryProgressDirective::WaitForRemainingBindings)
        }
        super::super::UiNativePhysicalRecoverySettlement::AttemptReady => {
            Ok(UiNativeRecoveryProgressDirective::ResumeReconstruction)
        }
    }
}

#[cfg(all(test, feature = "certification-support"))]
mod tests {
    use super::*;

    #[test]
    fn driver_reconstruction_stays_blocked_until_the_last_binding_settles() {
        let attempt =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let left = correlation(attempt, 1);
        let right = correlation(attempt, 2);
        let mut tracker = super::super::super::UiNativePhysicalRecoveryTracker::default();
        tracker.expect(attempt, left.binding()).unwrap();
        tracker.expect(attempt, right.binding()).unwrap();

        assert_eq!(
            progress_recovery_correlation(&mut tracker, left),
            Ok(UiNativeRecoveryProgressDirective::WaitForRemainingBindings)
        );
        assert_eq!(
            progress_recovery_correlation(&mut tracker, right),
            Ok(UiNativeRecoveryProgressDirective::ResumeReconstruction)
        );
    }

    fn correlation(
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        sequence: u64,
    ) -> worth_ui_host_native::UiNativePhysicalPresentationCorrelation {
        worth_ui_host_native::UiNativePhysicalPresentationCorrelation::from_certification(
            attempt,
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            sequence,
        )
        .unwrap()
    }
}
