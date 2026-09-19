use std::collections::BTreeMap;

use worth_ui_host_contract::UiHostSurfaceRegistrationRequest;

use super::lifecycle::UiNativeLifecycleOrchestrator;
use super::physical_work_signal::UiNativePhysicalSignalOwner;
use super::text_atlas::{UiNativeTextAtlas, UiNativeTextAtlasGpuPages, UiNativeTextAtlasInFlight};
use super::{
    event_loop::UiNativeOwnedWindow, UiNativeOwnedDevice, UiNativeOwnedPresentationSurface,
    UiNativePendingPresentation, UiNativePresentationAccess, UiNativeResourceCensus,
    UiNativeResourceOwner, UiNativeResourceRegistry, UiNativeRetainedDrawList,
    UiNativeRetainedFrameObservation,
};

pub(super) const NATIVE_OBSERVATION_HISTORY_CAPACITY: usize = 64;

mod presentation_lifecycle;
pub(super) use presentation_lifecycle::UiNativePresentationPhysicalProgress;
#[cfg(feature = "certification-support")]
#[path = "host_state/derived_state_loss.rs"]
mod derived_state_loss;
#[cfg(feature = "certification-support")]
#[path = "host_state/qualification.rs"]
mod qualification;
#[cfg(feature = "certification-support")]
pub(crate) use qualification::UiNativeQualificationState;
#[cfg(test)]
#[path = "host_state/temporal_retry_tests.rs"]
mod temporal_retry_tests;
mod text_atlas_commit;
mod text_atlas_lifecycle;
#[cfg(test)]
pub(crate) use text_atlas_lifecycle::UiNativeTextAtlasPhysicalProgress;

pub(crate) struct UiNativeHostState {
    pub(crate) registrations: BTreeMap<u64, UiHostSurfaceRegistrationRequest>,
    pub(crate) registration_resources: BTreeMap<u64, UiNativeResourceOwner>,
    pub(crate) window: Option<UiNativeOwnedWindow>,
    pub(crate) accepted_cursor: Option<(
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        winit::window::CursorIcon,
    )>,
    pub(crate) device: Option<UiNativeOwnedDevice>,
    pub(crate) presentation_surface: Option<UiNativeOwnedPresentationSurface>,
    pub(crate) last_retained_frame: Option<UiNativeRetainedFrameObservation>,
    pub(crate) retained_frame_observations: Vec<UiNativeRetainedFrameObservation>,
    pub(crate) resources: UiNativeResourceRegistry,
    pub(crate) pending_presentations: Vec<UiNativePendingPresentation>,
    pub(crate) retained_draw_lists: BTreeMap<u64, UiNativeRetainedDrawList>,
    pub(crate) presentation_epochs: BTreeMap<u64, worth_ui_host_contract::UiHostPresentationEpoch>,
    pub(crate) semantic_focus:
        BTreeMap<u64, worth_ui_host_contract::UiHostFocusPlacementAcknowledgement>,
    pub(crate) captures: super::capture::UiNativeCaptureState,
    pub(crate) lifecycle: UiNativeLifecycleOrchestrator,
    pub(crate) text_atlas: UiNativeTextAtlas,
    pub(crate) text_atlas_gpu: Option<UiNativeTextAtlasGpuPages>,
    pub(crate) text_atlas_in_flight: Option<UiNativeTextAtlasInFlight>,
    pub(crate) text_atlas_recovery: Option<super::text_atlas::UiNativeTextAtlasRecovery>,
    pub(crate) text_atlas_completion: Option<(
        worth_ui_host_contract::UiGlyphRasterTransactionPending,
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome,
    )>,
    pub(crate) text_pins_by_binding:
        BTreeMap<u64, Box<[worth_ui_host_contract::UiGlyphRasterPinRequest]>>,
    pub(crate) pending_text_presentations: BTreeMap<u64, UiNativePendingTextPresentation>,
    pub(crate) physical_signal: UiNativePhysicalSignalOwner,
    pub(crate) peak_census: UiNativeResourceCensus,
    pub(crate) peak_text_pins: Box<[super::text_atlas::UiNativeTextPinObservation]>,
    pub(crate) text_pin_frame_counts: Vec<u32>,
    pub(crate) text_pin_frame_observations:
        Vec<Box<[super::text_atlas::UiNativeTextPinObservation]>>,
    pub(crate) text_atlas_model_frame_digests: Vec<[u8; 32]>,
    pub(crate) text_atlas_plan_observations:
        Vec<super::text_atlas::UiNativeTextAtlasPlanObservation>,
    pub(crate) observation_history_overflowed: bool,
    #[cfg(feature = "certification-support")]
    pub(crate) qualification: UiNativeQualificationState,
}

