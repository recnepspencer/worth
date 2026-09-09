use super::super::{
    UiNativeEffectPosture, UiNativeGraphicsObservation, UiNativeInputObservationReport,
    UiNativePresentationObservation, UiNativeResourceCensus, UiNativeRetainedFrameObservation,
};
use super::UiNativeEventLoopCleanup;

mod application_readiness;
mod client_derived_state;
mod client_resources;
mod client_shutdown;
mod observation_readiness;
mod presentation_attribution;
mod readiness_grant;
mod run_report;
mod shutdown_overlap;
mod stop_report;
pub use application_readiness::{
    UiNativeApplicationReadinessGrant, UiNativeApplicationReadinessOwnerCount,
    UiNativeApplicationReadinessOwnerCountDenial, UiNativeReducedMotionPosture,
};
pub use client_derived_state::{
    UiNativeClientDerivedStateLossClass, UiNativeClientDerivedStateReconstructionObservation,
};
pub use client_resources::UiNativeClientResourceObservation;
pub use client_shutdown::mounted_identity::UiNativeClientAuthoredMountedInstanceObservation;
pub use client_shutdown::{
    UiNativeClientConditionalOutcome, UiNativeClientObservationIngressObservation,
    UiNativeClientPresentationMechanicIdentityObservation,
    UiNativeClientPresentationSemanticChange,
    UiNativeClientPresentationSemanticFrontierObservation,
    UiNativeClientPresentationSemanticSubscriberObservation,
    UiNativeClientPresentationTransitionKind, UiNativeClientPresentationTransitionObservation,
    UiNativeClientShutdownAttemptDisposition, UiNativeClientShutdownAttemptObservation,
    UiNativeClientShutdownObservation, UiNativeClientTextPresentationWorkObservation,
    UiNativeClientVisualCoordinateOrientation, UiNativeClientVisualCoordinateRounding,
    UiNativeClientVisualPixelColorSpace, UiNativeClientVisualSnapshotInput,
    UiNativeClientVisualSnapshotObservation, UiNativeClientVisualSnapshotRelation,
};
pub use observation_readiness::{UiNativeInputReachability, UiNativeObservationReadinessGrant};
pub use presentation_attribution::UiNativeClientPresentationAttribution;
pub use shutdown_overlap::UiNativeEventLoopShutdownOverlapObservation;

pub trait UiNativeEventLoopClient {
    fn install_observation_clock(
        &mut self,
        clock: super::UiNativeObservationClock,
    ) -> Result<(), UiNativeEventLoopClientFailure>;
    fn observation_time_ready(
        &mut self,
    ) -> Result<super::UiNativeObservationTimeProgress, UiNativeEventLoopClientFailure>;

    fn application_readiness_owner_count(&self) -> UiNativeApplicationReadinessOwnerCount {
        UiNativeApplicationReadinessOwnerCount::none()
    }
    fn install_application_readiness(
        &mut self,
        ports: Vec<crate::UiNativeApplicationReadinessPort>,
    ) -> Result<(), UiNativeEventLoopClientFailure> {
        if ports.is_empty() {
            Ok(())
        } else {
            Err(UiNativeEventLoopClientFailure::Rejected)
        }
    }
    fn application_readiness_ready(
        &mut self,
        _grant: UiNativeApplicationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        Err(UiNativeEventLoopClientFailure::Rejected)
    }
    fn native_surface_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure>;
    fn redraw_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure>;
    fn physical_work_progressed(
        &mut self,
        _grant: UiNativePhysicalProgressGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        Ok(UiNativeEventLoopDirective::Continue)
    }
    fn native_observations_ready(
        &mut self,
        grant: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure>;
    fn external_close_requested(
        &mut self,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        Ok(UiNativeEventLoopDirective::Close)
    }
    fn presentation_attribution(&self) -> Option<UiNativeClientPresentationAttribution>;
    fn close(self) -> UiNativeEventLoopClientClose;
}

#[must_use]
pub struct UiNativePhysicalProgressGrant {
    class: UiNativePhysicalProgressClass,
    presentation: Option<super::UiNativePhysicalPresentationCorrelation>,
    originating_presentation: Option<super::UiNativePhysicalPresentationCorrelation>,
    duplicate_presentation_observed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeEventLoopClientFailure {
    Rejected,
}

impl UiNativePhysicalProgressGrant {
    pub(super) const fn issued(
        class: UiNativePhysicalProgressClass,
        presentation: Option<super::UiNativePhysicalPresentationCorrelation>,
        duplicate_presentation_observed: bool,
    ) -> Self {
        Self {
            class,
            presentation,
            originating_presentation: None,
            duplicate_presentation_observed,
        }
    }

    pub(super) const fn issued_with_originating_presentation(
        class: UiNativePhysicalProgressClass,
        originating_presentation: super::UiNativePhysicalPresentationCorrelation,
    ) -> Self {
        Self {
            class,
            presentation: None,
            originating_presentation: Some(originating_presentation),
            duplicate_presentation_observed: false,
        }
    }

    pub const fn class(&self) -> UiNativePhysicalProgressClass {
        self.class
    }

    pub const fn presentation(&self) -> Option<super::UiNativePhysicalPresentationCorrelation> {
        self.presentation
    }

    pub const fn originating_presentation(
        &self,
    ) -> Option<super::UiNativePhysicalPresentationCorrelation> {
        self.originating_presentation
    }

