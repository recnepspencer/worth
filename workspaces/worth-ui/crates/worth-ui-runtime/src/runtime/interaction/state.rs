use worth_ui_host_contract::{UiHostObservationFamily, UiSurfaceBindingGeneration};

use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use super::draft::UiDraftRuntimeState;
use super::gesture::{UiPointerGestureRuntimeState, UiPointerGestureStopReason};
use super::{
    UiActivateInteraction, UiInteractionLifecycleSettlementReceipt, UiInteractionShutdownReport,
    UiInteractionStateSnapshot, UiLocalInputRecipientAdmission, UiLocalInputRecipientBindingStop,
    UiLocalInputRecipientContract, UiLocalInputStopReason,
};

mod application_rebind;
mod presentation_refresh;
pub(crate) use application_rebind::UiPreparedInteractionApplicationRebind;

#[path = "state_ingress.rs"]
mod ingress;
#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;

pub(crate) struct UiInteractionRuntimeState {
    pointer: UiPointerGestureRuntimeState,
    pointer_presence: Option<super::pointer_presence::UiPointerPresenceOwner>,
    pointer_presence_capacity: super::pointer_presence::UiPointerPresenceCapacity,
    draft: UiDraftRuntimeState,
    semantic_interactions: u64,
    presentation_refresh: Option<super::UiInteractionPresentationRefreshSnapshot>,
    application_generation: worth_ui_host_contract::UiHostApplicationGeneration,
    scroll_chrome_hover: super::scroll_chrome_hover::UiScrollChromeHoverState,
    scroll_gesture_latch: super::scroll_gesture_latch::UiScrollGestureLatchState,
}

#[derive(Clone, Copy)]
pub(crate) enum UiInteractionLifecycleStopReason {
    ObservationQuarantined,
    ObservationInvalid,
    ObservationLoss {
        family: UiHostObservationFamily,
        affected: Option<worth_ui_host_contract::UiHostObservationSequenceRange>,
    },
    SurfaceRebound,
    MountedInstanceRemoved,
    ApplicationRebound,
    Shutdown,
}

impl UiInteractionRuntimeState {
    pub(crate) const fn pointer_presence_is_enabled(&self) -> bool {
        self.pointer_presence.is_some()
    }

    /// The scroll-chrome thumb drag in progress, if one is. Reading it is how
    /// wheel and keyboard lanes learn which axis a capture already owns.
    pub(crate) fn scroll_chrome_latch(&self) -> Option<super::gesture::UiScrollChromeLatch> {
        self.pointer.scroll_chrome_latch()
    }

    pub(crate) fn scroll_chrome_capture_identity(
        &self,
    ) -> Option<(
        worth_ui_host_contract::UiHostPointerIdentity,
        worth_ui_host_contract::UiHostPointerCaptureEpoch,
    )> {
        self.pointer.scroll_chrome_capture_identity()
    }

    pub(crate) fn scroll_chrome_pending_capture(
        &self,
    ) -> Option<super::gesture::UiScrollChromePendingCapture> {
        self.pointer.scroll_chrome_latch_state().pending()
    }

    /// Whether a thumb drag already owns this axis of this region, so a wheel
    /// or a key naming the same axis must be ignored for the length of the
    /// capture. The rule lives in the latch slot; this is the way in.
    pub(crate) fn scroll_chrome_suppresses_axis(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        axis: crate::runtime::scroll::chrome::UiScrollChromeAxis,
    ) -> bool {
        self.pointer
            .scroll_chrome_latch_state()
            .suppresses_axis(owner, axis)
    }

    /// The chrome latch slot. The press, move and release lane takes the latch
    /// here; every cancellation path already clears it through `pointer`.
    pub(crate) fn scroll_chrome_latch_mut(
        &mut self,
    ) -> &mut super::gesture::UiScrollChromeLatchState {
        self.pointer.scroll_chrome_latch_mut()
    }

