use worth_ui::facade::app::{
    UiMountedFrameOutcome, WorthUiApplicationPreparationDenial, WorthUiMountedFrameExecutionStop,
    WorthUiNativeApplicationShellLaunchDenial, WorthUiNativeApplicationShutdownReceipt,
    WorthUiNativeSourceRebindDenial,
};
use worth_ui::facade::rebind::UiRebindPreparationDenial;
use worth_ui::facade::source::{
    UiSourceRebindAttemptFailure, WorthUiFilesystemWatcherBackend, WorthUiFilesystemWatcherDenial,
    WorthUiFilesystemWatcherShutdownReceipt,
};

use super::envelope::PlatformPulseLifecycleObservationEnvelope;
use super::launch::PlatformPulseLaunchConfigurationDenial;
use super::lifecycle::{
    PlatformPulseLifecycleObservation, PlatformPulseShutdownCompleted,
    PlatformPulseTerminalFailure, PlatformPulseTerminalFailureFamily,
    PlatformPulseWatcherBackendObservation,
};
use super::projection::{
    PlatformPulseLifecycleObservationProjectionDenial, PlatformPulseLifecycleObservationStream,
    PlatformPulseObservationState,
};

impl PlatformPulseLifecycleObservationStream {
    pub fn project_intent_preparation_failure(
        &mut self,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::IntentPreparation)
    }

    pub fn project_query_preparation_failure(
        &mut self,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::QueryPreparation)
    }

    pub fn project_query_shutdown_failure(
        &mut self,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::QueryShutdown)
    }

    pub fn project_shutdown(
        &mut self,
        watcher: &WorthUiFilesystemWatcherShutdownReceipt,
        query: super::query::PlatformPulseQueryShutdownEvidence,
        intent: super::intent::PlatformPulseIntentWatcherShutdownEvidence,
        application: &WorthUiNativeApplicationShutdownReceipt,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.published_predecessor()?;
        let visual_capture = application.visual_capture();
        let visual_overlay = application.visual_overlay();
        let query_watcher = query.watcher();
        let query_live = query.live();
        let query_projection = query.projection();
        let outcome =
            PlatformPulseLifecycleObservation::ShutdownCompleted(PlatformPulseShutdownCompleted {
                watcher_backend: watcher_backend(watcher.backend()),
                observed_notification_count: watcher.observed_notification_count(),
                query_watcher_joined: query_watcher.worker_joined(),
                pending_query_observation_count: query_watcher.pending_observation_count(),
                intent_watcher_joined: intent.worker_joined(),
                pending_intent_input_count: intent.pending_input_count(),
                intent_resources_empty: application.intent_resources_empty(),
                query_close_complete: application.query_close_complete(),
                query_owner_terminal: query.owner_terminal(),
                live_query_source_count: query_live.source_count(),
                live_query_attempt_count: query_live.attempt_count(),
                live_query_resource_count: query_live.resource_count(),
                live_query_consumer_lease_count: query_live.consumer_lease_count(),
                retained_query_projection_count: query_projection.retained_projection_count(),
                query_projection_receipt_count: query_projection.projection_receipt_count(),
                mounted_shutdown_attempt_count: application.mounted_shutdown_attempt_count() as u64,
                host_session_released: application.host_session_released(),
                released_surface_count: application.released_surface_count() as u64,
                cancelled_visual_capture_count: visual_capture.cancelled_capture_count() as u64,
                disposed_visual_snapshot_count: visual_capture.disposed_snapshot_count() as u64,
                disposed_visual_pixel_bytes: visual_capture.disposed_pixel_bytes(),
                disposed_visual_structural_bytes: visual_capture.disposed_structural_bytes(),
                cancelled_pending_overlay_count: visual_overlay.cancelled_pending_count() as u64,
                disposed_published_overlay_count: visual_overlay.disposed_published_count() as u64,
                disposed_clearing_overlay_count: visual_overlay.disposed_clearing_count() as u64,
            });
        let envelope = self.next_envelope(outcome)?;
        self.state = PlatformPulseObservationState::Terminal;
        Ok(envelope)
    }

    pub fn project_launch_configuration_failure(
        &mut self,
        denial: &PlatformPulseLaunchConfigurationDenial,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::LaunchConfiguration(
            denial.kind(),
        ))
    }

    pub fn project_filesystem_watcher_failure(
        &mut self,
        _denial: &WorthUiFilesystemWatcherDenial,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::FilesystemWatcher)
    }

    pub fn project_application_preparation_failure(
        &mut self,
        _denial: &WorthUiApplicationPreparationDenial,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::ApplicationPreparation)
    }

    pub fn project_candidate_submission_failure(
        &mut self,
        _denial: &UiSourceRebindAttemptFailure,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::CandidateSubmission)
    }

    pub fn project_native_surface_launch_failure(
        &mut self,
        _denial: &WorthUiNativeApplicationShellLaunchDenial,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::NativeSurfaceLaunch)
    }

    pub fn project_frame_execution_failure(
        &mut self,
        _denial: &WorthUiMountedFrameExecutionStop<'_>,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::MountedFrameExecution)
    }

    pub fn project_frame_outcome_failure(
        &mut self,
        outcome: &UiMountedFrameOutcome,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        match outcome {
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | UiMountedFrameOutcome::InFlight(_)
            | UiMountedFrameOutcome::PresentationIndeterminate(_)
            | UiMountedFrameOutcome::RetentionDenied(_)
            | UiMountedFrameOutcome::AdmissionDenied(_)
            | UiMountedFrameOutcome::CompletionDenied(_) => {
                self.project_terminal(PlatformPulseTerminalFailureFamily::MountedFrameExecution)
            }
            UiMountedFrameOutcome::Published(_)
            | UiMountedFrameOutcome::Unchanged(_)
            | UiMountedFrameOutcome::Reconciled(_)
            | UiMountedFrameOutcome::Superseded(_) => {
                Err(PlatformPulseLifecycleObservationProjectionDenial::OutcomeIsNotFailure)
            }
        }
    }

    pub fn project_native_rebind_failure(
        &mut self,
        denial: &WorthUiNativeSourceRebindDenial,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        let stage = match denial {
            WorthUiNativeSourceRebindDenial::Source(_) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::Source
            }
            WorthUiNativeSourceRebindDenial::ObservationTurn(_) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::ObservationTurn
            }
            WorthUiNativeSourceRebindDenial::ObservationAdmission(_) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::ObservationAdmission
            }
            WorthUiNativeSourceRebindDenial::Classification(_) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::Classification
            }
            WorthUiNativeSourceRebindDenial::Scope(_) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::Scope
            }
            WorthUiNativeSourceRebindDenial::Identity(_) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::Identity
            }
            WorthUiNativeSourceRebindDenial::Planning(_) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::Planning
            }
            WorthUiNativeSourceRebindDenial::Preparation(denial) => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::Preparation(
                    project_rebind_preparation_denial(denial),
                )
            }
            WorthUiNativeSourceRebindDenial::ManagedRebindAlreadyInFlight => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::ManagedRebindAlreadyInFlight
            }
            WorthUiNativeSourceRebindDenial::ManagedRebindSessionMismatch => {
                super::lifecycle::PlatformPulseNativeRebindDenialStage::ManagedRebindSessionMismatch
            }
        };
        self.project_terminal(
            PlatformPulseTerminalFailureFamily::NativeApplicationReplacement(stage),
        )
    }

    pub fn project_native_rebind_outcome_failure(
        &mut self,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(
            PlatformPulseTerminalFailureFamily::NativeApplicationReplacement(
                super::lifecycle::PlatformPulseNativeRebindDenialStage::NonterminalOutcome,
            ),
        )
    }

    pub fn project_source_worker_panic(
        &mut self,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::SourceWorkerPanicked)
    }

    pub fn project_visual_identity_failure(
        &mut self,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::VisualIdentity)
    }

    pub fn project_native_event_loop_failure(
        &mut self,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        self.project_terminal(PlatformPulseTerminalFailureFamily::NativeEventLoop)
    }

    fn project_terminal(
        &mut self,
        family: PlatformPulseTerminalFailureFamily,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        if matches!(self.state, PlatformPulseObservationState::Terminal) {
            return Err(PlatformPulseLifecycleObservationProjectionDenial::StreamTerminated);
        }
        let envelope = self.next_envelope(PlatformPulseLifecycleObservation::TerminalFailure(
            PlatformPulseTerminalFailure::new(family),
        ))?;
        self.state = PlatformPulseObservationState::Terminal;
        Ok(envelope)
    }
}

