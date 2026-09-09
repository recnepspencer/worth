use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostSurfacePosition, UiSemanticSurfaceIdentity,
};

use super::{resolve_presented_target, UiInteractionTargetingDenial, UiPresentedInteractionTarget};

/// A current presentation position retains surface affinity even outside every hit region.
pub(crate) struct UiPresentedPointerPosition {
    presentation: UiHostObservationPresentationBasis,
    position: UiHostSurfacePosition,
    surface: UiSemanticSurfaceIdentity,
    target: Option<UiPresentedInteractionTarget>,
}

impl UiPresentedPointerPosition {
    pub(crate) fn resolve(
        mounted: &crate::mounting::WorthUiMountedSessionState,
        presentation: UiHostObservationPresentationBasis,
        position: UiHostSurfacePosition,
    ) -> Result<Self, UiInteractionTargetingDenial> {
        let surface = current_pointer_surface(mounted, presentation)?;
        let target = match resolve_presented_target(mounted, presentation, position) {
            Ok(target) => Some(target),
            Err(UiInteractionTargetingDenial::NoTarget { .. }) => None,
            Err(denial) => return Err(denial),
        };
        Ok(Self {
            presentation,
            position,
            surface,
            target,
        })
    }

    pub(crate) const fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.presentation
    }

    pub(crate) const fn position(&self) -> UiHostSurfacePosition {
        self.position
    }

    pub(crate) const fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }

    pub(crate) fn target(&self) -> Option<&UiPresentedInteractionTarget> {
        self.target.as_ref()
    }
}

pub(crate) fn current_pointer_surface(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: UiHostObservationPresentationBasis,
) -> Result<UiSemanticSurfaceIdentity, UiInteractionTargetingDenial> {
    mounted
        .current_semantic_surface_for_presentation(presentation)
        .map_err(super::presented_frame::map_presentation_denial)
}
