use super::CoverageWorld;

impl CoverageWorld {
    /// Independent rectangle oracle: text-owner raster bytes/bearing plus authored
    /// tab stops and qualified baseline. Never calls native glyph/clip planning.
    pub(in crate::native_platform::text_presentation) fn expected_images(
        &self,
        x: i64,
        clip: [i64; 4],
    ) -> Vec<[i64; 4]> {
        let candidate = &self.fragment.text_candidates()[0];
        let demand = worth_ui_text::derive_glyph_raster_demand(
            &self.layout,
            worth_ui_text::UiGlyphRasterDemandRequest {
                paint_spans: candidate.foregrounds(),
                selection: worth_ui_text::UiGlyphRasterDemandSelection::CompleteLayout,
                scale: worth_ui_text::UiGlyphRasterScale::new(
                    1_000,
                    candidate.qualified_layout_scale(),
                )
                .unwrap(),
                placement: worth_ui_text::UiGlyphRasterPlacement::from_mounted_logical(
                    candidate.origin_x(),
                    0.0,
                )
                .unwrap(),
                lane: worth_ui_host_contract::UiGlyphRasterLane::Ordinary,
            },
        )
        .unwrap();
        let raster = worth_ui_text::rasterize_alpha_outline(&self.layout, &demand).unwrap();
        let image = &raster.batch().records()[0];
        assert_eq!(image.bearing().x_over_64() % 64, 0);
        assert_eq!(image.bearing().y_over_64() % 64, 0);
        let baseline = self.layout.view().lines()[0].baseline_millipoints() / 1_000;
        [0, 56, 112]
            .into_iter()
            .filter_map(|tab| {
                let left = x + tab + i64::from(image.bearing().x_over_64()) / 64;
                let top = baseline - i64::from(image.bearing().y_over_64()) / 64;
                let edges = [
                    left.max(clip[0]),
                    top.max(clip[1]),
                    (left + i64::from(image.extent().width())).min(clip[2]),
                    (top + i64::from(image.extent().height())).min(clip[3]),
                ];
                (edges[0] < edges[2] && edges[1] < edges[3]).then_some(edges)
            })
            .collect()
    }
}
