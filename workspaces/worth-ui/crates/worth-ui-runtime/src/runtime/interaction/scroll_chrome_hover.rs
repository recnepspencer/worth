//! The last pointer position each surface saw, kept for chrome hover.
//!
//! Hover over a scrollbar is re-resolved from the accepted displayed offset on
//! every presentation, not remembered from the frame the pointer last moved in:
//! a bar that scrolls under a stationary pointer must light up the part now
//! beneath it. That re-resolution needs the pointer's position, and nothing
//! else in the runtime keeps one per surface, so this slot does. It holds a
//! point and the binding it arrived under and nothing more; what is under the
//! point is decided by whoever asks.

use std::collections::BTreeMap;
use worth_ui_host_contract::{UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration};

#[derive(Debug, Default)]
pub(crate) struct UiScrollChromeHoverState {
    by_surface: BTreeMap<
        UiSemanticSurfaceIdentity,
        (
            UiSurfaceBindingGeneration,
            crate::mounting::presentation::UiPlatformPoint,
        ),
    >,
}

impl UiScrollChromeHoverState {
    /// Record where the pointer last was on `surface`, in logical points.
    pub(crate) fn observe(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        point: crate::mounting::presentation::UiPlatformPoint,
    ) {
        self.by_surface.insert(surface, (binding, point));
    }

    /// The pointer's last known position on `surface`, if it has had one.
    pub(crate) fn point(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<crate::mounting::presentation::UiPlatformPoint> {
        self.by_surface.get(&surface).map(|(_, point)| *point)
    }

    /// Forget positions that arrived under a binding the host has since
    /// replaced; the next report under the new binding supplies a fresh one.
    pub(crate) fn clear_binding(&mut self, binding: UiSurfaceBindingGeneration) {
        self.by_surface.retain(|_, (held, _)| *held != binding);
    }

    pub(crate) fn clear_all(&mut self) {
        self.by_surface.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_latest_position_replaces_the_previous_one() {
        let mut hover = UiScrollChromeHoverState::default();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let first = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let second = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        hover.observe(
            first,
            binding,
            crate::mounting::presentation::platform_point_for_test(10.0, 20.0),
        );
        hover.observe(
            first,
            binding,
            crate::mounting::presentation::platform_point_for_test(30.0, 40.0),
        );
        assert_eq!(
            hover.point(first),
            Some(crate::mounting::presentation::platform_point_for_test(
                30.0, 40.0
            ))
        );
        assert_eq!(hover.point(second), None);
    }

    #[test]
    fn a_replaced_binding_forgets_only_its_own_positions() {
        let mut hover = UiScrollChromeHoverState::default();
        let old = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let new = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let first = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let second = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        hover.observe(
            first,
            old,
            crate::mounting::presentation::platform_point_for_test(1.0, 1.0),
        );
        hover.observe(
            second,
            new,
            crate::mounting::presentation::platform_point_for_test(2.0, 2.0),
        );
        hover.clear_binding(old);
        assert_eq!(hover.point(first), None);
        assert_eq!(
            hover.point(second),
            Some(crate::mounting::presentation::platform_point_for_test(
                2.0, 2.0
            ))
        );
        hover.clear_all();
        assert_eq!(hover.point(second), None);
    }
}
