use super::{UiNativeApplicationQueryCloseObservation, WorthUiNativeApplicationShell};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct WorthUiNativeApplicationShutdownReceipt {
    mounted_shutdown_attempts: Box<[crate::mounting::UiMountedPresentationShutdownAttempt]>,
    intent_resources_empty: bool,
    runtime_service_resources_empty: bool,
    portal_closed_records: usize,
    portal_abandoned_indeterminate_records: usize,
    portal_final_active_records: usize,
    motion_abandoned_staged_tracks: u16,
    motion_terminated_active_tracks: u16,
    motion_cancelled_exit_retentions: u16,
    motion_final_census_is_zero: bool,
    command_routes_released: usize,
    scroll_owners_released: usize,
    selection_owners_released: usize,
    focus_abandoned_indeterminate_request:
        Option<worth_ui_host_contract::UiHostFocusPlacementRequestIdentity>,
    closed_query_resources: u64,
    query_close_complete: bool,
    query_transitions: Box<[worth_ui_query_binding::WorthUiPresentationTransitionObservation]>,
    query_transition_trace_complete: bool,
    text_presentation_work:
        Box<[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation]>,
    text_presentation_work_trace_complete: bool,
    authored_mounted_instances:
        Box<[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation]>,
    client_resource_peaks: [usize; 2],
    visual_capture: crate::inspection::visual_snapshot::UiVisualCaptureShutdownReport,
    visual_overlay: crate::inspection::visual_snapshot::UiVisualOverlayShutdownReport,
    host_session_released: bool,
    released_surface_count: usize,
    host_cleanup: Option<crate::facade::WorthUiHostSessionReleaseRecovery>,
    presentation_async_cleanup:
        Option<crate::native_platform::text_presentation::UiPresentationAsyncTerminalCleanup>,
}

#[derive(Debug)]
pub struct WorthUiNativeApplicationCleanup {
    pub(super) host_cleanup: Option<crate::facade::WorthUiHostSessionReleaseRecovery>,
    pub(super) presentation_async_cleanup:
        Option<crate::native_platform::text_presentation::UiPresentationAsyncTerminalCleanup>,
    pub(super) query_close: Box<super::query_close::UiNativeApplicationQueryCloseInput>,
}

