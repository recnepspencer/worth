use super::CoverageWorld;
use worth_ui_host_contract::*;

/// Where the host draws `points` from, read independently of it: the nearest
/// device pixel at 1.25 device pixels per point, in millipoints.
fn presented_millipoints(points: f32) -> i64 {
    let pixels = (f64::from(points) * 1.25).round();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "fixture origins are a few hundred points, far inside i64 millipoints"
    )]
    let millipoints = (pixels / 1.25 * 1_000.0).round() as i64;
    millipoints
}

// Independent oracle uses the text owner's raster bearing, extent and positioned outline.
pub(super) fn images(world: &CoverageWorld) -> Vec<[f32; 4]> {
    let text = &world.fragment.text_candidates()[0];
    let origin = presented_millipoints(text.origin_x());
    let demand = worth_ui_text::derive_glyph_raster_demand(
        &world.layout,
        worth_ui_text::UiGlyphRasterDemandRequest {
            paint_spans: text.foregrounds(),
            selection: worth_ui_text::UiGlyphRasterDemandSelection::CompleteLayout,
            scale: worth_ui_text::UiGlyphRasterScale::new(1_250, text.qualified_layout_scale())
                .unwrap(),
            placement: worth_ui_text::UiGlyphRasterPlacement::from_millipoints([origin, 0]),
            lane: UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap();
    let raster = worth_ui_text::rasterize_alpha_outline(&world.layout, &demand).unwrap();
    demand
        .records()
        .iter()
        .enumerate()
        .filter_map(|(ordinal, record)| {
            if ![0, 2, 4].contains(&record.attribution().original_range().start()) {
                return None;
            }
            let positioned = demand
                .positioned_glyph_for_record(&world.layout, ordinal)
                .unwrap();
            let image = raster
                .batch()
                .records()
                .iter()
                .find(|image| image.key() == record.key())
                .unwrap();
            Some([
                super::device_pixels(
                    ((origin + positioned.origin_x_millipoints()) as f64 * 1.25 / 1_000.0).floor(),
                ) + image.bearing().x_over_64() as f32 / 64.0,
                super::device_pixels(
                    (positioned.origin_y_millipoints() as f64 * 1.25 / 1_000.0).floor(),
                ) - image.bearing().y_over_64() as f32 / 64.0,
                image.extent().width() as f32,
                image.extent().height() as f32,
            ])
        })
        .collect()
}

pub(super) fn clear([x, y, w, h]: [f32; 4]) -> [f32; 4] {
    [
        x.floor(),
        y.floor(),
        (x + w).ceil() - x.floor(),
        (y + h).ceil() - y.floor(),
    ]
}

pub(super) fn intersection(a: [f32; 4], b: [f32; 4]) -> Option<[f32; 4]> {
    let x = a[0].max(b[0]);
    let y = a[1].max(b[1]);
    let right = (a[0] + a[2]).min(b[0] + b[2]);
    let bottom = (a[1] + a[3]).min(b[1] + b[3]);
    (x < right && y < bottom).then_some([x, y, right - x, bottom - y])
}