    /// Where the pointer last was on `surface`, for re-resolving chrome hover
    /// against the accepted displayed offset at presentation time.
    pub(crate) fn scroll_chrome_hover_point(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<crate::mounting::presentation::UiPlatformPoint> {
        self.scroll_chrome_hover.point(surface)
    }

    pub(crate) fn observe_scroll_chrome_hover(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        point: crate::mounting::presentation::UiPlatformPoint,
    ) {
        self.scroll_chrome_hover.observe(surface, binding, point);
    }

    /// The scroll gesture latch still holding at `input_tick`, if one is.
    /// Routing reads it before it resolves an owner from pointer location.
    pub(crate) fn scroll_gesture_latch_at(
        &self,
        input_tick: u64,
    ) -> Option<crate::runtime::scroll::UiScrollGestureLatch> {
        self.scroll_gesture_latch.held_at(input_tick)
    }

    /// Take or carry forward the latch, after the route it describes has
    /// been committed. A gesture that consumed nothing takes none.
    pub(crate) fn latch_scroll_gesture(
        &mut self,
        latch: crate::runtime::scroll::UiScrollGestureLatch,
    ) {
        self.scroll_gesture_latch.latch(latch);
    }

    /// The host ended or cancelled the phase, so the latch ends with it.
    pub(crate) fn end_scroll_gesture_latch(&mut self) -> bool {
        self.scroll_gesture_latch.end()
    }

    pub(crate) fn new(
        pointer_presence_enabled: bool,
        pressed_appearance_enabled: bool,
        pointer_presence_capacity: super::pointer_presence::UiPointerPresenceCapacity,
    ) -> Self {
        Self {
            pointer: UiPointerGestureRuntimeState::new(pressed_appearance_enabled),
            pointer_presence: pointer_presence_enabled.then(|| {
                super::pointer_presence::UiPointerPresenceOwner::new(pointer_presence_capacity)
            }),
            pointer_presence_capacity,
            draft: UiDraftRuntimeState::new(),
            semantic_interactions: 0,
            presentation_refresh: None,
            application_generation: worth_ui_host_contract::UiHostApplicationGeneration::new(1)
                .expect("the initial interaction application generation is nonzero"),
            scroll_chrome_hover: Default::default(),
            scroll_gesture_latch: Default::default(),
        }
    }

    pub(crate) fn bind_local_recipient<Install>(
        &mut self,
        activation: UiActivateInteraction,
        context: super::draft::UiLocalInputRecipientBindingContext<'_>,
        contract: UiLocalInputRecipientContract,
        install: Install,
    ) -> Result<UiLocalInputRecipientAdmission, UiLocalInputRecipientBindingStop>
    where
        Install: FnOnce(worth_ui_host_contract::UiHostInputRecipientBindingReceipt) -> bool,
    {
        self.draft.bind(activation, context, contract, install)
    }

    pub(crate) fn bind_focused_submit<Install>(
        &mut self,
        target: super::UiPresentedInteractionTargetView,
        context: super::draft::UiLocalInputRecipientBindingContext<'_>,
        install: Install,
    ) -> Result<
        super::UiLocalInputRecipientBindingReceipt,
        super::UiLocalInputRecipientBindingStopReason,
    >
    where
        Install: FnOnce(worth_ui_host_contract::UiHostInputRecipientBindingReceipt) -> bool,
    {
        self.draft.bind_focused_submit(target, context, install)
    }

    pub(crate) fn clear_focused_recipient(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) {
        self.draft
            .cancel_instance(instance, super::UiLocalInputStopReason::RecipientReplaced);
    }

    pub(crate) const fn application_generation(
        &self,
    ) -> worth_ui_host_contract::UiHostApplicationGeneration {
        self.application_generation
    }

    pub(crate) fn active_input_binding(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostInputRecipientBindingReceipt> {
        self.draft.active_input_binding()
    }

    pub(crate) fn commit_selection(
        &mut self,
        activation: UiActivateInteraction,
        option: worth_ui_query_binding::UiProjectionOptionReference,
        generation: &WorthUiActiveApplicationGenerationIdentity,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<super::UiSelectionCommitInteraction, super::UiSelectionCommitStop> {
        let interaction =
            super::semantic::commit_selection(activation, option, generation, mounted)?;
        self.record_semantic();
        Ok(interaction)
    }

    pub(crate) fn snapshot(&self) -> UiInteractionStateSnapshot {
        UiInteractionStateSnapshot::from_parts(
            self.pointer.snapshot(),
            self.pointer_presence
                .as_ref()
                .map_or(0, |owner| owner.pointer_count()),
            self.draft.snapshot(),
            self.semantic_interactions,
        )
    }

    pub(crate) fn reconcile_pointer_observation_demand(&mut self, presence: bool, pressed: bool) {
        match (presence, self.pointer_presence.is_some()) {
            (true, false) => {
                self.pointer_presence = Some(super::pointer_presence::UiPointerPresenceOwner::new(
                    self.pointer_presence_capacity,
                ))
            }
            (false, true) => self.pointer_presence = None,
            _ => {}
        }
        self.pointer.reconcile_appearance_enabled(pressed);
    }

    #[allow(
        dead_code,
        reason = "Gate 1 retains committed presentation observation for later mounted cutover"
    )]
    /// Each owner admits its own batch. The caller must handle both outcomes independently.
    #[cfg(test)]
    pub(crate) fn observe_committed_presentation(
        &mut self,
        trigger: &super::pointer_presence::UiPointerPresencePresentationTrigger,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> (
        Result<usize, super::targeting::UiInteractionTargetingDenial>,
        Result<usize, super::targeting::UiInteractionTargetingDenial>,
    ) {
        let hover = self.pointer_presence.as_mut().map_or(Ok(0), |owner| {
            owner.retest_committed_presentation(trigger, mounted)
        });
        let pressed = self.pointer.retest_committed_presentation(trigger, mounted);
        (hover, pressed)
    }

    #[allow(
        dead_code,
        reason = "milestone 3.16 Gate 0 exposes the owner snapshot only to the sealed close-turn lane"
    )]
    pub(crate) fn pointer_presence_appearance_snapshot(
        &self,
    ) -> Option<super::UiPointerPresenceAppearanceOwnerSnapshot> {
        self.pointer_presence
            .as_ref()
            .map(super::pointer_presence::UiPointerPresenceOwner::appearance_snapshot)
    }