impl WorthUiNativeApplicationShell {
    /// Consume the shell and report runtime, mounted, and host cleanup.
    pub fn shutdown(mut self) -> WorthUiNativeApplicationShutdownReceipt {
        self.cancel_managed_rebind_for_shutdown();
        let client_resource_peaks = self.session.mounted.native_client_resource_peaks();
        let authored_mounted_instances = authored_mounted_instances(&self);
        let mut runtime = self.session.shutdown();
        let intent_resources_empty = runtime.intent_resource_census().is_empty();
        let runtime_service_resources_empty = runtime.runtime_service_resource_census().is_empty();
        let portal_closed_records = runtime.portal_closed_records();
        let portal_abandoned_indeterminate_records =
            runtime.portal_abandoned_indeterminate_records();
        let portal_final_active_records = runtime.portal_final_active_records();
        let motion_abandoned_staged_tracks = runtime.motion_abandoned_staged_tracks();
        let motion_terminated_active_tracks = runtime.motion_terminated_active_tracks();
        let motion_cancelled_exit_retentions = runtime.motion_cancelled_exit_retentions();
        let motion_final_census_is_zero = runtime.motion_final_census_is_zero();
        let command_routes_released = runtime.command_routes_released();
        let scroll_owners_released = runtime.scroll_owners_released();
        let selection_owners_released = runtime.selection_owners_released();
        let focus_abandoned_indeterminate_request =
            runtime.focus_placement().abandoned_indeterminate_request();
        let (host_session_released, released_surface_count) = match runtime.host_session_release() {
            Some(worth_ui_host_contract::UiHostSessionReleaseOutcome::Released(receipt)) => {
                (true, receipt.released_surface_count())
            }
            Some(worth_ui_host_contract::UiHostSessionReleaseOutcome::ReleaseIndeterminate(_))
            | None => (false, 0),
        };
        WorthUiNativeApplicationShutdownReceipt {
            mounted_shutdown_attempts: runtime
                .mounted_presentation()
                .attempts()
                .to_vec()
                .into_boxed_slice(),
            intent_resources_empty,
            runtime_service_resources_empty,
            portal_closed_records,
            portal_abandoned_indeterminate_records,
            portal_final_active_records,
            motion_abandoned_staged_tracks,
            motion_terminated_active_tracks,
            motion_cancelled_exit_retentions,
            motion_final_census_is_zero,
            command_routes_released,
            scroll_owners_released,
            selection_owners_released,
            focus_abandoned_indeterminate_request,
            closed_query_resources: runtime.mounted_presentation().closed_query_resources(),
            query_close_complete: runtime.mounted_presentation().query_close_complete(),
            query_transitions: runtime
                .mounted_presentation()
                .query_transitions()
                .to_vec()
                .into_boxed_slice(),
            query_transition_trace_complete: runtime
                .mounted_presentation()
                .query_transition_trace_complete(),
            text_presentation_work: runtime
                .mounted_presentation()
                .text_presentation_work()
                .to_vec()
                .into_boxed_slice(),
            text_presentation_work_trace_complete: runtime
                .mounted_presentation()
                .text_presentation_work_trace_complete(),
            authored_mounted_instances,
            client_resource_peaks,
            visual_capture: runtime.visual_capture(),
            visual_overlay: runtime.visual_overlay(),
            host_session_released,
            released_surface_count,
            host_cleanup: runtime.take_host_session_recovery(),
            presentation_async_cleanup: runtime.take_presentation_async_cleanup(),
        }
    }
}

impl WorthUiNativeApplicationShutdownReceipt {
    pub fn mounted_shutdown_attempt_count(&self) -> usize {
        self.mounted_shutdown_attempts.len()
    }

    pub const fn intent_resources_empty(&self) -> bool {
        self.intent_resources_empty
    }

    pub const fn runtime_service_resources_empty(&self) -> bool {
        self.runtime_service_resources_empty
    }

    pub const fn portal_closed_records(&self) -> usize {
        self.portal_closed_records
    }

    pub const fn portal_abandoned_indeterminate_records(&self) -> usize {
        self.portal_abandoned_indeterminate_records
    }

    pub const fn portal_final_active_records(&self) -> usize {
        self.portal_final_active_records
    }

    pub const fn motion_abandoned_staged_tracks(&self) -> u16 {
        self.motion_abandoned_staged_tracks
    }

    pub const fn motion_terminated_active_tracks(&self) -> u16 {
        self.motion_terminated_active_tracks
    }

    pub const fn motion_cancelled_exit_retentions(&self) -> u16 {
        self.motion_cancelled_exit_retentions
    }

    pub const fn motion_final_census_is_zero(&self) -> bool {
        self.motion_final_census_is_zero
    }

    pub const fn command_routes_released(&self) -> usize {
        self.command_routes_released
    }

    pub const fn scroll_owners_released(&self) -> usize {
        self.scroll_owners_released
    }

    pub const fn selection_owners_released(&self) -> usize {
        self.selection_owners_released
    }

