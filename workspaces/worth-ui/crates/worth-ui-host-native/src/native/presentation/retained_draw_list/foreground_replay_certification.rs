//! Observe adopted paint in the ordinary ordered raster plan; no GPU acceptance.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::{
    appearance::text_foreground::UiNativeFinalizedTextForeground, text::glyph_vertices,
    UiNativeRasterOperation,
};
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::*;

#[derive(Debug, PartialEq)]
pub enum UiNativeTextReplayOperation {
    Clear {
        bounds: [f32; 4],
    },
    Solid {
        bounds: [f32; 4],
        color: [u8; 4],
    },
    AnalyticSurface {
        bounds: [f32; 4],
    },
    Glyph {
        run: UiGlyphRunView,
        bounds: [f32; 4],
        vertex_color: [f32; 4],
    },
}

impl UiNativeRetainedDrawList {
    pub(crate) fn foreground_reconstruction_for_certification(
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        foreground: &UiNativeFinalizedTextForeground,
        atlas: &UiNativeTextAtlas,
    ) -> Result<Box<[UiNativeTextReplayOperation]>, Denial> {
        let mut retained =
            Self::from_text_view_for_certification(commands, order, view, extent, atlas)?;
        let basis = retained
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?
            .basis;
        retained.initialize_text_coverage(vec![foreground.clone()], atlas)?;
        let plan = super::super::reconstruction::build_plan(basis, atlas, &mut retained)
            .map_err(|_| Denial::CommandMismatch)?;
        Ok(observe_plan(&plan, extent))
    }

    pub(crate) fn foreground_replay_for_certification(
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        foreground: &UiNativeFinalizedTextForeground,
        atlas: &UiNativeTextAtlas,
    ) -> Result<Box<[UiNativeTextReplayOperation]>, Denial> {
        let mut retained =
            Self::from_text_view_for_certification(commands, order, view, extent, atlas)?;
        let basis = retained
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?
            .basis;
        retained.initialize_text_coverage(vec![foreground.clone()], atlas)?;
        let appearance = &mut retained
            .staged_appearance
            .as_mut()
            .ok_or(Denial::CommandMismatch)?
            .1;
        // A replacement with the same image witness still exercises exact
        // old/new damage and the existing reversible appearance transaction.
        appearance
            .stage_text_replace(foreground.clone(), atlas)
            .map_err(|_| Denial::CommandMismatch)?;
        let regions = appearance
            .prepare_replay()
            .map_err(|_| Denial::CommandMismatch)?;
        let mut replay = retained.replay_plan(&[], 0, 0)?;
        replay.staged_appearance_regions = regions;
        let plan =
            super::super::retained_raster::build_plan(basis, &mut retained, replay, 0, atlas)
                .map_err(|_| Denial::CommandMismatch)?;
        Ok(observe_plan(&plan, extent))
    }
}

pub(super) fn observe_plan(
    plan: &crate::native::presentation::UiNativePresentationPortPlan,
    extent: [u32; 2],
) -> Box<[UiNativeTextReplayOperation]> {
    let operations = plan
        .operations
        .iter()
        .map(|operation| match operation {
            UiNativeRasterOperation::Clear(rect) => UiNativeTextReplayOperation::Clear {
                bounds: rect.physical_bounds(),
            },
            UiNativeRasterOperation::FilledRect { rect, source_rgba8 } => {
                UiNativeTextReplayOperation::Solid {
                    bounds: rect.physical_bounds(),
                    color: *source_rgba8,
                }
            }
            UiNativeRasterOperation::Surface(surface) => {
                UiNativeTextReplayOperation::AnalyticSurface {
                    bounds: surface.rect().physical_bounds(),
                }
            }
            UiNativeRasterOperation::Glyph(glyph) => UiNativeTextReplayOperation::Glyph {
                run: glyph.run,
                bounds: glyph.target,
                vertex_color: glyph_vertices(*glyph, extent)[0].color,
            },
        })
        .collect::<Vec<_>>();
    if plan.clear_retained_target {
        std::iter::once(UiNativeTextReplayOperation::Clear {
            bounds: [0.0, 0.0, extent[0] as f32, extent[1] as f32],
        })
        .chain(operations)
        .collect()
    } else {
        operations.into_boxed_slice()
    }
}
