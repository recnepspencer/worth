pub(super) mod mounted_identity;
mod observation_ingress;
mod visual_snapshot;

pub use observation_ingress::UiNativeClientObservationIngressObservation;
pub use visual_snapshot::{
    UiNativeClientVisualCoordinateOrientation, UiNativeClientVisualCoordinateRounding,
    UiNativeClientVisualPixelColorSpace, UiNativeClientVisualSnapshotInput,
    UiNativeClientVisualSnapshotObservation, UiNativeClientVisualSnapshotRelation,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiNativeClientShutdownObservation {
    managed_semantic_resources_closed: u64,
    managed_semantic_resources_complete: bool,
    presentation_transitions: Box<[UiNativeClientPresentationTransitionObservation]>,
    presentation_transition_trace_complete: bool,
    text_presentation_work: Box<[UiNativeClientTextPresentationWorkObservation]>,
    text_presentation_work_trace_complete: bool,
    authored_mounted_instances: mounted_identity::Observations,
    derived_state_reconstruction:
        Option<super::UiNativeClientDerivedStateReconstructionObservation>,
    resources: super::UiNativeClientResourceObservation,
    observation_ingress: UiNativeClientObservationIngressObservation,
    visual_snapshot: Option<UiNativeClientVisualSnapshotObservation>,
    shutdown_attempts: Box<[UiNativeClientShutdownAttemptObservation]>,
    intent_resources_empty: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeClientShutdownAttemptDisposition {
    CancelledBeforeEffects,
    PresentationIndeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeClientShutdownAttemptObservation {
    attempt: u64,
    disposition: UiNativeClientShutdownAttemptDisposition,
    affected_bindings: Box<[u64]>,
}

impl UiNativeClientShutdownAttemptObservation {
    pub fn reported(
        attempt: u64,
        disposition: UiNativeClientShutdownAttemptDisposition,
        affected_bindings: impl IntoIterator<Item = u64>,
    ) -> Self {
        Self {
            attempt,
            disposition,
            affected_bindings: affected_bindings.into_iter().collect(),
        }
    }

    pub const fn attempt(&self) -> u64 {
        self.attempt
    }

    pub const fn disposition(&self) -> UiNativeClientShutdownAttemptDisposition {
        self.disposition
    }

    pub fn affected_bindings(&self) -> &[u64] {
        &self.affected_bindings
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeClientPresentationTransitionKind {
    Pending,
    Superseded,
    StaleCompletionRejected,
    Completed,
    DuplicateCompletionRejected,
    Cancelled,
    Unresolved,
    RecoveryRequired,
    ReconstructionCurrent,
    TerminalClosed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeClientPresentationTransitionObservation {
    kind: UiNativeClientPresentationTransitionKind,
    attempt: u64,
    binding: u64,
}

impl UiNativeClientShutdownObservation {
    pub fn from_client(
        managed_semantic_resources_closed: u64,
        managed_semantic_resources_complete: bool,
    ) -> Self {
        Self {
            managed_semantic_resources_closed,
            managed_semantic_resources_complete,
            presentation_transitions: Box::new([]),
            presentation_transition_trace_complete: true,
            text_presentation_work: Box::new([]),
            text_presentation_work_trace_complete: true,
            authored_mounted_instances: Box::new([]),
            derived_state_reconstruction: None,
            resources: Default::default(),
            observation_ingress: Default::default(),
            visual_snapshot: None,
            shutdown_attempts: Box::new([]),
            intent_resources_empty: true,
        }
    }

    pub fn from_client_with_presentation_transitions(
        managed_semantic_resources_closed: u64,
        managed_semantic_resources_complete: bool,
        presentation_transitions: Box<[UiNativeClientPresentationTransitionObservation]>,
        presentation_transition_trace_complete: bool,
    ) -> Self {
        Self {
            managed_semantic_resources_closed,
            managed_semantic_resources_complete,
            presentation_transitions,
            presentation_transition_trace_complete,
            text_presentation_work: Box::new([]),
            text_presentation_work_trace_complete: true,
            authored_mounted_instances: Box::new([]),
            derived_state_reconstruction: None,
            resources: Default::default(),
            observation_ingress: Default::default(),
            visual_snapshot: None,
            shutdown_attempts: Box::new([]),
            intent_resources_empty: true,
        }
    }

    pub fn with_text_presentation_work(
        mut self,
        observations: Box<[UiNativeClientTextPresentationWorkObservation]>,
        trace_complete: bool,
    ) -> Self {
        self.text_presentation_work = observations;
        self.text_presentation_work_trace_complete = trace_complete;
        self
    }

    pub fn with_derived_state_reconstruction(
        mut self,
        observation: Option<super::UiNativeClientDerivedStateReconstructionObservation>,
    ) -> Self {
        self.derived_state_reconstruction = observation;
        self
    }

    pub const fn with_resources(
        mut self,
        resources: super::UiNativeClientResourceObservation,
    ) -> Self {
        self.resources = resources;
        self
    }

    pub const fn with_observation_ingress(
        mut self,
        observation_ingress: UiNativeClientObservationIngressObservation,
    ) -> Self {
        self.observation_ingress = observation_ingress;
        self
    }

    pub fn with_visual_snapshot(
        mut self,
        visual_snapshot: Option<UiNativeClientVisualSnapshotObservation>,
    ) -> Self {
        self.visual_snapshot = visual_snapshot;
        self
    }

    pub fn with_shutdown_attempts(
        mut self,
        shutdown_attempts: Box<[UiNativeClientShutdownAttemptObservation]>,
    ) -> Self {
        self.shutdown_attempts = shutdown_attempts;
        self
    }

    pub const fn with_intent_resources_empty(mut self, intent_resources_empty: bool) -> Self {
        self.intent_resources_empty = intent_resources_empty;
        self
    }

    pub const fn managed_semantic_resources_closed(&self) -> u64 {
        self.managed_semantic_resources_closed
    }

    pub const fn managed_semantic_resources_complete(&self) -> bool {
        self.managed_semantic_resources_complete
    }

    pub fn presentation_transitions(&self) -> &[UiNativeClientPresentationTransitionObservation] {
        &self.presentation_transitions
    }

    pub const fn presentation_transition_trace_complete(&self) -> bool {
        self.presentation_transition_trace_complete
    }

    pub fn text_presentation_work(&self) -> &[UiNativeClientTextPresentationWorkObservation] {
        &self.text_presentation_work
    }

    pub const fn text_presentation_work_trace_complete(&self) -> bool {
        self.text_presentation_work_trace_complete
    }

    pub const fn derived_state_reconstruction(
        &self,
    ) -> Option<super::UiNativeClientDerivedStateReconstructionObservation> {
        self.derived_state_reconstruction
    }

    pub const fn resources(&self) -> super::UiNativeClientResourceObservation {
        self.resources
    }

    pub const fn observation_ingress(&self) -> UiNativeClientObservationIngressObservation {
        self.observation_ingress
    }

    pub const fn visual_snapshot(&self) -> Option<&UiNativeClientVisualSnapshotObservation> {
        self.visual_snapshot.as_ref()
    }

    pub fn shutdown_attempts(&self) -> &[UiNativeClientShutdownAttemptObservation] {
        &self.shutdown_attempts
    }

    pub const fn intent_resources_empty(&self) -> bool {
        self.intent_resources_empty
    }

    pub(in crate::native::event_loop) const fn terminal_resources_complete(&self) -> bool {
        self.managed_semantic_resources_complete
            && self.intent_resources_empty
            && self.resources.terminal_mounted_layouts() == 0
            && self.resources.terminal_raster_cache_entries() == 0
    }
}

impl UiNativeClientPresentationTransitionObservation {
    pub const fn reported(
        kind: UiNativeClientPresentationTransitionKind,
        attempt: u64,
        binding: u64,
    ) -> Self {
        Self {
            kind,
            attempt,
            binding,
        }
    }

    pub const fn kind(self) -> UiNativeClientPresentationTransitionKind {
        self.kind
    }

    pub const fn attempt(self) -> u64 {
        self.attempt
    }

    pub const fn binding(self) -> u64 {
        self.binding
    }
}
mod text_presentation_work;

pub use text_presentation_work::{
    UiNativeClientPresentationMechanicIdentityObservation,
    UiNativeClientTextPresentationWorkObservation,
};
