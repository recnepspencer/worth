use super::{UiMountedAppearanceClip as Clip, UiMountedAppearanceGeometryDenial as Denial};
use crate::mounting::projection::placement::{UiPortalMove, UiPortalPresentable};
use worth_ui_host_contract::{
    UiAppearanceClip, UiMountedAllocationProjection, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedPortalOverlayMechanic,
};

/// Completed host-surface occurrence geometry, rebased from the exact anchor
/// neighborhood into the Portal's presented neighborhood.
impl UiPortalPresentable for UiMountedAllocationProjection {
    type Denial = Denial;

    fn moved(self, by: &UiPortalMove) -> Result<Option<Self>, Denial> {
        Ok(Some(match self {
            Self::Known { bounds, basis } => Self::Known {
                bounds: moved_box(bounds, by)?,
                basis,
            },
            Self::PortalAnchorObservation { bounds, basis } => Self::PortalAnchorObservation {
                bounds: moved_box(bounds, by)?,
                basis,
            },
            Self::Omitted(reason) => Self::Omitted(reason),
        }))
    }
}

impl UiPortalPresentable for UiMountedCanonicalBox {
    type Denial = Denial;

    fn moved(self, by: &UiPortalMove) -> Result<Option<Self>, Denial> {
        moved_box(self, by).map(Some)
    }
}

/// Portal bounds constrain translated ancestor coverage, never the child's own
/// allocation.
impl UiPortalPresentable for Clip {
    type Denial = Denial;

    fn moved(self, by: &UiPortalMove) -> Result<Option<Self>, Denial> {
        if matches!(self, Clip::Unresolved(denial)
            if !matches!(denial, super::UiMountedAppearanceClipDenial::PortalBindingUnavailable(_)))
        {
            return Ok(Some(self));
        }
        let Some(coverage) = portal_coverage(by.portal())? else {
            return Ok(Some(Clip::Suppressed));
        };
        Ok(Some(match self {
            Clip::Ancestor(ancestor) => {
                let ancestor = translate_clip(ancestor, by)?;
                super::clip::intersect_clips(coverage, ancestor)
                    .map_or(Clip::Suppressed, Clip::Ancestor)
            }
            Clip::Suppressed => Clip::Suppressed,
            Clip::Unclipped
            | Clip::Unresolved(super::UiMountedAppearanceClipDenial::PortalBindingUnavailable(_)) => {
                Clip::Ancestor(coverage)
            }
            Clip::Unresolved(_) => self,
        }))
    }
}

/// The clip of a Portal's own surface: what the Portal covers of its host
/// surface, which no Portal step moves.
pub(in crate::mounting::projection) fn portal_surface_clip(
    portal: UiMountedPortalOverlayMechanic,
) -> Result<Clip, Denial> {
    Ok(portal_coverage(portal)?.map_or(Clip::Suppressed, Clip::Ancestor))
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
pub(super) fn portal_step(by: &UiPortalMove) -> Result<[i64; 2], Denial> {
    let presented = super::geometry::allocation(by.portal().paint_bounds())?;
    let anchor = super::geometry::allocation(by.source_anchor())?;
    Ok([
        i64::from(presented.x()) - i64::from(anchor.x()),
        i64::from(presented.y()) - i64::from(anchor.y()),
    ])
}

/// Moves one logical-subpixel coordinate by a Portal step.
pub(super) fn step_coordinate(coordinate: i32, step: i64) -> Result<i32, Denial> {
    i32::try_from(i64::from(coordinate) + step).map_err(|_| Denial::CoordinateOverflow)
}

fn moved_box(
    bounds: UiMountedCanonicalBox,
    by: &UiPortalMove,
) -> Result<UiMountedCanonicalBox, Denial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: bounds.x() + by.portal().paint_bounds().x() - by.source_anchor().x(),
        y: bounds.y() + by.portal().paint_bounds().y() - by.source_anchor().y(),
        width: bounds.width(),
        height: bounds.height(),
        coordinate_space: bounds.coordinate_space(),
    })
    .map_err(|_| Denial::CoordinateOverflow)
}

fn translate_clip(clip: UiAppearanceClip, by: &UiPortalMove) -> Result<UiAppearanceClip, Denial> {
    let [x, y] = portal_step(by)?;
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