    pub const fn focus_abandoned_indeterminate_request(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostFocusPlacementRequestIdentity> {
        self.focus_abandoned_indeterminate_request
    }

    pub const fn closed_query_resources(&self) -> u64 {
        self.closed_query_resources
    }

    pub const fn query_close_complete(&self) -> bool {
        self.query_close_complete
    }

    pub fn query_transitions(
        &self,
    ) -> &[worth_ui_query_binding::WorthUiPresentationTransitionObservation] {
        &self.query_transitions
    }

    pub const fn query_transition_trace_complete(&self) -> bool {
        self.query_transition_trace_complete
    }

    pub(crate) fn into_query_close_observation(self) -> UiNativeApplicationQueryCloseObservation {
        UiNativeApplicationQueryCloseObservation::from_runtime(
            super::query_close::UiNativeApplicationQueryCloseInput {
                closed_resources: self.closed_query_resources,
                transitions: self.query_transitions,
                text_work: self.text_presentation_work,
                text_work_trace_complete: self.text_presentation_work_trace_complete,
                authored_mounted_instances: self.authored_mounted_instances,
                client_resource_peaks: self.client_resource_peaks,
                mounted_shutdown_attempts: self.mounted_shutdown_attempts,
                intent_resources_empty: self.intent_resources_empty,
                query_close_complete: self.query_close_complete,
                transition_trace_complete: self.query_transition_trace_complete,
            },
        )
    }

    pub fn host_session_released(&self) -> bool {
        self.host_session_released
    }

    pub const fn visual_capture(
        &self,
    ) -> crate::inspection::visual_snapshot::UiVisualCaptureShutdownReport {
        self.visual_capture
    }

    pub const fn visual_overlay(
        &self,
    ) -> crate::inspection::visual_snapshot::UiVisualOverlayShutdownReport {
        self.visual_overlay
    }

    pub fn released_surface_count(&self) -> usize {
        self.released_surface_count
    }

    pub(crate) fn into_application_cleanup(self) -> Option<WorthUiNativeApplicationCleanup> {
        if self.host_cleanup.is_none() && self.presentation_async_cleanup.is_none() {
            return None;
        }
        Some(WorthUiNativeApplicationCleanup {
            host_cleanup: self.host_cleanup,
            presentation_async_cleanup: self.presentation_async_cleanup,
            query_close: Box::new(super::query_close::UiNativeApplicationQueryCloseInput {
                closed_resources: self.closed_query_resources,
                transitions: self.query_transitions,
                text_work: self.text_presentation_work,
                text_work_trace_complete: self.text_presentation_work_trace_complete,
                authored_mounted_instances: self.authored_mounted_instances,
                client_resource_peaks: self.client_resource_peaks,
                mounted_shutdown_attempts: self.mounted_shutdown_attempts,
                intent_resources_empty: self.intent_resources_empty,
                query_close_complete: self.query_close_complete,
                transition_trace_complete: self.query_transition_trace_complete,
            }),
        })
    }
}

impl WorthUiNativeApplicationCleanup {
    pub(crate) fn retry(mut self) -> Result<UiNativeApplicationQueryCloseObservation, Self> {
        if let Some(cleanup) = self.presentation_async_cleanup.take() {
            match cleanup.retry() {
                Ok(receipt) => {
                    self.query_close.closed_resources = receipt.closed_query_resources();
                    self.query_close.query_close_complete = true;
                    self.query_close.transitions =
                        receipt.transitions().to_vec().into_boxed_slice();
                    self.query_close.transition_trace_complete =
                        receipt.transition_trace_complete();
                }
                Err(cleanup) => {
                    self.presentation_async_cleanup = Some(cleanup);
                    return Err(self);
                }
            }
        }
        if let Some(recovery) = self.host_cleanup.take() {
            if let Err(recovery) = recovery.retry() {
                self.host_cleanup = Some(recovery);
                return Err(self);
            }
        }
        Ok(UiNativeApplicationQueryCloseObservation::from_runtime(
            *self.query_close,
        ))
    }
}

fn authored_mounted_instances(
    shell: &WorthUiNativeApplicationShell,
) -> Box<[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation]> {
    let mut observations = shell
        .mounted_row_indices
        .iter()
        .map(|(authored, index)| {
            worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation::reported(
                Sha256::digest(authored.as_bytes()).into(),
                shell.mounted_rows[*index].latest_mounted.diagnostic_value(),
            )
        })
        .collect::<Vec<_>>();
    observations.sort_unstable();
    observations.into_boxed_slice()
}
