/// Borrowed pure-text input attached to an ordinary mounted presentation.
///
/// This value carries no atlas, Signal, device, or settlement authority. The
/// native host may invoke the raster callback only while executing the
/// authority-checked mounted-surface operation that borrowed it.
pub struct UiMountedTextRasterWork<'work> {
    demands: &'work [crate::UiGlyphRasterDemandBatchView<'work>],
    glyph_runs: &'work [crate::UiGlyphRunView],
    pins: crate::UiGlyphRasterPinTransitionView<'work>,
    binding_pins: &'work [crate::UiGlyphRasterPinRequest],
    rasterizer: &'work dyn UiMountedTextRasterCallback,
}

/// Successful read-only correspondence costs, separate from raster and atlas work.
/// Denials do not currently carry comparison costs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedTextDemandValidationCost {
    pub demand_sources_checked: usize,
    pub demand_records_checked: usize,
    pub glyph_runs_checked: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedTextDemandValidationDenial {
    MissingCommand,
    IncompleteDemand,
    DemandMismatch,
    GlyphRunMismatch,
}

pub trait UiMountedTextRasterCallback {
    /// Authenticate exact complete demand and ordered command attribution against
    /// runtime's owned preparation. This performs no raster or cache work and
    /// grants no publication, atlas, or settlement authority.
    fn validate_complete_demand(
        &self,
        command: crate::UiMountedPaintCommandIdentity,
        demand: crate::UiGlyphRasterDemandBatchView<'_>,
        glyph_runs: &[crate::UiGlyphRunView],
    ) -> Result<UiMountedTextDemandValidationCost, UiMountedTextDemandValidationDenial>;

    fn rasterize(
        &self,
        misses: crate::UiGlyphRasterMissSelectionView<'_>,
        sink: &mut dyn crate::UiGlyphRasterBatchSink,
    ) -> Result<(), crate::UiGlyphRasterCallbackDenial>;
}

#[doc(hidden)]
impl<'work> UiMountedTextRasterWork<'work> {
    pub fn from_text_mechanics(
        demands: &'work [crate::UiGlyphRasterDemandBatchView<'work>],
        glyph_runs: &'work [crate::UiGlyphRunView],
        pins: crate::UiGlyphRasterPinTransitionView<'work>,
        binding_pins: &'work [crate::UiGlyphRasterPinRequest],
        rasterizer: &'work dyn UiMountedTextRasterCallback,
    ) -> Self {
        Self {
            demands,
            glyph_runs,
            pins,
            binding_pins,
            rasterizer,
        }
    }

    pub fn demands(&self) -> &[crate::UiGlyphRasterDemandBatchView<'work>] {
        self.demands
    }

    pub const fn glyph_runs(&self) -> &'work [crate::UiGlyphRunView] {
        self.glyph_runs
    }

    pub const fn pins(&self) -> crate::UiGlyphRasterPinTransitionView<'work> {
        self.pins
    }

    pub const fn binding_pins(&self) -> &'work [crate::UiGlyphRasterPinRequest] {
        self.binding_pins
    }

    pub fn validate_complete_demand(
        &self,
        command: crate::UiMountedPaintCommandIdentity,
        demand: crate::UiGlyphRasterDemandBatchView<'_>,
        glyph_runs: &[crate::UiGlyphRunView],
    ) -> Result<UiMountedTextDemandValidationCost, UiMountedTextDemandValidationDenial> {
        self.rasterizer
            .validate_complete_demand(command, demand, glyph_runs)
    }

    pub fn rasterize(
        &self,
        misses: crate::UiGlyphRasterMissSelectionView<'_>,
        sink: &mut dyn crate::UiGlyphRasterBatchSink,
    ) -> Result<(), crate::UiGlyphRasterCallbackDenial> {
        self.rasterizer.rasterize(misses, sink)
    }
}
