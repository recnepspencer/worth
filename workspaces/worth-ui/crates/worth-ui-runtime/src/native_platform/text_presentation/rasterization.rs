//! Runtime-owned join from native-admitted misses to text-owned raster bytes.

use worth_ui_host_contract::{
    UiGlyphRasterBatchSink, UiGlyphRasterCallbackDenial, UiGlyphRasterMissRasterizer,
    UiGlyphRasterMissSelectionView,
};

use super::UiNativeTextPresentationPrepared;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiNativeTextRasterWorkReport {
    rasterized_glyphs: u32,
    rasterized_texels: u64,
    produced_bytes: u64,
}

impl UiNativeTextRasterWorkReport {
    #[cfg(test)]
    pub(crate) const fn not_admitted() -> Self {
        Self {
            rasterized_glyphs: 0,
            rasterized_texels: 0,
            produced_bytes: 0,
        }
    }

    pub(crate) const fn rasterized_glyphs(self) -> u32 {
        self.rasterized_glyphs
    }

    pub(crate) const fn rasterized_texels(self) -> u64 {
        self.rasterized_texels
    }

    pub(crate) const fn produced_bytes(self) -> u64 {
        self.produced_bytes
    }

    fn add(&mut self, cost: worth_ui_text::UiGlyphRasterCost) {
        for lane in [cost.ordinary(), cost.reconstructive()] {
            self.rasterized_glyphs = self
                .rasterized_glyphs
                .saturating_add(lane.rasterized_glyphs());
            self.rasterized_texels = self
                .rasterized_texels
                .saturating_add(lane.rasterized_texels());
            self.produced_bytes = self.produced_bytes.saturating_add(lane.produced_bytes());
        }
    }
}

struct UiNativeTextRasterSource<'layout> {
    command: worth_ui_host_contract::UiMountedPaintCommandIdentity,
    glyph_runs: &'layout [worth_ui_host_contract::UiGlyphRunView],
    layout: &'layout worth_ui_text::UiQualifiedTextLayout,
    demand: &'layout worth_ui_text::UiGlyphRasterDemandBatch,
}

