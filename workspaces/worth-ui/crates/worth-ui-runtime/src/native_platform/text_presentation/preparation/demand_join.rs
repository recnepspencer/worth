use worth_ui_host_contract::{
    UiGlyphRunView, UiGlyphRunViewInput, UiMountedLogicalDamage, UiMountedSemanticTextMechanic,
};
use worth_ui_text::{
    derive_glyph_raster_demand, UiGlyphRasterDemandBatch, UiGlyphRasterDemandDenial,
    UiGlyphRasterDemandRequest, UiGlyphRasterLane, UiGlyphRasterPlacement, UiGlyphRasterScale,
};

use super::{
    MountedSemanticTextCommand, UiMountedEventTimeDpiAuthority, UiNativeTextPresentationReadiness,
};

pub(super) struct PreparedDemand {
    pub(super) demands: Box<[UiGlyphRasterDemandBatch]>,
    pub(super) glyph_runs: Box<[UiGlyphRunView]>,
}

/// Joins mounted mechanics to durable layouts under the event-time DPI basis.
/// Demand preparation performs no host/native effect.
pub(super) struct MountedTextDemandJoin<'damage, 'work, Resolve> {
    pub(super) dpi: UiMountedEventTimeDpiAuthority,
    pub(super) lane: UiGlyphRasterLane,
    pub(super) damage: &'damage [UiMountedLogicalDamage],
    pub(super) resolve: Resolve,
    pub(super) _layout: std::marker::PhantomData<&'work ()>,
}

