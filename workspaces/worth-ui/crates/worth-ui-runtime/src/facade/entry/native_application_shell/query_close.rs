pub(crate) struct UiNativeApplicationQueryCloseObservation {
    closed_query_resources: u64,
    transitions: Box<[worth_ui_query_binding::WorthUiPresentationTransitionObservation]>,
    text_presentation_work:
        Box<[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation]>,
    text_presentation_work_trace_complete: bool,
    authored_mounted_instances:
        Box<[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation]>,
    client_resource_peaks: [usize; 2],
    mounted_shutdown_attempts: Box<[crate::mounting::UiMountedPresentationShutdownAttempt]>,
    intent_resources_empty: bool,
    query_close_complete: bool,
    transition_trace_complete: bool,
}

#[derive(Debug)]
pub(super) struct UiNativeApplicationQueryCloseInput {
    pub(super) closed_resources: u64,
    pub(super) transitions: Box<[worth_ui_query_binding::WorthUiPresentationTransitionObservation]>,
    pub(super) text_work:
        Box<[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation]>,
    pub(super) text_work_trace_complete: bool,
    pub(super) authored_mounted_instances:
        Box<[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation]>,
    pub(super) client_resource_peaks: [usize; 2],
    pub(super) mounted_shutdown_attempts:
        Box<[crate::mounting::UiMountedPresentationShutdownAttempt]>,
    pub(super) intent_resources_empty: bool,
    pub(super) query_close_complete: bool,
    pub(super) transition_trace_complete: bool,
}

impl UiNativeApplicationQueryCloseObservation {
    pub(super) fn from_runtime(input: UiNativeApplicationQueryCloseInput) -> Self {
        Self {
            closed_query_resources: input.closed_resources,
            transitions: input.transitions,
            text_presentation_work: input.text_work,
            text_presentation_work_trace_complete: input.text_work_trace_complete,
            authored_mounted_instances: input.authored_mounted_instances,
            client_resource_peaks: input.client_resource_peaks,
            mounted_shutdown_attempts: input.mounted_shutdown_attempts,
            intent_resources_empty: input.intent_resources_empty,
            query_close_complete: input.query_close_complete,
            transition_trace_complete: input.transition_trace_complete,
        }
    }

    pub(crate) fn empty_complete() -> Self {
        Self::from_runtime(UiNativeApplicationQueryCloseInput {
            closed_resources: 0,
            transitions: Box::new([]),
            text_work: Box::new([]),
            text_work_trace_complete: true,
            authored_mounted_instances: Box::new([]),
            client_resource_peaks: [0, 0],
            mounted_shutdown_attempts: Box::new([]),
            intent_resources_empty: true,
            query_close_complete: true,
            transition_trace_complete: true,
        })
    }

    pub(crate) const fn closed_query_resources(&self) -> u64 {
        self.closed_query_resources
    }

    pub(crate) fn transitions(
        &self,
    ) -> &[worth_ui_query_binding::WorthUiPresentationTransitionObservation] {
        &self.transitions
    }

    pub(crate) const fn query_close_complete(&self) -> bool {
        self.query_close_complete
    }

    pub(crate) const fn transition_trace_complete(&self) -> bool {
        self.transition_trace_complete
    }

    pub(crate) fn text_presentation_work(
        &self,
    ) -> &[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation] {
        &self.text_presentation_work
    }

    pub(crate) const fn text_presentation_work_trace_complete(&self) -> bool {
        self.text_presentation_work_trace_complete
    }

    pub(crate) fn authored_mounted_instances(
        &self,
    ) -> &[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation] {
        &self.authored_mounted_instances
    }

    pub(crate) const fn client_resource_peaks(&self) -> [usize; 2] {
        self.client_resource_peaks
    }

    pub(crate) fn mounted_shutdown_attempts(
        &self,
    ) -> &[crate::mounting::UiMountedPresentationShutdownAttempt] {
        &self.mounted_shutdown_attempts
    }

    pub(crate) const fn intent_resources_empty(&self) -> bool {
        self.intent_resources_empty
    }
}