pub(crate) struct UiNativePendingTextPresentation {
    pub(crate) atlas: worth_ui_host_contract::UiGlyphRasterTransactionPending,
    pub(crate) continuation: UiNativePendingTextContinuation,
    pub(crate) binding: u64,
    pub(crate) binding_pins: Box<[worth_ui_host_contract::UiGlyphRasterPinRequest]>,
}

pub(crate) enum UiNativePendingTextContinuation {
    AtlasReady,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UiNativeEffectPosture {
    #[default]
    BeforeEffects,
    Presentation(UiNativePresentationEffectPhase),
    PresentationIndeterminate,
    Presented,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePresentationEffectPhase {
    Prepared,
    SurfaceAcquired,
    Encoded,
    Submitted,
    PresentHandoff,
}

impl UiNativeHostState {
    pub(crate) fn presentation_access(&self) -> Option<UiNativePresentationAccess<'_>> {
        Some(UiNativePresentationAccess::new(
            self.device.as_ref()?,
            self.presentation_surface.as_ref()?,
        ))
    }

    pub(crate) fn new() -> Self {
        let mut state = Self {
            registrations: BTreeMap::new(),
            registration_resources: BTreeMap::new(),
            window: None,
            accepted_cursor: None,
            device: None,
            presentation_surface: None,
            last_retained_frame: None,
            retained_frame_observations: Vec::new(),
            resources: UiNativeResourceRegistry::new(),
            pending_presentations: Vec::new(),
            retained_draw_lists: BTreeMap::new(),
            presentation_epochs: BTreeMap::new(),
            semantic_focus: BTreeMap::new(),
            captures: super::capture::UiNativeCaptureState::default(),
            lifecycle: UiNativeLifecycleOrchestrator::new(),
            text_atlas: UiNativeTextAtlas::new(),
            text_atlas_gpu: None,
            text_atlas_in_flight: None,
            text_atlas_recovery: None,
            text_atlas_completion: None,
            text_pins_by_binding: BTreeMap::new(),
            pending_text_presentations: BTreeMap::new(),
            physical_signal: UiNativePhysicalSignalOwner::new(),
            peak_census: UiNativeResourceCensus::default(),
            peak_text_pins: Box::new([]),
            text_pin_frame_counts: Vec::new(),
            text_pin_frame_observations: Vec::new(),
            text_atlas_model_frame_digests: Vec::new(),
            text_atlas_plan_observations: Vec::new(),
            observation_history_overflowed: false,
            #[cfg(feature = "certification-support")]
            qualification: UiNativeQualificationState::ordinary(),
        };
        state.record_compiler_total_peak();
        state
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn new_for_certification(plan: crate::UiNativeQualificationPlan) -> Self {
        let mut state = Self::new();
        state.qualification = UiNativeQualificationState::from_plan(plan);
        state
    }

    pub(crate) fn record_compiler_total_peak(&mut self) {
        let current = self
            .resources
            .current()
            .with_text_atlas(self.text_atlas_census())
            .with_physical_signal(self.physical_signal.observation())
            .with_host_state(self);
        self.peak_census = self.peak_census.max(self.resources.peak()).max(current);
        let pins = self.text_atlas.pin_observations();
        if pins.len() > self.peak_text_pins.len() {
            self.peak_text_pins = pins;
        }
    }

    pub(crate) fn compiler_total_peak(&self) -> UiNativeResourceCensus {
        let current = self
            .resources
            .current()
            .with_text_atlas(self.text_atlas_census())
            .with_physical_signal(self.physical_signal.observation())
            .with_host_state(self);
        self.peak_census.max(self.resources.peak()).max(current)
    }

    pub(crate) const fn certified_derived_state_reconstruction(
        &self,
    ) -> Option<crate::UiNativeDerivedStateReconstructionObservation> {
        #[cfg(feature = "certification-support")]
        {
            self.qualification
                .derived_state_reconstruction_observation()
        }
        #[cfg(not(feature = "certification-support"))]
        {
            None
        }
    }

    pub(crate) fn record_text_pin_frame_observation(&mut self) {
        let observations = self.text_atlas.pin_observations();
        self.make_observation_history_room();
        self.text_pin_frame_counts
            .push(u32::try_from(observations.len()).unwrap_or(u32::MAX));
        self.text_pin_frame_observations.push(observations);
        self.text_atlas_model_frame_digests
            .push(self.text_atlas.semantic_model_digest());
    }

    pub(crate) fn record_retained_frame_observation(
        &mut self,
        observation: UiNativeRetainedFrameObservation,
    ) {
        self.last_retained_frame = Some(observation.clone());
        if self.retained_frame_observations.len() == NATIVE_OBSERVATION_HISTORY_CAPACITY {
            self.retained_frame_observations.remove(0);
            self.observation_history_overflowed = true;
        }
        self.retained_frame_observations.push(observation);
    }

    pub(crate) fn record_text_atlas_plan_observation(
        &mut self,
        observation: super::text_atlas::UiNativeTextAtlasPlanObservation,
    ) {
        if self.text_atlas_plan_observations.len() == NATIVE_OBSERVATION_HISTORY_CAPACITY {
            self.text_atlas_plan_observations.remove(0);
            self.observation_history_overflowed = true;
        }
        self.text_atlas_plan_observations.push(observation);
    }

    fn make_observation_history_room(&mut self) {
        if self.text_pin_frame_counts.len() < NATIVE_OBSERVATION_HISTORY_CAPACITY {
            return;
        }
        self.text_pin_frame_counts.remove(0);
        self.text_pin_frame_observations.remove(0);
        self.text_atlas_model_frame_digests.remove(0);
        self.observation_history_overflowed = true;
    }

    pub(crate) fn current_resource_census(&self) -> UiNativeResourceCensus {
        self.resources
            .current()
            .with_text_atlas(self.text_atlas_census())
            .with_physical_signal(self.physical_signal.observation())
            .with_host_state(self)
    }

    pub(crate) fn require_surface_reconstruction(&mut self, cause: super::UiNativeRecoveryCause) {
        self.captures.invalidate_all_sources();
        let bindings = self.registrations.keys().copied().collect::<Vec<_>>();
        self.lifecycle.require_recovery_for(bindings, cause);
    }

    pub(crate) fn observe_surface_basis_transition(
        &mut self,
        transition: super::UiNativeSurfaceBasisTransition,
    ) -> super::UiNativeLifecycleDirective {
        self.captures.invalidate_all_sources();
        let bindings = self.registrations.keys().copied().collect::<Vec<_>>();
        self.lifecycle
            .observe_surface_transition(transition, bindings)
    }

    pub(crate) fn close(&mut self) -> UiNativeResourceCensus {
        super::lifecycle::progress_shutdown(self)
    }

    pub(crate) fn progress_one_physical_signal_ready(&mut self) -> bool {
        self.progress_one_physical_signal_ready_outcome()
            != UiNativeHostPhysicalProgress::NoProgress
    }

    pub(super) fn progress_one_physical_signal_ready_outcome(
        &mut self,
    ) -> UiNativeHostPhysicalProgress {
        match self.physical_signal.next_ready_work() {
            Some(super::physical_work_signal::UiNativePhysicalSignalWork::AtlasUpload(
                identity,
            )) => match self.progress_text_atlas_physical(identity.pending()) {
                text_atlas_lifecycle::UiNativeTextAtlasPhysicalProgress::Terminal => {
                    UiNativeHostPhysicalProgress::TextAtlas(identity.request())
                }
                text_atlas_lifecycle::UiNativeTextAtlasPhysicalProgress::Pending
                | text_atlas_lifecycle::UiNativeTextAtlasPhysicalProgress::NoProgress => {
                    UiNativeHostPhysicalProgress::NoProgress
                }
            },
            Some(super::physical_work_signal::UiNativePhysicalSignalWork::Presentation(
                identity,
            )) => UiNativeHostPhysicalProgress::Presentation(
                identity,
                self.progress_pending_presentation(identity),
            ),
            Some(super::physical_work_signal::UiNativePhysicalSignalWork::AtlasPlanning(_))
            | None => UiNativeHostPhysicalProgress::NoProgress,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiNativeHostPhysicalProgress {
    NoProgress,
    TextAtlas(super::physical_work_signal::UiNativePhysicalAtlasRequestIdentity),
    Presentation(
        super::physical_work_signal::UiNativePhysicalPresentationIdentity,
        UiNativePresentationPhysicalProgress,
    ),
}
