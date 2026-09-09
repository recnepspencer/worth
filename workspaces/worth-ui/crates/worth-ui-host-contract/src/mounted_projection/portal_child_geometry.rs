use super::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedGeometryDenial,
    UiMountedPortalOverlayMechanic,
};

pub(super) struct UiPortalChildGeometry {
    pub(super) bounds: UiMountedCanonicalBox,
    pub(super) clip: UiMountedCanonicalBox,
}

/// A Portal translates child geometry and narrows its authored clip. Empty
/// visible coverage suppresses the mechanic; malformed geometry remains a denial.
pub(super) fn project(
    bounds: UiMountedCanonicalBox,
    clip: UiMountedCanonicalBox,
    portal: UiMountedPortalOverlayMechanic,
) -> Result<Option<UiPortalChildGeometry>, UiMountedGeometryDenial> {
    let bounds = translate(bounds, portal)?;
    let clip = translate(clip, portal)?;
    let Some(clip) = intersect(clip, portal.bounds())? else {
        return Ok(None);
    };
    let Some(clip) = intersect(clip, portal.clip_bounds())? else {
        return Ok(None);
    };
    if intersect(bounds, clip)?.is_none() {
        return Ok(None);
    }
    Ok(Some(UiPortalChildGeometry { bounds, clip }))
}

fn translate(
    occurrence: UiMountedCanonicalBox,
    portal: UiMountedPortalOverlayMechanic,
) -> Result<UiMountedCanonicalBox, UiMountedGeometryDenial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: occurrence.x() + portal.bounds().x() - portal.anchor_bounds().x(),
        y: occurrence.y() + portal.bounds().y() - portal.anchor_bounds().y(),
        width: occurrence.width(),
        height: occurrence.height(),
        coordinate_space: portal.bounds().coordinate_space(),
    })
}

fn intersect(
    a: UiMountedCanonicalBox,
    b: UiMountedCanonicalBox,
) -> Result<Option<UiMountedCanonicalBox>, UiMountedGeometryDenial> {
    let x = a.x().max(b.x());
    let y = a.y().max(b.y());
    let right = (a.x() + a.width()).min(b.x() + b.width());
    let bottom = (a.y() + a.height()).min(b.y() + b.height());
    if right <= x || bottom <= y {
        return Ok(None);
    }
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width: right - x,
        height: bottom - y,
        coordinate_space: a.coordinate_space(),
    })
    .map(Some)
}
