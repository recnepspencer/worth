#[path = "unpublished_validation/identity.rs"]
mod identity;
#[path = "unpublished_validation/mechanics.rs"]
mod mechanics;
#[path = "unpublished_validation/text.rs"]
mod text;

use super::UiUnpublishedAppearanceFrameProjectionDenial;

pub(super) fn validate_fragment(
    fragment: &super::UiUnpublishedAppearanceFragment,
    frame: crate::UiMountedFrameIdentity,
    presentation: crate::UiMountedPresentationAttemptIdentity,
) -> Result<(), UiUnpublishedAppearanceFrameProjectionDenial> {
    let work_frame = fragment.work.successor().frame();
    let surface = fragment.work.successor().semantic_surface();
    if work_frame != frame {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FrameMismatch);
    }
    if fragment.work.successor().overlay_order().presentation() != presentation {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::PresentationMismatch);
    }
    if fragment.presentation_affinity.successor() != frame
        || fragment.presentation_affinity.predecessor() != fragment.work.predecessor()
        || fragment.presentation_affinity.surface() != surface
        || fragment.presentation_affinity.binding() != fragment.surface_binding.binding()
        || fragment.presentation_affinity.baseline() != fragment.surface_binding.baseline()
        || fragment.presentation_affinity.content().diagnostic_value() == 0
    {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::PresentationAffinityMismatch);
    }
    if fragment.surface_binding.semantic_surface() != surface {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch);
    }
    identity::validate(fragment, frame, surface)?;
    text::validate(fragment, frame, surface)?;
    Ok(())
}

pub(super) fn fragment_mechanic_identities(
    fragment: &super::UiUnpublishedAppearanceFragment,
) -> Vec<crate::UiMountedAppearanceMechanicIdentity> {
    mechanics::identities(fragment)
}
