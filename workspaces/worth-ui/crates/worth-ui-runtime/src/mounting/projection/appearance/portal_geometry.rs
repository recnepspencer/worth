use super::{UiMountedAppearanceClip as Clip, UiMountedAppearanceGeometryDenial as Denial};
use worth_ui_host_contract::{
    UiAppearanceClip, UiMountedAllocationProjection, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedPortalOverlayMechanic,
};

/// Rebase completed host-surface occurrence geometry from the exact anchor
/// neighborhood into the Portal's presented neighborhood.
pub(in crate::mounting::projection) fn portal_presented_allocation(
    allocation: UiMountedAllocationProjection,
    portal: UiMountedPortalOverlayMechanic,
) -> Result<UiMountedAllocationProjection, Denial> {
    let (bounds, basis, anchor) = match allocation {
        UiMountedAllocationProjection::Known { bounds, basis } => (bounds, basis, false),
        UiMountedAllocationProjection::PortalAnchorObservation { bounds, basis } => {
            (bounds, basis, true)
        }
        UiMountedAllocationProjection::Omitted(reason) => {
            return Ok(UiMountedAllocationProjection::Omitted(reason));
        }
    };
    let bounds = translate_box(bounds, portal)?;
    Ok(if anchor {
        UiMountedAllocationProjection::PortalAnchorObservation { bounds, basis }
    } else {
        UiMountedAllocationProjection::Known { bounds, basis }
    })
}

/// Portal bounds constrain translated ancestor coverage, never the child's own
/// allocation.
pub(in crate::mounting::projection) fn portal_ancestor_clip(
    ancestry: Clip,
    portal: UiMountedPortalOverlayMechanic,
) -> Result<Clip, Denial> {
    if matches!(ancestry, Clip::Unresolved(denial)
        if !matches!(denial, super::UiMountedAppearanceClipDenial::PortalBindingUnavailable(_)))
    {
        return Ok(ancestry);
    }
    let bounds = canonical_clip(portal.bounds())?;
    let clip = canonical_clip(portal.clip_bounds())?;
    let (Some(bounds), Some(clip)) = (bounds, clip) else {
        return Ok(Clip::Suppressed);
    };
    let Some(coverage) = super::clip::intersect_clips(bounds, clip) else {
        return Ok(Clip::Suppressed);
    };
    match ancestry {
        Clip::Ancestor(ancestor) => {
            let ancestor = translate_clip(ancestor, portal)?;
            Ok(super::clip::intersect_clips(coverage, ancestor)
                .map_or(Clip::Suppressed, Clip::Ancestor))
        }
        Clip::Suppressed => Ok(Clip::Suppressed),
        Clip::Unclipped
        | Clip::Unresolved(super::UiMountedAppearanceClipDenial::PortalBindingUnavailable(_)) => {
            Ok(Clip::Ancestor(coverage))
        }
        Clip::Unresolved(_) => Ok(ancestry),
    }
}

fn translate_box(
    bounds: UiMountedCanonicalBox,
    portal: UiMountedPortalOverlayMechanic,
) -> Result<UiMountedCanonicalBox, Denial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: bounds.x() + portal.bounds().x() - portal.anchor_bounds().x(),
        y: bounds.y() + portal.bounds().y() - portal.anchor_bounds().y(),
        width: bounds.width(),
        height: bounds.height(),
        coordinate_space: bounds.coordinate_space(),
    })
    .map_err(|_| Denial::CoordinateOverflow)
}

fn translate_clip(
    clip: UiAppearanceClip,
    portal: UiMountedPortalOverlayMechanic,
) -> Result<UiAppearanceClip, Denial> {
    let presented = super::geometry::allocation(portal.bounds())?;
    let anchor = super::geometry::allocation(portal.anchor_bounds())?;
    let x = i64::from(clip.x()) + i64::from(presented.x()) - i64::from(anchor.x());
    let y = i64::from(clip.y()) + i64::from(presented.y()) - i64::from(anchor.y());
    UiAppearanceClip::new(
        i32::try_from(x).map_err(|_| Denial::CoordinateOverflow)?,
        i32::try_from(y).map_err(|_| Denial::CoordinateOverflow)?,
        clip.width(),
        clip.height(),
    )
    .map_err(|_| Denial::EmptyAtCanonicalPrecision)
}

fn canonical_clip(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
) -> Result<Option<UiAppearanceClip>, Denial> {
    match super::geometry::allocation(bounds) {
        Ok(bounds) => Ok(Some(
            UiAppearanceClip::new(bounds.x(), bounds.y(), bounds.width(), bounds.height())
                .expect("canonical allocation has area"),
        )),
        Err(Denial::AllocationHasNoArea | Denial::EmptyAtCanonicalPrecision) => Ok(None),
        Err(denial) => Err(denial),
    }
}
