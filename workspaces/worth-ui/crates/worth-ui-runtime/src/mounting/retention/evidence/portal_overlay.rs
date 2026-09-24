//! Reading the Portal overlay a retained frame showed at an epoch it presented.

use super::{UiPresentedFrameBasisDenial, UiRetainedPresentedFrame};

impl UiRetainedPresentedFrame {
    /// The Portal overlay this frame showed at `presentation`. A frame keeps
    /// its overlays for every epoch it presented, so an epoch the host stamped
    /// on input before this frame's latest one still reads the same overlay.
    pub(crate) fn presented_portal_overlay(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Result<
        Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
        UiPresentedFrameBasisDenial,
    > {
        let binding = presentation.binding();
        let retained = self
            .presentation_bindings
            .binary_search_by_key(&binding, |entry| entry.binding)
            .ok()
            .map(|index| &self.presentation_bindings[index])
            .ok_or(UiPresentedFrameBasisDenial::BindingNotPresented)?;
        retained.require_host(presentation)?;
        if presentation.epoch() > retained.epoch {
            return Err(UiPresentedFrameBasisDenial::PresentationEpochMismatch);
        }
        Ok(self
            .visual_region_basis(binding)
            .presented_portal_overlay(target))
    }
}