fn project_rebind_preparation_denial(
    denial: &UiRebindPreparationDenial,
) -> super::lifecycle::PlatformPulseNativeRebindPreparationDenial {
    use super::lifecycle::PlatformPulseNativeRebindPreparationDenial as Projected;

    match denial {
        UiRebindPreparationDenial::ForeignSession => Projected::ForeignSession,
        UiRebindPreparationDenial::StaleSourceBasis => Projected::StaleSourceBasis,
        UiRebindPreparationDenial::StalePredecessorGeneration => {
            Projected::StalePredecessorGeneration
        }
        UiRebindPreparationDenial::CandidateGenerationMismatch => {
            Projected::CandidateGenerationMismatch
        }
        UiRebindPreparationDenial::TimedOutBeforeEffects => Projected::TimedOutBeforeEffects,
        UiRebindPreparationDenial::CancelledBeforeEffects => Projected::CancelledBeforeEffects,
        UiRebindPreparationDenial::Reservation(_) => Projected::Reservation,
        UiRebindPreparationDenial::CandidateBindingMismatch => Projected::CandidateBindingMismatch,
        UiRebindPreparationDenial::CandidateAllocation => Projected::CandidateAllocation,
        UiRebindPreparationDenial::CandidateLowering => Projected::CandidateLowering,
        UiRebindPreparationDenial::CandidateStaging => Projected::CandidateStaging,
        UiRebindPreparationDenial::FrameBoundaryUnavailable => Projected::FrameBoundaryUnavailable,
        UiRebindPreparationDenial::ContentMountedPreparation(_) => {
            Projected::ContentMountedPreparation
        }
        UiRebindPreparationDenial::CandidateMountedPreparation(_) => {
            Projected::CandidateMountedPreparation
        }
        UiRebindPreparationDenial::CandidateCutoverPreparation => {
            Projected::CandidateCutoverPreparation
        }
        UiRebindPreparationDenial::PlannedChangeBecameSemanticNoOp => {
            Projected::PlannedChangeBecameSemanticNoOp
        }
        UiRebindPreparationDenial::UnsupportedNonSourcePlan => Projected::UnsupportedNonSourcePlan,
        UiRebindPreparationDenial::InvalidSemanticProof => Projected::InvalidSemanticProof,
        UiRebindPreparationDenial::AppearanceThemeSuccession(_) => {
            Projected::CandidateCutoverPreparation
        }
        UiRebindPreparationDenial::AppearanceInspectionSuccession(_) => {
            Projected::CandidateCutoverPreparation
        }
    }
}

