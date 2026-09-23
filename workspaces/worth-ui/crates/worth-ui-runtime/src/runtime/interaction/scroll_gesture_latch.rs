//! The slot that holds one scroll gesture's latch for the gesture's lifetime.
//!
//! Scroll decides which owner a gesture belongs to. Interaction owns how long
//! it keeps that owner, because a latch ends for the same reasons every other
//! gesture ends: the host ended or cancelled the phase, the surface was
//! rebound, the owner was unmounted, focus or modality moved away. Those are
//! lifecycle facts rather than scrolling ones, so the slot lives beside the
//! pointer gesture table and is cleared by the paths that already clear presses
//! and drags. Scrolling introduces no lifetime rule of its own.
//!
//! One latch at a time: there is one physical wheel or trackpad under the
//! reader's hand, and a second gesture on the same surface replaces the first
//! rather than running beside it.

use crate::runtime::scroll::UiScrollGestureLatch;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSurfaceBindingGeneration};

#[derive(Default)]
pub(crate) struct UiScrollGestureLatchState {
    held: Option<UiScrollGestureLatch>,
}

impl UiScrollGestureLatchState {
    /// The latch still holding at `tick`, if one is. A quiet-interval latch
    /// that has aged out is reported as gone rather than resolved: pointer
    /// location decides the next event.
    pub(crate) fn held_at(&self, tick: u64) -> Option<UiScrollGestureLatch> {
        self.held.filter(|latch| latch.is_live_at(tick))
    }

    /// Take or carry forward the latch. A gesture that takes one while another
    /// is held replaces it, which is what a new gesture means.
    pub(crate) fn latch(&mut self, latch: UiScrollGestureLatch) {
        self.held = Some(latch);
    }

    /// End whatever is held, and say whether anything was.
    pub(crate) fn end(&mut self) -> bool {
        self.held.take().is_some()
    }

    /// The surface binding went away, so any gesture latched under it did too.
    pub(crate) fn clear_binding(&mut self, binding: UiSurfaceBindingGeneration) {
        if self.held.is_some_and(|latch| latch.binding() == binding) {
            self.held = None;
        }
    }

    /// The mounted occurrence the latch named was removed.
    pub(crate) fn clear_instance(&mut self, instance: UiMountedInstanceIdentity) {
        if self
            .held
            .is_some_and(|latch| latch.mounted_instance() == instance)
        {
            self.held = None;
        }
    }

    pub(crate) fn clear_all(&mut self) {
        self.held = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{UiScrollGestureLatch, UiScrollGestureLatchState};
    use crate::runtime::scroll::{UiScrollGestureLatchLifetime, UiScrollOwnerIdentity};
    use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSurfaceBindingGeneration};

    fn owner() -> UiScrollOwnerIdentity {
        UiScrollOwnerIdentity::declared_region(
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface"),
            crate::graph::UiGraphNodeIdentity::new(3_161),
            1,
            4,
        )
    }

    fn latch(
        lifetime: UiScrollGestureLatchLifetime,
        input_tick: u64,
        instance: UiMountedInstanceIdentity,
        binding: UiSurfaceBindingGeneration,
    ) -> UiScrollGestureLatch {
        UiScrollGestureLatch::new(
            owner(),
            crate::runtime::scroll::UiScrollOwnerIncarnation::new(1).expect("incarnation"),
            instance,
            0,
            binding,
            lifetime,
            input_tick,
        )
    }

    fn instance() -> UiMountedInstanceIdentity {
        UiMountedInstanceIdentity::mint_unbound().expect("a mounted instance identity")
    }

    fn binding() -> UiSurfaceBindingGeneration {
        UiSurfaceBindingGeneration::mint_unbound().expect("a surface binding generation")
    }

    #[test]
    fn a_phased_gesture_holds_its_latch_until_it_is_ended() {
        let mut state = UiScrollGestureLatchState::default();
        state.latch(latch(
            UiScrollGestureLatchLifetime::PhasedGesture,
            10,
            instance(),
            binding(),
        ));
        assert!(
            state.held_at(10_000).is_some(),
            "no amount of elapsed time ends a phase the host has not ended"
        );
        assert!(state.end());
        assert!(state.held_at(10).is_none());
        assert!(!state.end(), "ending twice ends nothing the second time");
    }

    #[test]
    fn an_unphased_wheel_latch_expires_a_quiet_interval_after_its_latest_event() {
        let mut state = UiScrollGestureLatchState::default();
        let held = latch(
            UiScrollGestureLatchLifetime::QuietInterval { quiet_ticks: 6 },
            10,
            instance(),
            binding(),
        );
        state.latch(held);
        assert!(
            state.held_at(16).is_some(),
            "the last live tick still holds"
        );
        assert!(
            state.held_at(17).is_none(),
            "one tick past the quiet interval the pointer decides again"
        );

        // A further notch inside the interval carries the same latch forward.
        state.latch(held.refreshed(16));
        assert!(state.held_at(22).is_some());
        assert!(state.held_at(23).is_none());
    }

    #[test]
    fn a_latch_ends_with_the_binding_or_the_occurrence_it_named() {
        let held_instance = instance();
        let held_binding = binding();
        let held = latch(
            UiScrollGestureLatchLifetime::PhasedGesture,
            1,
            held_instance,
            held_binding,
        );
        let mut state = UiScrollGestureLatchState::default();

        state.latch(held);
        state.clear_binding(binding());
        state.clear_instance(instance());
        assert!(
            state.held_at(1).is_some(),
            "another surface and another occurrence are not this gesture"
        );

        state.clear_instance(held_instance);
        assert!(state.held_at(1).is_none());

        state.latch(held);
        state.clear_binding(held_binding);
        assert!(state.held_at(1).is_none());

        state.latch(held);
        state.clear_all();
        assert!(state.held_at(1).is_none());
    }
}
