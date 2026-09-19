use worth_ui_host_contract::{UiMountedPresentationAttemptIdentity, UiSurfaceBindingGeneration};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPresentationShutdownDisposition {
    CancelledBeforeEffects,
    PresentationIndeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedPresentationShutdownAttempt {
    attempt: UiMountedPresentationAttemptIdentity,
    disposition: UiMountedPresentationShutdownDisposition,
    affected_bindings: Box<[UiSurfaceBindingGeneration]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedPresentationShutdownReport {
    attempts: Box<[UiMountedPresentationShutdownAttempt]>,
    closed_query_resources: u64,
    query_close_complete: bool,
    query_transitions: Box<[worth_ui_query_binding::WorthUiPresentationTransitionObservation]>,
    query_transition_trace_complete: bool,
    text_presentation_work:
        Box<[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation]>,
    text_presentation_work_trace_complete: bool,
}

pub(crate) struct UiMountedPresentationQueryShutdown {
    pub(super) closed_resources: u64,
    pub(super) complete: bool,
    pub(super) transitions: Box<[worth_ui_query_binding::WorthUiPresentationTransitionObservation]>,
    pub(super) transition_trace_complete: bool,
}

pub(crate) struct UiMountedPresentationTextShutdown {
    pub(super) work:
        Box<[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation]>,
    pub(super) trace_complete: bool,
}

impl UiMountedPresentationShutdownAttempt {
    pub(super) fn new(
        attempt: UiMountedPresentationAttemptIdentity,
        disposition: UiMountedPresentationShutdownDisposition,
        affected_bindings: Vec<UiSurfaceBindingGeneration>,
    ) -> Self {
        Self {
            attempt,
            disposition,
            affected_bindings: affected_bindings.into_boxed_slice(),
        }
    }

    pub fn attempt(&self) -> UiMountedPresentationAttemptIdentity {
        self.attempt
    }

    pub fn disposition(&self) -> UiMountedPresentationShutdownDisposition {
        self.disposition
    }

    pub fn affected_bindings(&self) -> &[UiSurfaceBindingGeneration] {
        &self.affected_bindings
    }
}

impl UiMountedPresentationShutdownReport {
    pub(super) fn new(
        attempts: Vec<UiMountedPresentationShutdownAttempt>,
        query: UiMountedPresentationQueryShutdown,
        text: UiMountedPresentationTextShutdown,
    ) -> Self {
        Self {
            attempts: attempts.into_boxed_slice(),
            closed_query_resources: query.closed_resources,
            query_close_complete: query.complete,
            query_transitions: query.transitions,
            query_transition_trace_complete: query.transition_trace_complete,
            text_presentation_work: text.work,
            text_presentation_work_trace_complete: text.trace_complete,
        }
    }

    pub fn attempts(&self) -> &[UiMountedPresentationShutdownAttempt] {
        &self.attempts
    }

    pub fn is_empty(&self) -> bool {
        self.attempts.is_empty()
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

    pub(crate) fn text_presentation_work(
        &self,
    ) -> &[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation] {
        &self.text_presentation_work
    }

    pub(crate) const fn text_presentation_work_trace_complete(&self) -> bool {
        self.text_presentation_work_trace_complete
    }
}

impl Default for UiMountedPresentationShutdownReport {
    fn default() -> Self {
        Self {
            attempts: Box::new([]),
            closed_query_resources: 0,
            query_close_complete: true,
            query_transitions: Box::new([]),
            query_transition_trace_complete: true,
            text_presentation_work: Box::new([]),
            text_presentation_work_trace_complete: true,
        }
    }
}