    #[allow(
        dead_code,
        reason = "milestone 3.16 Gate 0 exposes the owner snapshot only to the sealed close-turn lane"
    )]
    pub(crate) fn pressed_appearance_snapshot(
        &self,
    ) -> super::gesture::UiPressedAppearanceOwnerSnapshot {
        self.pointer.appearance_snapshot()
    }

    pub(crate) fn cancel_binding(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        reason: UiInteractionLifecycleStopReason,
    ) -> UiInteractionLifecycleSettlementReceipt {
        if let Some(owner) = self.pointer_presence.as_mut() {
            owner.cancel_binding(binding);
        }
        self.scroll_chrome_hover.clear_binding(binding);
        self.scroll_gesture_latch.clear_binding(binding);
        let pointer = self
            .pointer
            .cancel_binding(binding, reason.pointer_reason());
        let draft = self.draft.cancel_binding(binding, reason.local_reason());
        self.settlement(pointer, draft)
    }

    pub(crate) fn cancel_instance(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        reason: UiInteractionLifecycleStopReason,
    ) -> UiInteractionLifecycleSettlementReceipt {
        if let Some(owner) = self.pointer_presence.as_mut() {
            owner.cancel_instance(instance);
        }
        self.scroll_gesture_latch.clear_instance(instance);
        let pointer = self
            .pointer
            .cancel_instance(instance, reason.pointer_reason());
        let draft = self.draft.cancel_instance(instance, reason.local_reason());
        self.settlement(pointer, draft)
    }

    pub(crate) fn unchanged_settlement(&self) -> UiInteractionLifecycleSettlementReceipt {
        UiInteractionLifecycleSettlementReceipt::new(Vec::new(), Vec::new(), self.snapshot())
    }

    pub(crate) fn cancel_all(
        &mut self,
        reason: UiInteractionLifecycleStopReason,
    ) -> UiInteractionLifecycleSettlementReceipt {
        if let Some(owner) = self.pointer_presence.as_mut() {
            owner.cancel_all();
        }
        self.scroll_chrome_hover.clear_all();
        self.scroll_gesture_latch.clear_all();
        let pointer = self.pointer.cancel_all(reason.pointer_reason());
        let draft = self.draft.cancel_all(reason.local_reason());
        if matches!(reason, UiInteractionLifecycleStopReason::ApplicationRebound) {
            let next = self
                .application_generation
                .get()
                .checked_add(1)
                .and_then(worth_ui_host_contract::UiHostApplicationGeneration::new)
                .expect("bounded application generation exhausted");
            self.application_generation = next;
        }
        self.settlement(pointer, draft)
    }

    pub(crate) fn shutdown(&mut self) -> UiInteractionShutdownReport {
        UiInteractionShutdownReport {
            settlement: Some(self.cancel_all(UiInteractionLifecycleStopReason::Shutdown)),
        }
    }

    fn settlement(
        &self,
        pointer: Vec<super::UiPointerGestureStop>,
        draft: Vec<super::UiLocalInputStop>,
    ) -> UiInteractionLifecycleSettlementReceipt {
        UiInteractionLifecycleSettlementReceipt::new(pointer, draft, self.snapshot())
    }

    fn record_semantic(&mut self) {
        self.semantic_interactions = self
            .semantic_interactions
            .checked_add(1)
            .expect("bounded semantic interaction counter exhausted");
    }
}

impl UiInteractionLifecycleStopReason {
    fn pointer_reason(self) -> UiPointerGestureStopReason {
        match self {
            Self::ObservationQuarantined => UiPointerGestureStopReason::ObservationQuarantined,
            Self::ObservationInvalid => UiPointerGestureStopReason::InvalidObservation,
            Self::ObservationLoss {
                family: UiHostObservationFamily::PointerButton,
                affected: Some(affected),
            } => UiPointerGestureStopReason::PointerButtonLoss { affected },
            Self::ObservationLoss { .. } => UiPointerGestureStopReason::InvalidObservation,
            Self::SurfaceRebound => UiPointerGestureStopReason::SurfaceRebound,
            Self::MountedInstanceRemoved => UiPointerGestureStopReason::MountedInstanceRemoved,
            Self::ApplicationRebound => UiPointerGestureStopReason::ApplicationRebound,
            Self::Shutdown => UiPointerGestureStopReason::Shutdown,
        }
    }

    fn local_reason(self) -> UiLocalInputStopReason {
        match self {
            Self::ObservationQuarantined | Self::ObservationInvalid => {
                UiLocalInputStopReason::ObservationInvalid
            }
            Self::ObservationLoss { family, .. } => UiLocalInputStopReason::ObservationLoss(family),
            Self::SurfaceRebound => UiLocalInputStopReason::SurfaceRebound,
            Self::MountedInstanceRemoved => UiLocalInputStopReason::MountedInstanceRemoved,
            Self::ApplicationRebound => UiLocalInputStopReason::ApplicationRebound,
            Self::Shutdown => UiLocalInputStopReason::Shutdown,
        }
    }
}
