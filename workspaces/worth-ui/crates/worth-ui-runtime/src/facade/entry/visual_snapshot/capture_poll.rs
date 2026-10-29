use worth_ui_inspection::{
    UiVisualArtifactPolicy, UiVisualSnapshotDenial, UiVisualSnapshotOmission,
};

use crate::inspection::visual_snapshot::{
    UiPendingVisualCapture, UiVisualCapturePoll, UiVisualSnapshotOutcome, UiVisualTarget,
};

use super::super::WorthUiActiveApplicationSession;

mod captured_artifact;
mod derived_artifact;
mod failure;
mod host_request;

use failure::UiVisualCaptureFailure;

impl WorthUiActiveApplicationSession {
    pub fn poll_visual_snapshot<Target, Policy>(
        &mut self,
        pending: UiPendingVisualCapture<Target, Policy>,
        now_tick: u64,
    ) -> UiVisualCapturePoll<Target, Policy>
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        if deadline_elapsed(&pending, now_tick) {
            return UiVisualCapturePoll::Completed(self.timeout_visual_capture(&pending));
        }
        let pending = match pending.into_route() {
            crate::inspection::visual_snapshot::UiPendingVisualCaptureRoute::Host(pending) => {
                pending
            }
            crate::inspection::visual_snapshot::UiPendingVisualCaptureRoute::DerivedRegion(
                derived,
            ) => {
                let outcome = match derived_artifact::seal_derived_region(derived) {
                    Ok(receipt) => UiVisualSnapshotOutcome::Captured(receipt),
                    Err(failure) => failure.into_outcome(),
                };
                return UiVisualCapturePoll::Completed(outcome);
            }
        };
        let requested = match self.ensure_host_capture_request(pending) {
            Ok(requested) => requested,
            Err(failure) => {
                return UiVisualCapturePoll::Completed(failure.into_outcome());
            }
        };
        self.observe_host_capture(requested)
    }

    fn timeout_visual_capture<Target, Policy>(
        &self,
        pending: &UiPendingVisualCapture<Target, Policy>,
    ) -> UiVisualSnapshotOutcome<Policy::CapturedPosture>
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        let Some(request) = pending.host_request() else {
            return UiVisualSnapshotOutcome::Denied(UiVisualSnapshotDenial::DeadlineAlreadyElapsed);
        };
        let host = self.host_session.effect_port();
        match host
            .adapter()
            .cancel_visual_capture(host.authority(), request)
        {
            worth_ui_host_contract::UiHostCaptureCancellationOutcome::CleanupIndeterminate => {
                UiVisualSnapshotOutcome::Indeterminate(
                    worth_ui_inspection::UiVisualSnapshotIndeterminate::Cleanup,
                )
            }
            worth_ui_host_contract::UiHostCaptureCancellationOutcome::CancelledBeforeReadback
            | worth_ui_host_contract::UiHostCaptureCancellationOutcome::ReadbackMayHaveBegun => {
                UiVisualSnapshotOutcome::Indeterminate(
                    worth_ui_inspection::UiVisualSnapshotIndeterminate::TimeoutAfterHostRequest,
                )
            }
        }
    }

    fn ensure_host_capture_request<Target, Policy>(
        &self,
        pending: UiPendingVisualCapture<Target, Policy>,
    ) -> Result<
        crate::inspection::visual_snapshot::UiRequestedHostVisualCapture<Target, Policy>,
        UiVisualCaptureFailure,
    >
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        pending.into_host_requested(|pinned| {
            let maximum_pixel_bytes = match host_request::capture_byte_limit(
                Policy::PIXELS_REQUIRED,
                self.host_session
                    .output_adapter()
                    .visual_capture_capability(),
                self.visual_inspection.policy(),
            ) {
                host_request::UiHostCaptureAdmission::Supported(bytes) => bytes,
                host_request::UiHostCaptureAdmission::Unsupported => {
                    return Err(UiVisualCaptureFailure::Omitted(
                        UiVisualSnapshotOmission::HostCapabilityUnsupported,
                    ));
                }
                host_request::UiHostCaptureAdmission::AffinityIndeterminate => {
                    return Err(UiVisualCaptureFailure::Indeterminate(
                        worth_ui_inspection::UiVisualSnapshotIndeterminate::CaptureAffinity,
                    ));
                }
            };
            Ok(host_request::host_capture_request(
                Policy::PIXELS_REQUESTED,
                pinned.capture_identity(),
                pinned.presentation(),
                self.host_session.identity().as_u64(),
                maximum_pixel_bytes,
            ))
        })
    }

    fn observe_host_capture<Target, Policy>(
        &self,
        requested: crate::inspection::visual_snapshot::UiRequestedHostVisualCapture<Target, Policy>,
    ) -> UiVisualCapturePoll<Target, Policy>
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        let host = self.host_session.effect_port();
        let request = requested.host_request();
        let outcome = host
            .adapter()
            .capture_visual_presentation(host.authority(), request);
        let relation = self
            .mounted
            .visual_snapshot_relation(requested.presentation().frame)
            .expect("a pending visual capture keeps its exact mounted frame pinned");
        map_host_outcome(
            requested,
            relation,
            outcome,
            self.visual_inspection.policy(),
        )
    }
}

