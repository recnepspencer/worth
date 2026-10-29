//! Portable runtime transaction joining mounted text meaning to the native
//! host's private atlas effect facade.

use worth_ui_host_contract::UiGlyphRasterPinTransitionView;

use super::{UiNativeTextMissRasterizer, UiNativeTextPresentationPrepared};

pub(crate) struct UiNativeTextAtlasTransaction<'layout, 'cache> {
    demand_views: Box<[worth_ui_host_contract::UiGlyphRasterDemandBatchView<'layout>]>,
    glyph_runs: &'layout [worth_ui_host_contract::UiGlyphRunView],
    rasterizer: UiNativeTextMissRasterizer<'layout, 'cache>,
}

impl<'layout, 'cache> UiNativeTextAtlasTransaction<'layout, 'cache> {
    pub(crate) fn prepare(
        prepared: &'layout UiNativeTextPresentationPrepared,
        resolve: &super::MountedLayoutResolve<'layout>,
        cache: &'cache mut worth_ui_text::UiGlyphRasterCache,
    ) -> Option<Self> {
        let demand_views = prepared
            .demand_batches()
            .iter()
            .map(worth_ui_text::UiGlyphRasterDemandBatch::as_view)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let rasterizer = UiNativeTextMissRasterizer::for_prepared(prepared, resolve, cache)?;
        Some(Self {
            demand_views,
            glyph_runs: prepared.glyph_runs(),
            rasterizer,
        })
    }

    pub(crate) fn reconstruct_cache(&mut self) -> bool {
        self.rasterizer.reconstruct_cache().is_ok()
    }

    pub(crate) fn cache_len(&self) -> usize {
        self.rasterizer.cache_len()
    }

    pub(crate) fn with_mounted_work<Output>(
        &mut self,
        pins: UiGlyphRasterPinTransitionView<'_>,
        binding_pins: &[worth_ui_host_contract::UiGlyphRasterPinRequest],
        operation: impl FnOnce(&worth_ui_host_contract::UiMountedTextRasterWork<'_>) -> Output,
    ) -> (Output, super::rasterization::UiNativeTextRasterWorkReport) {
        let mut operation = Some(operation);
        let mut output = None;
        let report = self.project_mounted_work(pins, binding_pins, &mut |work| {
            output = Some(operation.take().expect("mounted text work is visited once")(work));
        });
        (output.expect("mounted text work is visited once"), report)
    }

    #[inline(never)]
    fn project_mounted_work(
        &mut self,
        pins: UiGlyphRasterPinTransitionView<'_>,
        binding_pins: &[worth_ui_host_contract::UiGlyphRasterPinRequest],
        operation: &mut dyn FnMut(&worth_ui_host_contract::UiMountedTextRasterWork<'_>),
    ) -> super::rasterization::UiNativeTextRasterWorkReport {
        struct Callback<'rasterizer, 'layout, 'cache> {
            rasterizer:
                std::cell::RefCell<&'rasterizer mut UiNativeTextMissRasterizer<'layout, 'cache>>,
        }
        impl worth_ui_host_contract::UiMountedTextRasterCallback for Callback<'_, '_, '_> {
            fn validate_complete_demand(
                &self,
                command: worth_ui_host_contract::UiMountedPaintCommandIdentity,
                demand: worth_ui_host_contract::UiGlyphRasterDemandBatchView<'_>,
                glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
            ) -> Result<
                worth_ui_host_contract::UiMountedTextDemandValidationCost,
                worth_ui_host_contract::UiMountedTextDemandValidationDenial,
            > {
                self.rasterizer
                    .borrow()
                    .validate_complete_demand(command, demand, glyph_runs)
            }

            fn rasterize(
                &self,
                misses: worth_ui_host_contract::UiGlyphRasterMissSelectionView<'_>,
                sink: &mut dyn worth_ui_host_contract::UiGlyphRasterBatchSink,
            ) -> Result<(), worth_ui_host_contract::UiGlyphRasterCallbackDenial> {
                use worth_ui_host_contract::UiGlyphRasterMissRasterizer;
                self.rasterizer.borrow_mut().rasterize(misses, sink)
            }
        }
        let callback = Callback {
            rasterizer: std::cell::RefCell::new(&mut self.rasterizer),
        };
        let work = worth_ui_host_contract::UiMountedTextRasterWork::from_text_mechanics(
            &self.demand_views,
            self.glyph_runs,
            pins,
            binding_pins,
            &callback,
        );
        operation(&work);
        self.rasterizer.report()
    }
}
