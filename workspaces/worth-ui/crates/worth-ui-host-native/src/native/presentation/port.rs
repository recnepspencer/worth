use worth_ui_host_contract::UiHostPresentationCostReport;

use crate::native::UiNativePresentationAccess;

use super::{RasterRect, UiNativePendingExternalObligation};

pub(crate) mod orchestrator;
mod phase;
mod transaction;

pub(crate) use phase::UiNativeSurfaceAcquireFailure;

/// Contractual boundary for one native presentation transaction.
///
/// The real implementation owns wgpu acquisition, encoding, submission,
/// present handoff, and retained-source readback. Protocol tests may replace
/// only this boundary; they cannot return a framework settlement verdict.
pub(crate) trait UiNativePresentationPort {
    fn present(
        graphics: &mut UiNativePresentationAccess,
        atlas: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
        plan: UiNativePresentationPortPlan,
        defer_initial_observation: bool,
        lifecycle: &mut crate::native::lifecycle::UiNativeLifecycleOrchestrator,
    ) -> Result<UiNativePresentationPortObservation, UiNativePresentationPortFailure>;
}

pub(crate) enum UiNativePresentationPortFailure {
    Surface(UiNativeSurfaceAcquireFailure),
    ReadbackUnsettled(Box<dyn UiNativePendingExternalObligation>),
}

pub(crate) struct UiWgpuNativePresentationPort;

#[derive(Clone)]
pub(crate) enum UiNativeRasterOperation {
    Clear(RasterRect),
    FilledRect {
        rect: RasterRect,
        source_rgba8: [u8; 4],
    },
    Surface(super::appearance::UiNativeSurfaceRasterOperation),
    Glyph(super::text::UiNativeGlyphCommand),
}

pub(crate) struct UiNativePresentationPortPlan {
    pub(super) clear_retained_target: bool,
    pub(super) operations: Box<[UiNativeRasterOperation]>,
    pub(super) cost: UiHostPresentationCostReport,
}

pub(crate) struct UiNativePresentationPortObservation {
    pixels: [[u8; 4]; 2],
    cost: UiHostPresentationCostReport,
    crossing_count: u8,
}

impl UiNativePresentationPortObservation {
    pub(super) const fn from_async_readback(
        pixels: [[u8; 4]; 2],
        cost: UiHostPresentationCostReport,
    ) -> Self {
        Self {
            pixels,
            cost,
            crossing_count: 2,
        }
    }

    pub(super) fn into_parts(self) -> ([[u8; 4]; 2], UiHostPresentationCostReport, u8) {
        (self.pixels, self.cost, self.crossing_count)
    }

    pub(crate) fn into_superseded_cost(self) -> UiHostPresentationCostReport {
        self.cost
    }

    #[cfg(test)]
    pub(crate) fn test() -> Self {
        Self::from_async_readback([[0; 4]; 2], UiHostPresentationCostReport::default())
    }
}

impl UiNativePresentationPort for UiWgpuNativePresentationPort {
    fn present(
        graphics: &mut UiNativePresentationAccess,
        atlas: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
        plan: UiNativePresentationPortPlan,
        defer_initial_observation: bool,
        lifecycle: &mut crate::native::lifecycle::UiNativeLifecycleOrchestrator,
    ) -> Result<UiNativePresentationPortObservation, UiNativePresentationPortFailure> {
        transaction::present(graphics, atlas, plan, defer_initial_observation, lifecycle)
    }
}