fn watcher_backend(
    backend: WorthUiFilesystemWatcherBackend,
) -> PlatformPulseWatcherBackendObservation {
    match backend {
        WorthUiFilesystemWatcherBackend::Fsevent => PlatformPulseWatcherBackendObservation::Fsevent,
        WorthUiFilesystemWatcherBackend::Inotify => PlatformPulseWatcherBackendObservation::Inotify,
        WorthUiFilesystemWatcherBackend::Kqueue => PlatformPulseWatcherBackendObservation::Kqueue,
        WorthUiFilesystemWatcherBackend::ReadDirectoryChanges => {
            PlatformPulseWatcherBackendObservation::ReadDirectoryChanges
        }
        WorthUiFilesystemWatcherBackend::OtherNative => {
            PlatformPulseWatcherBackendObservation::OtherNative
        }
    }
}

#[cfg(test)]
mod tests {
    use super::project_rebind_preparation_denial;
    use crate::observation_contract::PlatformPulseNativeRebindPreparationDenial;
    use worth_ui::facade::{
        appearance::UiAppearanceInspectionGenerationSuccessionDenial,
        rebind::UiRebindPreparationDenial,
    };

    #[test]
    fn terminal_projection_preserves_the_exact_preparation_denial() {
        assert_eq!(
            project_rebind_preparation_denial(&UiRebindPreparationDenial::CandidateAllocation),
            PlatformPulseNativeRebindPreparationDenial::CandidateAllocation
        );
    }

    #[test]
    fn terminal_projection_groups_appearance_inspection_succession_under_candidate_cutover() {
        assert_eq!(
            project_rebind_preparation_denial(
                &UiRebindPreparationDenial::AppearanceInspectionSuccession(
                    UiAppearanceInspectionGenerationSuccessionDenial::StaleInspectionGeneration,
                ),
            ),
            PlatformPulseNativeRebindPreparationDenial::CandidateCutoverPreparation
        );
    }
}
