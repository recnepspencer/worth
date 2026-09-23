//! What the facade derives for one presentation attempt beyond the node tree.
//!
//! Overlays and scroll chrome are both paint that no mounted node authored:
//! overlays come from the portal and backdrop world, chrome from the accepted
//! scroll pose and the pointer's posture over it. Both are resolved against the
//! attempt's own theme binding inside the presentation closure, and both reach
//! frame lowering through this one value so a theme switch repaints them in the
//! same frame it repaints the nodes.

/// Everything one attempt's closure derived for appearance lowering.
#[derive(Default)]
pub(crate) struct UiMountedAppearanceDerivedInput {
    pub(crate) scroll_geometry_reservations:
        std::collections::BTreeMap<worth_ui_host_contract::UiSemanticSurfaceIdentity, usize>,
    pub(crate) overlays: Vec<super::UiMountedAppearanceSurfaceOverlayInput>,
    pub(crate) scroll_chrome: Vec<super::UiMountedAppearanceScrollChromeInput>,
    pub(crate) scroll_motion:
        Vec<crate::mounting::presentation::work_producer::UiMountedScrollMotionGroupInput>,
}
