//! The presentation basis scroll chrome is derived and painted against: the
//! mounted pose offset of one region occurrence, and the device grid its
//! surface is bound at.
//!
//! Scroll chrome reports where the content already is, not where it is
//! heading, so the thumb has to read the pose mounted geometry has actually
//! applied. That pose is written by whichever lane settles the offset — the
//! immediate one directly, the accepted-sample one from the Motion sample a
//! witness displayed — which makes it the one place both policies agree on.
//!
//! A region whose pose has never been applied has never moved, so it reports
//! the origin rather than nothing: a scrollbar at rest still has a thumb.

impl super::WorthUiMountedSessionState {
    /// The offset the mounted pose of `owner_instance` was built from.
    /// `None` only when the occurrence is not mounted on a bound surface.
    pub(crate) fn mounted_scroll_pose(
        &self,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::runtime::scroll::UiScrollOffset> {
        let instance = self.identity.projection_instance(target)?;
        Some(
            self.occurrence_geometry
                .applied_scroll_pose(instance.basis().semantic_surface_identity(), owner_instance)
                .unwrap_or_else(crate::runtime::scroll::UiScrollOffset::origin),
        )
    }
    /// The device grid `surface` is presented on, for snapping painted chrome.
    /// `None` when the surface has no current binding.
    pub(crate) fn scroll_chrome_device_scale(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<crate::runtime::scroll::UiScrollPresentationDeviceScale> {
        let (binding, _) = self.identity.projection_surface(surface)?;
        crate::runtime::scroll::UiScrollPresentationDeviceScale::admit(
            binding.profile().device_scale_milli(),
        )
        .ok()
    }
}
