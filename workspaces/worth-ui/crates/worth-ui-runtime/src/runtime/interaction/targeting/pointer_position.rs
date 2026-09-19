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
    /// Event-time evidence admits the coordinates; Hover is resolved against
    /// the currently presented geometry on the same live physical binding.
    /// Gesture targeting continues to consume the original event presentation.
    pub(crate) fn resolve_observation(
        mounted: &crate::mounting::WorthUiMountedSessionState,
        observed: UiHostObservationPresentationBasis,
        position: UiHostSurfacePosition,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Result<Self, UiInteractionTargetingDenial> {
        mounted
            .classify_interaction_presentation(observed)
            .map_err(super::presented_frame::map_presentation_denial)?;
        let surface = mounted
            .current_surface_for_binding(observed.binding())
            .ok_or(UiInteractionTargetingDenial::BindingNoLongerCurrent)?;
        let current = mounted
            .current_presentation_for_surface(surface)
            .ok_or(UiInteractionTargetingDenial::PresentationTruthUnavailable)?;
        if current.binding() != observed.binding()
            || current.host_surface() != observed.host_surface()
        {
            return Err(UiInteractionTargetingDenial::MountedSurfaceAffinityChanged);
        }
        Self::resolve(mounted, current, position, work)
    }

    pub(crate) fn resolve(
        mounted: &crate::mounting::WorthUiMountedSessionState,
        presentation: UiHostObservationPresentationBasis,
        position: UiHostSurfacePosition,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Result<Self, UiInteractionTargetingDenial> {
        let surface = current_pointer_surface(mounted, presentation)?;
        let target = match resolve_presented_target(mounted, presentation, position, work) {
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
