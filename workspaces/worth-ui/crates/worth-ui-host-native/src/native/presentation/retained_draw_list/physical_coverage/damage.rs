//! Old and new visible text images become physical clears, without allocation fallback.
use super::super::UiNativeRetainedDrawListDenial as Denial;
use super::{UiNativeCommandImageCoverage, UiNativePhysicalCoverage};
use crate::native::presentation::raster::{
    raster_physical_bounds, RasterRect, UiNativeRasterBasis,
};
use worth_ui_host_contract::UiMountedPaintCommandIdentity;

pub(in crate::native::presentation::retained_draw_list) fn append_text_transition(
    output: &mut Vec<RasterRect>,
    identity: UiMountedPaintCommandIdentity,
    previous: Option<&UiNativeCommandImageCoverage>,
    next: Option<&UiNativeCommandImageCoverage>,
    basis: UiNativeRasterBasis,
) -> Result<(), Denial> {
    if identity.semantic_text_identity_parts().is_none() {
        return Ok(());
    }
    let old = previous.map_or(&[][..], |record| record.current.as_ref());
    let new = next.map_or(&[][..], |record| record.current.as_ref());
    // Equal geometry still needs one replay for paint or opacity changes.
    let successor = if old == new { &[][..] } else { new };
    output.reserve(old.len() + successor.len());
    for image in old.iter().chain(successor) {
        if let Some(rect) = image_clear(*image, basis.extent())? {
            output.push(rect);
        }
    }
    Ok(())
}

/// Order changes consume retained current images, including existing Motion sampling.
pub(in crate::native::presentation::retained_draw_list) fn append_text_order_damage(
    coverage: &UiNativePhysicalCoverage,
    edits: &[worth_ui_host_contract::UiMountedPaintOrderEdit],
    changed: &[UiMountedPaintCommandIdentity],
    output: &mut Vec<RasterRect>,
) -> Result<(), Denial> {
    if edits.is_empty() {
        return Ok(());
    }
    let mut covered = changed
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    for edit in edits {
        let identity = edit.identity().command();
        if edit.is_removal()
            || identity.semantic_text_identity_parts().is_none()
            || !covered.insert(identity)
        {
            continue;
        }
        let current = coverage.get(identity).ok_or(Denial::CommandMismatch)?;
        append_text_transition(
            output,
            identity,
            Some(current),
            Some(current),
            coverage.basis,
        )?;
    }
    Ok(())
}

fn image_clear(
    [x, y, width, height]: [f32; 4],
    extent: [u32; 2],
) -> Result<Option<RasterRect>, Denial> {
    let edges = [
        x.floor(),
        y.floor(),
        (x + width).ceil(),
        (y + height).ceil(),
    ];
    if edges.iter().any(|edge| !edge.is_finite()) || width <= 0.0 || height <= 0.0 {
        return Err(Denial::CommandMismatch);
    }
    let limits = [extent[0], extent[1], extent[0], extent[1]];
    let bounds =
        std::array::from_fn(|i| f64::from(edges[i]).clamp(0.0, f64::from(limits[i])) as u32);
    Ok(raster_physical_bounds(bounds, extent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_image_edges_clamp_to_viewport_without_rescaling() {
        for (image, expected) in [
            ([10.3, 4.7, 2.1, 3.5], Some([10.0, 4.0, 3.0, 5.0])),
            ([-2.2, -1.5, 4.0, 3.0], Some([0.0, 0.0, 2.0, 2.0])),
            ([98.7, 48.7, 4.0, 4.0], Some([98.0, 48.0, 2.0, 2.0])),
            ([-10.0, 0.0, 2.0, 3.0], None),
            ([100.0, 0.0, 2.0, 3.0], None),
        ] {
            assert_eq!(
                image_clear(image, [100, 50])
                    .unwrap()
                    .map(|rect| rect.physical_bounds()),
                expected
            );
        }
    }

    #[test]
    fn invalid_image_extent_or_overflow_denies_before_physical_clear() {
        for image in [
            [f32::MAX, 0.0, f32::MAX, 1.0],
            [0.0, 0.0, f32::NAN, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ] {
            assert_eq!(image_clear(image, [100, 50]), Err(Denial::CommandMismatch));
        }
    }
}
