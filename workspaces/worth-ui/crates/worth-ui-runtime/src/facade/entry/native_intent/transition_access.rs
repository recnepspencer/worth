use super::{
    WorthUiNativeIntentAttemptPrepared, WorthUiNativeIntentConfirmationRequired,
    WorthUiNativeIntentIngress, WorthUiNativeIntentPosture, WorthUiNativeIntentStop,
    WorthUiNativeIntentStopped, WorthUiNativeIntentTransition,
};

impl WorthUiNativeIntentIngress {
    pub(super) fn deferred(stop: super::WorthUiNativeInteractionIngressStop) -> Self {
        Self {
            transitions: Box::new([]),
            dismissals: Box::new([]),
            duplicate_batches: 0,
            interaction_stops: Box::new([stop]),
        }
    }

    pub fn into_interaction_stops(self) -> Box<[super::WorthUiNativeInteractionIngressStop]> {
        self.interaction_stops
    }
    pub fn transitions(&self) -> &[WorthUiNativeIntentTransition] {
        &self.transitions
    }

    pub fn into_transitions(self) -> Box<[WorthUiNativeIntentTransition]> {
        self.transitions
    }

    pub fn dismissals(&self) -> &[crate::facade::interaction::UiDismissInteraction] {
        &self.dismissals
    }

    pub const fn duplicate_batches(&self) -> usize {
        self.duplicate_batches
    }

    pub fn interaction_stops(&self) -> &[super::WorthUiNativeInteractionIngressStop] {
        &self.interaction_stops
    }
}

impl WorthUiNativeIntentAttemptPrepared {
    pub const fn dispatch(&self) -> crate::facade::intent::UiIntentExecutionDispatchReceipt {
        self.dispatch
    }

    pub fn into_posture(self) -> WorthUiNativeIntentPosture {
        self.posture
    }
}

impl WorthUiNativeIntentConfirmationRequired {
    pub const fn pending(&self) -> &crate::facade::intent::UiPendingIntentConfirmation {
        &self.pending
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::facade::intent::UiPendingIntentConfirmation,
        WorthUiNativeIntentPosture,
    ) {
        (self.pending, self.posture)
    }
}

impl WorthUiNativeIntentStopped {
    pub const fn stop(&self) -> &WorthUiNativeIntentStop {
        &self.stop
    }

    pub fn into_parts(self) -> (WorthUiNativeIntentStop, Option<WorthUiNativeIntentPosture>) {
        (self.stop, self.posture)
    }
}
