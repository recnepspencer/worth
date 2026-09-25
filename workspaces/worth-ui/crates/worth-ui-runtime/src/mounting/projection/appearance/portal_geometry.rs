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
    source_anchor: UiMountedCanonicalBox,
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
    let bounds = translate_box(bounds, portal, source_anchor)?;
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
    source_anchor: UiMountedCanonicalBox,
) -> Result<Clip, Denial> {
    if matches!(ancestry, Clip::Unresolved(denial)
        if !matches!(denial, super::UiMountedAppearanceClipDenial::PortalBindingUnavailable(_)))
    {
        return Ok(ancestry);
    }
    let Some(coverage) = portal_coverage(portal)? else {
        return Ok(Clip::Suppressed);
    };
    match ancestry {
        Clip::Ancestor(ancestor) => {
            let ancestor = translate_clip(ancestor, portal, source_anchor)?;
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

/// What of its host surface an open Portal presents content over: its paint
/// bounds within its clip. `None` when that is nothing. Every reader of
/// Portal coverage takes it from here.
pub(in crate::mounting::projection) fn portal_coverage_box(
    portal: UiMountedPortalOverlayMechanic,
) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
    portal.paint_bounds().intersection(portal.clip_bounds())
}

/// [`portal_coverage_box`] as an appearance clip: `None` when nothing of it
/// survives canonical precision.
pub(super) fn portal_coverage(
    portal: UiMountedPortalOverlayMechanic,
) -> Result<Option<UiAppearanceClip>, Denial> {
    portal_coverage_box(portal).map_or(Ok(None), canonical_clip)
}

/// The step, in logical subpixels, from where the Portal's source anchor is
/// laid out to where the Portal presents it.
pub(super) fn portal_step(
    portal: UiMountedPortalOverlayMechanic,
    source_anchor: UiMountedCanonicalBox,
) -> Result<[i64; 2], Denial> {
    let presented = super::geometry::allocation(portal.paint_bounds())?;
    let anchor = super::geometry::allocation(source_anchor)?;
    Ok([
        i64::from(presented.x()) - i64::from(anchor.x()),
        i64::from(presented.y()) - i64::from(anchor.y()),
    ])
}

/// Moves one logical-subpixel coordinate by a Portal step.
pub(super) fn step_coordinate(coordinate: i32, step: i64) -> Result<i32, Denial> {
    i32::try_from(i64::from(coordinate) + step).map_err(|_| Denial::CoordinateOverflow)
}

pub(in crate::mounting::projection) fn translate_box(
    bounds: UiMountedCanonicalBox,
    portal: UiMountedPortalOverlayMechanic,
    source_anchor: UiMountedCanonicalBox,
) -> Result<UiMountedCanonicalBox, Denial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: bounds.x() + portal.paint_bounds().x() - source_anchor.x(),
        y: bounds.y() + portal.paint_bounds().y() - source_anchor.y(),
        width: bounds.width(),
        height: bounds.height(),
        coordinate_space: bounds.coordinate_space(),
    })
    .map_err(|_| Denial::CoordinateOverflow)
}

fn translate_clip(
    clip: UiAppearanceClip,
    portal: UiMountedPortalOverlayMechanic,
    source_anchor: UiMountedCanonicalBox,
) -> Result<UiAppearanceClip, Denial> {
    let [x, y] = portal_step(portal, source_anchor)?;
    UiAppearanceClip::new(
        step_coordinate(clip.x(), x)?,
        step_coordinate(clip.y(), y)?,
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
