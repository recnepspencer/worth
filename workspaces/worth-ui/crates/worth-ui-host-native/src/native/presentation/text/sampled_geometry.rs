//! One clipping order for retained image coverage and glyph raster replay.
use super::{clip_glyph_command, UiNativeGlyphCommand, UiNativeGlyphCommandDenial as Denial};
use crate::native::presentation::raster::UiNativeRasterBasis;
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedPresentationSampleChange};

pub(in crate::native::presentation) fn sampled_image(
    raw: [f32; 4],
    source_clip: UiMountedCanonicalBox,
    intrinsic_clip: UiMountedCanonicalBox,
    sample: Option<UiMountedPresentationSampleChange>,
    basis: UiNativeRasterBasis,
) -> Result<Option<[f32; 4]>, Denial> {
    let surface = [0.0, 0.0, basis.extent()[0] as f32, basis.extent()[1] as f32];
    if let Some(clip) = sample.and_then(|sample| sample.clip()) {
        let Some(intrinsic) = intersect(raw, physical_clip(intrinsic_clip, basis)) else {
            return Ok(None);
        };
        let transformed = transform(intrinsic, sample, basis)?;
        return Ok(intersect(transformed, physical_clip(clip, basis))
            .and_then(|rect| intersect(rect, surface)));
    }
    // Ordinary Motion preserves its established clip-then-transform contract.
    intersect(raw, physical_clip(source_clip, basis))
        .and_then(|rect| intersect(rect, surface))
        .map(|rect| transform(rect, sample, basis))
        .transpose()
}

pub(in crate::native::presentation) fn sampled_glyph(
    mut glyph: UiNativeGlyphCommand,
    sample: Option<UiMountedPresentationSampleChange>,
    basis: UiNativeRasterBasis,
) -> Result<Option<UiNativeGlyphCommand>, Denial> {
    let Some(target) = sampled_image(
        glyph.target,
        glyph.run.clip_bounds(),
        glyph.run.intrinsic_clip_bounds(),
        sample,
        basis,
    )?
    else {
        return Ok(None);
    };
    glyph.target = transform(glyph.target, sample, basis)?;
    if let Some(sample) = sample {
        glyph.opacity = sample.opacity().factor();
    }
    Ok(clip_glyph_command(glyph, target))
}

fn transform(
    rect: [f32; 4],
    sample: Option<UiMountedPresentationSampleChange>,
    basis: UiNativeRasterBasis,
) -> Result<[f32; 4], Denial> {
    sample
        .and_then(|sample| sample.transform())
        .map_or(Ok(rect), |transform| {
            crate::native::presentation::sample::transform_physical_box(rect, transform, basis)
                .map_err(|_| Denial::GeometryOverflow)
        })
}

fn physical_clip(clip: UiMountedCanonicalBox, basis: UiNativeRasterBasis) -> [f32; 4] {
    let scale = basis.scale_factor();
    [
        clip.x() * scale,
        clip.y() * scale,
        clip.width() * scale,
        clip.height() * scale,
    ]
}

fn intersect(a: [f32; 4], b: [f32; 4]) -> Option<[f32; 4]> {
    let x = a[0].max(b[0]);
    let y = a[1].max(b[1]);
    let right = (a[0] + a[2]).min(b[0] + b[2]);
    let bottom = (a[1] + a[3]).min(b[1] + b[3]);
    (right > x && bottom > y).then_some([x, y, right - x, bottom - y])
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::*;

    #[test]
    fn scroll_reveal_never_discards_the_row_owned_clip() {
        let bounds = |x, width| {
            UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x,
                y: 0.0,
                width,
                height: 30.0,
                coordinate_space: UiMountedCoordinateSpace::Viewport,
            })
            .unwrap()
        };
        let source = bounds(0.0, 30.0);
        let target = bounds(10.0, 30.0);
        let identity = UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
            UiMountedInstanceIdentity::mint_unbound().unwrap(),
            0,
            None,
        );
        let sample = UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
            identity,
            UiMountedPresentationTransform::from_runtime_sampling(source, target).unwrap(),
            UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            bounds(0.0, 100.0),
        )
        .unwrap();
        // Raw glyph extends beyond the row's width. Movement reveals only its
        // intrinsic left half, even though the successor viewport is wider.
        assert_eq!(
            sampled_image(
                [8.0, 2.0, 4.0, 4.0],
                bounds(0.0, 0.0),
                bounds(0.0, 10.0),
                Some(sample),
                UiNativeRasterBasis::new([100, 40], 1.0)
            )
            .unwrap(),
            Some([18.0, 2.0, 2.0, 4.0])
        );
    }
}