pub(crate) struct UiNativeTextMissRasterizer<'layout, 'cache> {
    sources: Box<[UiNativeTextRasterSource<'layout>]>,
    report: UiNativeTextRasterWorkReport,
    cache: &'cache mut worth_ui_text::UiGlyphRasterCache,
}

impl<'layout, 'cache> UiNativeTextMissRasterizer<'layout, 'cache> {
    pub(crate) fn for_prepared(
        prepared: &'layout UiNativeTextPresentationPrepared,
        resolve: impl Fn(
            worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
        ) -> Option<&'layout worth_ui_text::UiQualifiedTextLayout>,
        cache: &'cache mut worth_ui_text::UiGlyphRasterCache,
    ) -> Option<Self> {
        if prepared.demand_batches().len() != prepared.pin_commands().len() {
            return None;
        }
        let mut remaining_runs = prepared.glyph_runs();
        let sources = prepared
            .demand_batches()
            .iter()
            .zip(prepared.pin_commands())
            .map(|(demand, &command)| {
                let count = demand.records().len();
                let glyph_runs = remaining_runs.get(..count)?;
                remaining_runs = remaining_runs.get(count..)?;
                if glyph_runs.iter().any(|run| run.mechanic() != command) {
                    return None;
                }
                Some(UiNativeTextRasterSource {
                    command,
                    glyph_runs,
                    layout: resolve(demand.layout_identity())?,
                    demand,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        if !remaining_runs.is_empty() {
            return None;
        }
        Some(Self {
            sources: sources.into_boxed_slice(),
            report: UiNativeTextRasterWorkReport::default(),
            cache,
        })
    }

    pub(crate) fn validate_complete_demand(
        &self,
        command: worth_ui_host_contract::UiMountedPaintCommandIdentity,
        demand: worth_ui_host_contract::UiGlyphRasterDemandBatchView<'_>,
        glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    ) -> Result<
        worth_ui_host_contract::UiMountedTextDemandValidationCost,
        worth_ui_host_contract::UiMountedTextDemandValidationDenial,
    > {
        use worth_ui_host_contract::{
            UiGlyphRasterDemandScope, UiMountedTextDemandValidationDenial as Denial,
        };
        let (index, source) = self
            .sources
            .iter()
            .enumerate()
            .find(|(_, source)| source.command == command)
            .ok_or(Denial::MissingCommand)?;
        if source.demand.scope() != UiGlyphRasterDemandScope::CompleteLayout {
            return Err(Denial::IncompleteDemand);
        }
        if source.demand.as_view() != demand {
            return Err(Denial::DemandMismatch);
        }
        if source.glyph_runs != glyph_runs {
            return Err(Denial::GlyphRunMismatch);
        }
        Ok(worth_ui_host_contract::UiMountedTextDemandValidationCost {
            demand_sources_checked: index + 1,
            demand_records_checked: source.demand.records().len(),
            glyph_runs_checked: source.glyph_runs.len(),
        })
    }

    pub(crate) const fn report(&self) -> UiNativeTextRasterWorkReport {
        self.report
    }

    pub(crate) fn cache_len(&self) -> usize {
        self.cache.len()
    }

    pub(crate) fn reconstruct_cache(
        &mut self,
    ) -> Result<(), worth_ui_text::UiGlyphRasterizationDenial> {
        for source in &self.sources {
            let keys = source
                .demand
                .records()
                .iter()
                .map(|record| record.key())
                .collect::<Vec<_>>();
            let alpha = worth_ui_text::rasterize_alpha_outline_selection_cached(
                source.layout,
                source.demand,
                &keys,
                self.cache,
            )?;
            let color = worth_ui_text::rasterize_intrinsic_color_selection_cached(
                source.layout,
                source.demand,
                &keys,
                self.cache,
            )?;
            self.report.add(alpha.cost());
            self.report.add(color.cost());
        }
        Ok(())
    }
}

impl UiGlyphRasterMissRasterizer for UiNativeTextMissRasterizer<'_, '_> {
    fn rasterize(
        &mut self,
        misses: UiGlyphRasterMissSelectionView<'_>,
        sink: &mut dyn UiGlyphRasterBatchSink,
    ) -> Result<(), UiGlyphRasterCallbackDenial> {
        let (layout, demand) = self
            .sources
            .iter()
            .find(|source| {
                source.demand.identity() == misses.demand_identity()
                    && source.demand.layout_identity() == misses.layout_identity()
                    && source.demand.lane() == misses.lane()
            })
            .map(|source| (source.layout, source.demand))
            .ok_or(UiGlyphRasterCallbackDenial::DemandMismatch)?;
        let keys = misses
            .records()
            .iter()
            .map(|record| record.key())
            .collect::<Vec<_>>();
        let alpha = worth_ui_text::rasterize_alpha_outline_selection_cached(
            layout, demand, &keys, self.cache,
        )
        .map_err(|_| UiGlyphRasterCallbackDenial::RasterizationDenied)?;
        let color = worth_ui_text::rasterize_intrinsic_color_selection_cached(
            layout, demand, &keys, self.cache,
        )
        .map_err(|_| UiGlyphRasterCallbackDenial::RasterizationDenied)?;
        if !alpha.batch().records().is_empty() {
            alpha
                .batch()
                .with_view(|batch| sink.submit_alpha(batch))
                .map_err(UiGlyphRasterCallbackDenial::BatchRejected)?;
        }
        if !color.batch().records().is_empty() {
            color
                .batch()
                .with_view(|batch| sink.submit_color(batch))
                .map_err(UiGlyphRasterCallbackDenial::BatchRejected)?;
        }
        self.report.add(alpha.cost());
        self.report.add(color.cost());
        Ok(())
    }
}
