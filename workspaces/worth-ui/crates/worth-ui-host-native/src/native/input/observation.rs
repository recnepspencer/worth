use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiHostKeyboardModifiers, UiHostObservationPresentationBasis, UiHostObservationRetention,
    UiHostObservationSessionRegistrationDenial, UiHostPresentationEpoch, UiHostProtocolAgreement,
};

mod admission;
mod recipient;
mod recovery;
mod retention;
pub use recovery::{UiNativeInputRecoveryAcknowledgement, UiNativeInputRecoveryGrant};

const INPUT_OBSERVATION_HISTORY_CAPACITY: usize = 64;

use super::pointer;
use super::profile::{event_profile, UiNativeEventProfile};
pub use super::report::{
    UiNativeInputObservationEventFamily, UiNativeInputObservationReport,
    UiNativeInputObservationStop,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeInputObservationDisposition {
    Ignored,
    Retained,
    Stopped,
}

#[derive(Clone, Copy)]
struct UiNativePendingPresentationContext {
    protocol: UiHostProtocolAgreement,
    host_session: u64,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
}

pub(crate) struct UiNativeInputObservationState {
    pub(super) retention: UiHostObservationRetention,
    pub(super) active_host_session: Option<u64>,
    pub(super) completed: Option<(
        UiHostProtocolAgreement,
        u64,
        UiHostObservationPresentationBasis,
    )>,
    pub(super) input_recipient: Option<worth_ui_host_contract::UiHostInputRecipientBindingReceipt>,
    pending_presentations: BTreeMap<u64, UiNativePendingPresentationContext>,
    pub(super) profile: Option<UiNativeEventProfile>,
    pub(super) profile_requires_completion: bool,
    pub(super) profile_observation_pending: bool,
    pub(super) profile_transition_tick: Option<u64>,
    pub(super) ime_composition_active: bool,
    pub(super) ime_enabled: bool,
    pub(super) pointer: pointer::UiNativePointerState,
    pub(super) modifiers: UiHostKeyboardModifiers,
    pub(super) next_sequence: Option<u64>,
    pub(super) next_revision: Option<u64>,
    pub(super) completed_presentation_count: usize,
    pub(super) event_tick: u64,
    pub(super) stops: Vec<UiNativeInputObservationStop>,
    pub(super) terminal_stop: Option<UiNativeInputObservationStop>,
    pub(super) stop_history_complete: bool,
    pub(super) evidence: super::evidence::UiNativeInputObservationEvidence,
}

impl UiNativeInputObservationState {
    pub(crate) fn new() -> Self {
        Self {
            retention: UiHostObservationRetention::default(),
            active_host_session: None,
            completed: None,
            input_recipient: None,
            pending_presentations: BTreeMap::new(),
            profile: None,
            profile_requires_completion: false,
            profile_observation_pending: false,
            profile_transition_tick: None,
            ime_composition_active: false,
            ime_enabled: false,
            pointer: pointer::UiNativePointerState::new(),
            modifiers: UiHostKeyboardModifiers::default(),
            next_sequence: Some(1),
            next_revision: Some(1),
            completed_presentation_count: 0,
            event_tick: 0,
            stops: Vec::new(),
            terminal_stop: None,
            stop_history_complete: true,
            evidence: Default::default(),
        }
    }

    pub(crate) fn install_initial_profile(&mut self, scale_factor: f64, physical_size: [u32; 2]) {
        match event_profile(scale_factor, physical_size) {
            Ok(profile) => self.profile = Some(profile),
            Err(stop) => self.record_terminal_stop(stop),
        }
    }

    pub(crate) fn set_event_tick(&mut self, event_tick: u64) {
        self.event_tick = event_tick;
    }

    #[cfg(test)]
    pub(crate) fn observe_profile_transition(
        &mut self,
        scale_factor: f64,
        physical_size: [u32; 2],
    ) {
        self.observe_profile_transition_at(scale_factor, physical_size, self.event_tick);
    }

    pub(crate) fn observe_profile_transition_at(
        &mut self,
        scale_factor: f64,
        physical_size: [u32; 2],
        event_tick: u64,
    ) {
        self.event_tick = event_tick;
        if self
            .terminal_stop
            .is_some_and(|stop| !stop.permits_retention_recovery())
        {
            return;
        }
        let profile = match event_profile(scale_factor, physical_size) {
            Ok(profile) => profile,
            Err(stop) => {
                self.record_terminal_stop(stop);
                return;
            }
        };
        let changed = self.profile.is_none_or(|previous| {
            previous.scale_micros != profile.scale_micros
                || previous.physical_size != profile.physical_size
        });
        self.profile = Some(profile);
        if changed {
            self.profile_requires_completion = true;
            self.profile_observation_pending = false;
            self.profile_transition_tick = Some(event_tick);
        }
    }

    pub(crate) fn record_completed_presentation(
        &mut self,
        protocol: UiHostProtocolAgreement,
        host_session: u64,
        presentation: UiHostObservationPresentationBasis,
    ) -> bool {
        if !self.begin_host_session(host_session) {
            return false;
        }
        let Some(count) = self.completed_presentation_count.checked_add(1) else {
            self.record_terminal_stop(
                UiNativeInputObservationStop::CompletedPresentationCountExhausted,
            );
            return false;
        };
        self.completed = Some((protocol, host_session, presentation));
        self.completed_presentation_count = count;
        !matches!(
            self.emit_profile_transition(),
            UiNativeInputObservationDisposition::Stopped
        ) || self
            .terminal_stop
            .is_some_and(UiNativeInputObservationStop::permits_retention_recovery)
    }

    pub(crate) fn register_session(
        &self,
        host_session: u64,
    ) -> Result<(), UiHostObservationSessionRegistrationDenial> {
        self.retention.register_session(host_session)
    }

    pub(crate) fn remember_pending_presentation(
        &mut self,
        protocol: UiHostProtocolAgreement,
        host_session: u64,
        host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        completion_identity: u64,
    ) -> bool {
        if !self.begin_host_session(host_session) {
            return false;
        }
        if self
            .pending_presentations
            .contains_key(&completion_identity)
        {
            self.record_terminal_stop(
                UiNativeInputObservationStop::MissingPendingPresentationContext,
            );
            return false;
        }
        self.pending_presentations.insert(
            completion_identity,
            UiNativePendingPresentationContext {
                protocol,
                host_session,
                host_surface,
                binding,
            },
        );
        true
    }

    pub(crate) fn complete_pending_presentation(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        epoch: UiHostPresentationEpoch,
        completion_identity: u64,
    ) -> bool {
        let Some(context) = self.pending_presentations.remove(&completion_identity) else {
            self.record_terminal_stop(
                UiNativeInputObservationStop::MissingPendingPresentationContext,
            );
            return false;
        };
        if context.binding != binding {
            self.record_terminal_stop(UiNativeInputObservationStop::StalePresentationAffinity);
            return false;
        }
        self.record_completed_presentation(
            context.protocol,
            context.host_session,
            UiHostObservationPresentationBasis::new(context.host_surface, frame, binding, epoch),
        )
    }

    pub(crate) fn abandon_pending_presentation(
        &mut self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        completion_identity: Option<u64>,
    ) {
        if let Some(identity) = completion_identity {
            if self
                .pending_presentations
                .get(&identity)
                .is_some_and(|context| context.binding == binding)
            {
                self.pending_presentations.remove(&identity);
            }
            return;
        }
        self.pending_presentations
            .retain(|_, context| context.binding != binding);
    }

    pub(crate) fn has_pending_presentations(&self) -> bool {
        !self.pending_presentations.is_empty()
    }

    pub(crate) fn drain(
        &self,
        host_session_identity: u64,
    ) -> worth_ui_host_contract::UiHostObservationDrain {
        self.retention.drain(host_session_identity)
    }

    pub(crate) fn has_retained_observations(&self) -> bool {
        self.retention.pending_batch_count() != 0
    }

    pub(crate) fn observation_drain_capacity_reached(&self) -> bool {
        self.retention.pending_batch_count()
            >= worth_ui_host_contract::UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT
    }

    pub(crate) fn release_session(&mut self, host_session_identity: u64) {
        self.retention.release_session(host_session_identity);
        if self.active_host_session == Some(host_session_identity) {
            self.reset_session_state();
            self.active_host_session = None;
        } else {
            self.pending_presentations
                .retain(|_, context| context.host_session != host_session_identity);
        }
    }

    pub(crate) fn close(&mut self) {
        if let Some(host_session) = self.active_host_session {
            self.retention.release_session(host_session);
        }
        self.pending_presentations.clear();
        self.completed = None;
        self.input_recipient = None;
        self.profile = None;
        self.active_host_session = None;
        self.profile_requires_completion = false;
        self.profile_observation_pending = false;
        self.profile_transition_tick = None;
        self.ime_composition_active = false;
        self.ime_enabled = false;
        self.pointer.reset();
        self.modifiers = UiHostKeyboardModifiers::default();
        self.event_tick = 0;
    }

    pub(crate) fn report(&self) -> UiNativeInputObservationReport {
        UiNativeInputObservationReport {
            last_completed_presentation: self.completed.map(|(_, _, presentation)| presentation),
            completed_presentation_count: self.completed_presentation_count,
            stops: self.stops.clone().into_boxed_slice(),
            terminal_stop: self.terminal_stop,
            stop_history_complete: self.stop_history_complete,
            retained_batch_count: self.evidence.retained_batch_count(),
            coalesced_motion_batch_count: self.evidence.coalesced_motion_batch_count(),
            retained_event_count: self.evidence.retained_event_count(),
            first_retained_sequence: self.evidence.first_retained_sequence(),
            last_retained_sequence: self.evidence.last_retained_sequence(),
            family_counts: self.evidence.family_counts(),
            last_pointer_button: self.evidence.last_pointer_button(),
            last_vertical_scroll: self.evidence.last_vertical_scroll(),
            last_horizontal_scroll: self.evidence.last_horizontal_scroll(),
            profile_transition_count: self.evidence.profile_transition_count(),
        }
    }

    pub(super) fn take_revision(&mut self) -> Option<u64> {
        let value = self.next_revision?;
        self.next_revision = value.checked_add(1);
        Some(value)
    }

    pub(super) fn record_terminal_stop(&mut self, stop: UiNativeInputObservationStop) {
        if self
            .terminal_stop
            .is_none_or(UiNativeInputObservationStop::permits_retention_recovery)
        {
            self.terminal_stop = Some(stop);
        }
        self.record_stop(stop);
    }

    pub(super) fn record_stop(&mut self, stop: UiNativeInputObservationStop) {
        if self.stops.len() == INPUT_OBSERVATION_HISTORY_CAPACITY {
            self.stops.remove(0);
            self.stop_history_complete = false;
        }
        self.stops.push(stop);
    }

    pub(super) fn begin_host_session(&mut self, host_session: u64) -> bool {
        if !self.retention.is_session_active(host_session) {
            self.record_terminal_stop(UiNativeInputObservationStop::Retention(
                worth_ui_host_contract::UiHostObservationRetentionDenial::InactiveSession,
            ));
            return false;
        }
        match self.active_host_session {
            None => {
                let profile = self.profile;
                self.reset_session_state();
                self.profile = profile;
                self.active_host_session = Some(host_session);
                true
            }
            Some(active) if active == host_session => true,
            Some(_) => {
                self.record_terminal_stop(UiNativeInputObservationStop::StalePresentationAffinity);
                false
            }
        }
    }

    pub(super) fn reset_session_state(&mut self) {
        self.completed = None;
        self.input_recipient = None;
        self.pending_presentations.clear();
        self.profile = None;
        self.profile_requires_completion = false;
        self.profile_observation_pending = false;
        self.profile_transition_tick = None;
        self.ime_composition_active = false;
        self.ime_enabled = false;
        self.pointer.reset();
        self.modifiers = UiHostKeyboardModifiers::default();
        self.next_sequence = Some(1);
        self.next_revision = Some(1);
        self.completed_presentation_count = 0;
        self.event_tick = 0;
        self.stops.clear();
        self.terminal_stop = None;
        self.stop_history_complete = true;
        self.evidence = Default::default();
    }
}

#[cfg(test)]
mod phase6_tests;
#[cfg(test)]
mod tests;