pub(super) fn prepare_demands<'work, Resolve>(
    mechanics: &[MountedSemanticTextCommand<'work>],
    join: &MountedTextDemandJoin<'_, 'work, Resolve>,
) -> Result<PreparedDemand, UiNativeTextPresentationReadiness>
where
    Resolve: Fn(
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&'work worth_ui_text::UiQualifiedTextLayout>,
{
    let layouts = mechanics
        .iter()
        .map(|(_, mechanic)| join.layout_for(mechanic))
        .collect::<Result<Vec<_>, _>>()?;
    let demands = mechanics
        .iter()
        .zip(&layouts)
        .map(|((_, mechanic), layout)| join.demand_for(layout, mechanic))
        .collect::<Result<Vec<_>, _>>()?;
    let glyph_runs = rebuild_glyph_runs(mechanics, &layouts, &demands);
    Ok(PreparedDemand {
        demands: demands.into_boxed_slice(),
        glyph_runs,
    })
}

/// Rebuild only borrowed draw attribution against an already admitted demand.
/// This is the paint-only path: it updates foreground bytes while retaining
/// the exact qualified layout, positioned records, and raster keys.
pub(super) fn rebuild_glyph_runs<'work>(
    mechanics: &[MountedSemanticTextCommand<'work>],
    layouts: &[&'work worth_ui_text::UiQualifiedTextLayout],
    demands: &[UiGlyphRasterDemandBatch],
) -> Box<[UiGlyphRunView]> {
    debug_assert_eq!(mechanics.len(), layouts.len());
    debug_assert_eq!(mechanics.len(), demands.len());
    mechanics
        .iter()
        .zip(layouts)
        .zip(demands)
        .flat_map(|(((identity, mechanic), layout), demand)| {
            demand
                .records()
                .iter()
                .enumerate()
                .map(move |(record_index, record)| {
                    let original_range = record.attribution().original_range();
                    let mut spans = mechanic.foregrounds().iter().copied().filter(|span| {
                        span.original_range().start() <= original_range.start()
                            && span.original_range().end() >= original_range.end()
                    });
                    let span = spans
                        .next()
                        .expect("text demand admitted exactly one covering paint span");
                    debug_assert!(spans.next().is_none());
                    let positioned = demand
                        .positioned_glyph_for_record(layout, record_index)
                        .expect("retained demand preserves positioned-glyph provenance");
                    let mounted_x = mounted_origin_millipoints(mechanic.origin_x());
                    let mounted_y = mounted_origin_millipoints(mechanic.origin_y());
                    UiGlyphRunView::from_text_mechanics(UiGlyphRunViewInput {
                        mechanic: *identity,
                        layout: demand.layout_identity(),
                        paint_span: span.identity(),
                        original_range,
                        foreground: span.color(),
                        raster_key: record.key(),
                        origin_x_millipoints: positioned
                            .origin_x_millipoints()
                            .checked_add(mounted_x)
                            .expect("admitted mounted glyph origin remains bounded"),
                        origin_y_millipoints: positioned
                            .origin_y_millipoints()
                            .checked_add(mounted_y)
                            .expect("admitted mounted glyph origin remains bounded"),
                        line_index: positioned.line_index(),
                        visual_run_index: positioned.visual_run_index(),
                        clip_bounds: mechanic.clip_bounds(),
                        layer_semantic_order: mechanic.layer_semantic_order(),
                    })
                })
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

impl<'work, Resolve> MountedTextDemandJoin<'_, 'work, Resolve>
where
    Resolve: Fn(
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&'work worth_ui_text::UiQualifiedTextLayout>,
{
    fn layout_for(
        &self,
        mechanic: &UiMountedSemanticTextMechanic,
    ) -> Result<&'work worth_ui_text::UiQualifiedTextLayout, UiNativeTextPresentationReadiness>
    {
        let layout = (self.resolve)(mechanic.qualified_layout_identity())
            .ok_or(UiNativeTextPresentationReadiness::LayoutMismatch)?;
        validate_mounted_layout(layout, mechanic)?;
        Ok(layout)
    }

    fn demand_for(
        &self,
        layout: &worth_ui_text::UiQualifiedTextLayout,
        mechanic: &UiMountedSemanticTextMechanic,
    ) -> Result<UiGlyphRasterDemandBatch, UiNativeTextPresentationReadiness> {
        let scale =
            UiGlyphRasterScale::new(self.dpi.dpi_milli(), mechanic.qualified_layout_scale())
                .ok_or(UiNativeTextPresentationReadiness::DemandDenied(
                    UiGlyphRasterDemandDenial::ZeroDpi,
                ))?;
        derive_glyph_raster_demand(
            layout,
            UiGlyphRasterDemandRequest {
                paint_spans: mechanic.foregrounds(),
                logical_damage: self.damage,
                scale,
                placement: UiGlyphRasterPlacement::from_mounted_logical(
                    mechanic.origin_x(),
                    mechanic.origin_y(),
                )
                .ok_or(UiNativeTextPresentationReadiness::DemandDenied(
                    UiGlyphRasterDemandDenial::OriginOverflow,
                ))?,
                lane: self.lane,
            },
        )
        .map_err(UiNativeTextPresentationReadiness::DemandDenied)
    }
}

fn mounted_origin_millipoints(value: f32) -> i64 {
    let scaled = f64::from(value) * 1_000.0;
    debug_assert!(scaled.is_finite());
    scaled.round() as i64
}

fn validate_mounted_layout(
    layout: &worth_ui_text::UiQualifiedTextLayout,
    mechanic: &UiMountedSemanticTextMechanic,
) -> Result<(), UiNativeTextPresentationReadiness> {
    if layout.identity() != mechanic.qualified_layout_identity()
        || layout.view().request_identity() != mechanic.qualified_layout_request()
        || layout.view().profile_generation() != mechanic.qualified_layout_profile()
        || layout.view().font_collection_generation() != mechanic.qualified_layout_fonts()
        || layout.view().text_scale_generation() != mechanic.qualified_layout_scale()
    {
        return Err(UiNativeTextPresentationReadiness::LayoutMismatch);
    }
    if layout.source() != mechanic.text() {
        return Err(UiNativeTextPresentationReadiness::SourceMismatch);
    }
    Ok(())
}