    pub const fn duplicate_presentation_observed(&self) -> bool {
        self.duplicate_presentation_observed
    }

    #[cfg(feature = "certification-support")]
    #[doc(hidden)]
    pub const fn from_certification(
        class: UiNativePhysicalProgressClass,
        presentation: Option<super::UiNativePhysicalPresentationCorrelation>,
        duplicate_presentation_observed: bool,
    ) -> Self {
        Self::issued(class, presentation, duplicate_presentation_observed)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePhysicalProgressClass {
    TextAtlas,
    Presentation,
    PresentationRecovery,
}

pub trait UiNativeEventLoopClientCleanup {
    fn retry(self: Box<Self>) -> UiNativeEventLoopClientClose;
}

pub enum UiNativeEventLoopClientClose {
    Complete,
    CompleteWithObservation(UiNativeClientShutdownObservation),
    Incomplete(Box<dyn UiNativeEventLoopClientCleanup>),
}

impl UiNativeEventLoopClientClose {
    pub(super) fn into_parts(
        self,
    ) -> (
        Option<Box<dyn UiNativeEventLoopClientCleanup>>,
        Option<UiNativeClientShutdownObservation>,
    ) {
        match self {
            Self::Complete => (None, None),
            Self::CompleteWithObservation(observation) => (None, Some(observation)),
            Self::Incomplete(cleanup) => (Some(cleanup), None),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeEventLoopDirective {
    Continue,
    WaitUntil(std::time::Instant),
    Close,
}

/// Result of closing native observation time and progressing its presentation owner.
pub struct UiNativeObservationTimeProgress {
    deadline: Option<u64>,
    directive: UiNativeEventLoopDirective,
}

impl UiNativeObservationTimeProgress {
    pub const fn new(deadline: Option<u64>, directive: UiNativeEventLoopDirective) -> Self {
        Self {
            deadline,
            directive,
        }
    }
    pub const fn deadline(&self) -> Option<u64> {
        self.deadline
    }
    pub const fn directive(&self) -> UiNativeEventLoopDirective {
        self.directive
    }
}

#[must_use]
pub struct UiNativeReadinessGrant {
    generation: u64,
    surface_basis_generation: u64,
    scale_factor_milli: u32,
    client_physical_size: [u32; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeEventLoopRunDenial {
    EventLoopCreation,
    WindowCreation,
    GraphicsPreparation,
    ApplicationDriver,
    PresentationDeadlineExpired,
    EventLoopRun,
    IncompleteCleanup,
}

#[derive(Debug)]
pub struct UiNativeEventLoopStopReport {
    pub(super) cause: UiNativeEventLoopRunDenial,
    pub(super) effect_posture: UiNativeEffectPosture,
    pub(super) peak_census: Box<UiNativeResourceCensus>,
    pub(super) terminal_census: Box<UiNativeResourceCensus>,
    pub(super) client_cleanup_complete: bool,
    pub(super) cleanup: Option<UiNativeEventLoopCleanup>,
    pub(super) peak_text_pins: Box<[crate::native::text_atlas::UiNativeTextPinObservation]>,
    pub(super) input_observations: Box<UiNativeInputObservationReport>,
    pub(super) shutdown_overlap: UiNativeEventLoopShutdownOverlapObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeEventLoopRunReport {
    pub(super) presentation: UiNativePresentationObservation,
    pub(super) graphics: UiNativeGraphicsObservation,
    pub(super) event_loop_thread: Box<str>,
    pub(super) event_loop_thread_matches_launch: bool,
    pub(super) event_loop_thread_posture: super::UiNativeEventLoopThreadPosture,
    pub(super) client_attribution: UiNativeClientPresentationAttribution,
    pub(super) readiness_signals: u64,
    pub(super) redraw_turns: u64,
    pub(super) idle_wait_turns: u64,
    pub(super) coalesced_wakes: u64,
    pub(super) peak_census: UiNativeResourceCensus,
    pub(super) terminal_census: UiNativeResourceCensus,
    pub(super) port_crossings: u8,
    pub(super) retained_frames: Box<[UiNativeRetainedFrameObservation]>,
    pub(super) peak_text_pins: Box<[crate::native::text_atlas::UiNativeTextPinObservation]>,
    pub(super) text_pin_frame_counts: Box<[u32]>,
    pub(super) text_pin_frame_observations:
        Box<[Box<[crate::native::text_atlas::UiNativeTextPinObservation]>]>,
    pub(super) text_atlas_model_frame_digests: Box<[[u8; 32]]>,
    pub(super) observation_history_complete: bool,
    pub(super) text_atlas_transactions: u64,
    pub(super) derived_state_reconstruction:
        Option<crate::UiNativeDerivedStateReconstructionObservation>,
    pub(super) text_atlas_plan_observations:
        Box<[crate::native::text_atlas::UiNativeTextAtlasPlanObservation]>,
    pub(super) physical_signal_transition_observations:
        Box<[crate::native::physical_work_signal::UiNativePhysicalSignalTransitionObservation]>,
    pub(super) physical_signal_transition_trace_complete: bool,
    pub(super) physical_signal_lifecycle: crate::native::UiNativePhysicalSignalLifecycleObservation,
    pub(super) client_shutdown: Option<UiNativeClientShutdownObservation>,
    pub(super) input_observations: UiNativeInputObservationReport,
    pub(super) shutdown_overlap: UiNativeEventLoopShutdownOverlapObservation,
}
