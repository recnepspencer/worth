use super::extent::{UiOverlayMotionSnapshot, UiOverlaySurfaceExtentSnapshot};
use super::planner::{UiOverlayCompositionInput, UiOverlayPortalBinding};
use super::snapshot::UiOverlayApplicationGeneration;

pub(crate) struct UiOverlayCompositionOwnerInput<'a> {
    generation: UiOverlayApplicationGeneration,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    extent: &'a UiOverlaySurfaceExtentSnapshot,
    portal_owner: &'a crate::runtime::portal::UiPortalRuntimeState,
    portal_bindings: &'a [UiOverlayPortalBinding],
    motion: Option<&'a UiOverlayMotionSnapshot>,
}

impl<'a> UiOverlayCompositionOwnerInput<'a> {
    pub(super) fn new(
        generation: UiOverlayApplicationGeneration,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        extent: &'a UiOverlaySurfaceExtentSnapshot,
        portal_owner: &'a crate::runtime::portal::UiPortalRuntimeState,
        portal_bindings: &'a [UiOverlayPortalBinding],
        motion: Option<&'a UiOverlayMotionSnapshot>,
    ) -> Self {
        Self {
            generation,
            presentation,
            extent,
            portal_owner,
            portal_bindings,
            motion,
        }
    }

    pub(super) fn with_composition_input<T>(
        &self,
        compose: impl FnOnce(UiOverlayCompositionInput<'_>) -> T,
    ) -> T {
        let portal_snapshot = self.portal_owner.stack_snapshot();
        compose(UiOverlayCompositionInput::from_sealed_owner_snapshot(
            self.generation.clone(),
            self.presentation,
            self.extent,
            &portal_snapshot,
            self.portal_bindings,
            self.motion,
        ))
    }
}
