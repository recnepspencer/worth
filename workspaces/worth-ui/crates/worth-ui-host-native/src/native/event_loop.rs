use std::cell::RefCell;
use std::rc::Rc;

use crate::UiNativeWindowConfiguration;

use super::UiNativeHostState;

mod application_handler;
mod application_readiness;
mod callback_thread;
mod cleanup;
mod client_close;
#[cfg(feature = "certification-support")]
mod client_close_certification;
mod close_request;
mod completion_report;
mod contract;
mod directive;
mod failure;
mod finish;
mod finish_capture;
mod finish_cleanup;
mod physical_clock;
mod physical_progression;
mod pointer_position;
mod presentation_correlation;
mod presentation_retry;
mod qualified_surface_basis;
mod readiness_progress;
mod redraw;
mod resume;
mod run;
mod run_preflight;
mod terminal_cleanup;
#[cfg(test)]
mod tests;
mod thread_posture;
mod window_port;

#[cfg(test)]
use run::stop_before_callbacks;

pub use cleanup::UiNativeEventLoopCleanup;
#[cfg(feature = "certification-support")]
pub use client_close_certification::{
    certify_client_close_with_queued_readiness, UiNativeQueuedReadinessCloseCertification,
};
pub use contract::{
    UiNativeApplicationReadinessGrant, UiNativeApplicationReadinessOwnerCount,
    UiNativeApplicationReadinessOwnerCountDenial, UiNativeClientAuthoredMountedInstanceObservation,
    UiNativeClientConditionalOutcome, UiNativeClientDerivedStateLossClass,
    UiNativeClientDerivedStateReconstructionObservation,
    UiNativeClientObservationIngressObservation, UiNativeClientPresentationAttribution,
    UiNativeClientPresentationMechanicIdentityObservation,
    UiNativeClientPresentationSemanticChange,
    UiNativeClientPresentationSemanticFrontierObservation,
    UiNativeClientPresentationSemanticSubscriberObservation,
    UiNativeClientPresentationTransitionKind, UiNativeClientPresentationTransitionObservation,
    UiNativeClientResourceObservation, UiNativeClientShutdownAttemptDisposition,
    UiNativeClientShutdownAttemptObservation, UiNativeClientShutdownObservation,
    UiNativeClientTextPresentationWorkObservation, UiNativeClientVisualCoordinateOrientation,
    UiNativeClientVisualCoordinateRounding, UiNativeClientVisualPixelColorSpace,
    UiNativeClientVisualSnapshotInput, UiNativeClientVisualSnapshotObservation,
    UiNativeClientVisualSnapshotRelation, UiNativeEventLoopClient, UiNativeEventLoopClientCleanup,
    UiNativeEventLoopClientClose, UiNativeEventLoopClientFailure, UiNativeEventLoopDirective,
    UiNativeEventLoopRunDenial, UiNativeEventLoopRunReport,
    UiNativeEventLoopShutdownOverlapObservation, UiNativeEventLoopStopReport,
    UiNativeInputReachability, UiNativeObservationReadinessGrant, UiNativePhysicalProgressClass,
    UiNativePhysicalProgressGrant, UiNativeReadinessGrant, UiNativeReducedMotionPosture,
};
use physical_clock::UiNativePhysicalEventClock;
pub use presentation_correlation::UiNativePhysicalPresentationCorrelation;
pub use thread_posture::UiNativeEventLoopThreadPosture;
pub(crate) use window_port::UiNativeOwnedWindow;

pub struct WorthUiNativeEventLoop {
    state: Rc<RefCell<UiNativeHostState>>,
    window: UiNativeWindowConfiguration,
    thread_posture: UiNativeEventLoopThreadPosture,
}

struct UiNativeEventLoopApplication<Client> {
    shared: Rc<RefCell<UiNativeHostState>>,
    configuration: UiNativeWindowConfiguration,
    client: Option<Client>,
    first_frame_presented: bool,
    readiness: super::UiNativeReadinessRegistry,
    readiness_owner: super::UiNativeReadyOwner,
    physical_readiness_owner: super::UiNativeReadyOwner,
    input_readiness_owner: super::UiNativeReadyOwner,
    application_readiness_owners: Box<[super::UiNativeReadyOwner]>,
    readiness_signals: u64,
    redraw_turns: u64,
    idle_wait_turns: u64,
    coalesced_wakes: u64,
    failure: Option<UiNativeEventLoopRunDenial>,
    run_thread: std::thread::ThreadId,
    thread_observation: Option<callback_thread::UiNativeEventLoopThreadObservation>,
    loop_resources: Vec<super::UiNativeResourceOwner>,
    port_crossings: u8,
    physical_clock: UiNativePhysicalEventClock,
    pointer_input: Option<Box<pointer_position::UiNativePointerInputPort>>,
    pending_input_reachability: contract::UiNativeInputReachability,
    thread_posture: UiNativeEventLoopThreadPosture,
}

impl WorthUiNativeEventLoop {
    pub(crate) fn from_preparation(
        state: Rc<RefCell<UiNativeHostState>>,
        window: UiNativeWindowConfiguration,
        thread_posture: UiNativeEventLoopThreadPosture,
    ) -> Self {
        Self {
            state,
            window,
            thread_posture,
        }
    }
}