fn map_host_outcome<Target, Policy>(
    requested: crate::inspection::visual_snapshot::UiRequestedHostVisualCapture<Target, Policy>,
    relation: worth_ui_inspection::UiVisualSnapshotRelation,
    outcome: worth_ui_host_contract::UiHostCaptureObservationOutcome,
    inspection_policy: worth_ui_inspection::UiVisualInspectionPolicy,
) -> UiVisualCapturePoll<Target, Policy>
where
    Target: UiVisualTarget,
    Policy: UiVisualArtifactPolicy,
{
    let completed = match outcome {
        worth_ui_host_contract::UiHostCaptureObservationOutcome::Pending => {
            return UiVisualCapturePoll::Pending(UiPendingVisualCapture::host_requested(requested));
        }
        worth_ui_host_contract::UiHostCaptureObservationOutcome::Captured(observation) => {
            match captured_artifact::seal_host_capture(
                requested.observe(observation),
                relation,
                inspection_policy,
            ) {
                Ok(receipt) => UiVisualSnapshotOutcome::Captured(receipt),
                Err(failure) => failure.into_outcome(),
            }
        }
        worth_ui_host_contract::UiHostCaptureObservationOutcome::SupersededBeforeReadback => {
            UiVisualCaptureFailure::Superseded(
                worth_ui_inspection::UiVisualSnapshotSuperseded::from_runtime_projection(false),
            )
            .into_outcome()
        }
        worth_ui_host_contract::UiHostCaptureObservationOutcome::CaptureAffinityIndeterminate => {
            UiVisualCaptureFailure::Indeterminate(
                worth_ui_inspection::UiVisualSnapshotIndeterminate::CaptureAffinity,
            )
            .into_outcome()
        }
        worth_ui_host_contract::UiHostCaptureObservationOutcome::ReadbackCompletionIndeterminate => {
            UiVisualCaptureFailure::Indeterminate(
                worth_ui_inspection::UiVisualSnapshotIndeterminate::HostCompletion,
            )
            .into_outcome()
        }
        worth_ui_host_contract::UiHostCaptureObservationOutcome::Unsupported => {
            UiVisualCaptureFailure::Omitted(UiVisualSnapshotOmission::HostCapabilityUnsupported)
                .into_outcome()
        }
        worth_ui_host_contract::UiHostCaptureObservationOutcome::CapacityExceeded => {
            UiVisualCaptureFailure::Denied(UiVisualSnapshotDenial::CapacityExceeded).into_outcome()
        }
    };
    UiVisualCapturePoll::Completed(completed)
}

fn deadline_elapsed<Target, Policy>(
    pending: &UiPendingVisualCapture<Target, Policy>,
    now_tick: u64,
) -> bool
where
    Policy: UiVisualArtifactPolicy,
{
    pending
        .capture_deadline()
        .is_some_and(|deadline| now_tick > deadline.tick())
}
