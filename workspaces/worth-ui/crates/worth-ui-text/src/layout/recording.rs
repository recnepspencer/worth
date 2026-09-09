use worth_ui_host_contract::{
    UiPositionedTextGlyphInput, UiPositionedTextGlyphRecord, UiQualifiedTextGlyphRecord,
    UiQualifiedTextLineInput, UiQualifiedTextLineRecord, UiQualifiedTextRunRecord,
    UiQualifiedTextVisualRunInput, UiQualifiedTextVisualRunRecord, UiTextOriginalRange, UiTextRect,
};

use super::{
    interaction::{PositionedCluster, PositionedLineAnchor},
    line_fitting::LinePlan,
    units::{LayoutUnit, UnitKind},
    visual_order::{PositionedUnit, VisualLine},
};
use crate::UiShapedTextParagraph;

pub(super) struct Output<'output> {
    pub(super) lines: &'output mut Vec<UiQualifiedTextLineRecord>,
    pub(super) visual_runs: &'output mut Vec<UiQualifiedTextVisualRunRecord>,
    pub(super) glyphs: &'output mut Vec<UiPositionedTextGlyphRecord>,
    pub(super) positioned_units: &'output mut Vec<PositionedCluster>,
    pub(super) line_anchors: &'output mut Vec<PositionedLineAnchor>,
}

struct LineAnchorInput<'layout> {
    shaped: &'layout UiShapedTextParagraph,
    units: &'layout [LayoutUnit],
    plan: &'layout LinePlan,
    visual: &'layout VisualLine,
    line_index: usize,
    top_millipoints: i64,
    line_height_millipoints: i64,
}

pub(super) fn line(
    shaped: &UiShapedTextParagraph,
    units: &[LayoutUnit],
    plan: &LinePlan,
    visual: &VisualLine,
    logical_runs: &[UiQualifiedTextRunRecord],
    logical_glyphs: &[UiQualifiedTextGlyphRecord],
    line_index: usize,
    output: &mut Output<'_>,
) {
    let line_height = i64::from(shaped.constraints().line_height_millipoints());
    let top = i64::try_from(line_index).expect("line cap fits i64") * line_height;
    let baseline = baseline(shaped, logical_runs, units, plan, top, line_height);
    let positioned_glyph_start = output.glyphs.len();
    let visual_run_start = u32::try_from(output.visual_runs.len()).expect("run cap fits u32");
    if visual.groups.is_empty() {
        record_line_anchor(
            LineAnchorInput {
                shaped,
                units,
                plan,
                visual,
                line_index,
                top_millipoints: top,
                line_height_millipoints: line_height,
            },
            output,
        );
    }
    for group in &visual.groups {
        let group_units = &visual.units[group.unit_start..group.unit_end];
        let source_start = group_units
            .iter()
            .map(|unit| units[unit.unit_index].original_range.start())
            .min()
            .expect("nonempty group");
        let source_end = group_units
            .iter()
            .map(|unit| units[unit.unit_index].original_range.end())
            .max()
            .expect("nonempty group");
        let fallback_run = u32::try_from(logical_runs.len()).expect("run cap fits u32");
        let logical_start = if group.logical_run_index == u32::MAX {
            fallback_run
        } else {
            group.logical_run_index
        };
        let visual_run_index = u32::try_from(output.visual_runs.len()).expect("run cap fits u32");
        let bounds = UiTextRect::from_text_mechanics(
            group.x_start_millipoints,
            top,
            group.x_end_millipoints,
            top + line_height,
        )
        .expect("ordered run bounds");
        output
            .visual_runs
            .push(UiQualifiedTextVisualRunRecord::from_text_mechanics(
                UiQualifiedTextVisualRunInput {
                    original_range: UiTextOriginalRange::from_text_mechanics(
                        source_start,
                        source_end,
                    )
                    .expect("ordered group range"),
                    line_index: u32::try_from(line_index).expect("line cap fits u32"),
                    logical_run_start: logical_start,
                    logical_run_end: logical_start
                        .saturating_add(u32::from(logical_start != fallback_run)),
                    bidi_level: group.bidi_level,
                    bounds,
                },
            ));
        for positioned in group_units {
            let unit = &units[positioned.unit_index];
            position_glyphs(
                shaped,
                logical_runs,
                logical_glyphs,
                unit,
                positioned,
                line_index,
                visual_run_index,
                baseline,
                output.glyphs,
            );
            output.positioned_units.push(PositionedCluster {
                original_range: unit.original_range,
                line_index: u32::try_from(line_index).expect("line cap fits u32"),
                visual_run_index,
                bidi_level: unit.bidi_level,
                bounds: UiTextRect::from_text_mechanics(
                    positioned.x_millipoints,
                    top,
                    positioned.x_millipoints + positioned.width_millipoints,
                    top + line_height,
                )
                .expect("ordered cluster bounds"),
            });
        }
    }
    let visual_run_end = u32::try_from(output.visual_runs.len()).expect("run cap fits u32");
    output
        .lines
        .push(UiQualifiedTextLineRecord::from_text_mechanics(
            UiQualifiedTextLineInput {
                original_range: line_range(shaped, units, plan),
                visual_run_start,
                visual_run_end,
                logical_bounds: UiTextRect::from_text_mechanics(
                    visual.x_offset_millipoints,
                    top,
                    visual.x_offset_millipoints + plan.width_millipoints,
                    top + line_height,
                )
                .expect("ordered line bounds"),
                ink_bounds: aggregate_ink_bounds(
                    &output.glyphs[positioned_glyph_start..],
                    visual.x_offset_millipoints,
                    baseline,
                ),
                baseline_millipoints: baseline,
                hard_break: plan.hard_break,
                overflowed: plan.overflowed,
            },
        ));
}

fn record_line_anchor(input: LineAnchorInput<'_>, output: &mut Output<'_>) {
    let boundary = line_anchor_boundary(input.shaped, input.units, input.plan);
    let original_range = UiTextOriginalRange::from_text_mechanics(boundary, boundary)
        .expect("line anchor is an ordered empty range");
    let line_index = u32::try_from(input.line_index).expect("line cap fits u32");
    let visual_run_index = u32::try_from(output.visual_runs.len()).expect("run cap fits u32");
    let bounds = UiTextRect::from_text_mechanics(
        input.visual.x_offset_millipoints,
        input.top_millipoints,
        input.visual.x_offset_millipoints,
        input.top_millipoints + input.line_height_millipoints,
    )
    .expect("line anchor bounds are ordered");
    output
        .visual_runs
        .push(UiQualifiedTextVisualRunRecord::from_text_mechanics(
            UiQualifiedTextVisualRunInput {
                original_range,
                line_index,
                logical_run_start: 0,
                logical_run_end: 0,
                bidi_level: input.visual.base_level,
                bounds,
            },
        ));
    output.line_anchors.push(PositionedLineAnchor {
        original_boundary: boundary,
        line_index,
        visual_run_index,
        bounds,
        hit_bounds: UiTextRect::from_text_mechanics(
            0,
            input.top_millipoints,
            i64::from(input.shaped.constraints().width_millipoints()),
            input.top_millipoints + input.line_height_millipoints,
        )
        .expect("empty line hit bounds are ordered"),
    });
}

fn line_anchor_boundary(
    shaped: &UiShapedTextParagraph,
    units: &[LayoutUnit],
    plan: &LinePlan,
) -> u32 {
    units
        .get(plan.unit_start)
        .filter(|_| plan.unit_start < plan.unit_end)
        .map_or_else(
            || u32::try_from(shaped.source().len()).expect("admitted source fits u32"),
            |unit| unit.original_range.start(),
        )
}

fn position_glyphs(
    shaped: &UiShapedTextParagraph,
    logical_runs: &[UiQualifiedTextRunRecord],
    logical_glyphs: &[UiQualifiedTextGlyphRecord],
    unit: &LayoutUnit,
    positioned: &PositionedUnit,
    line_index: usize,
    visual_run_index: u32,
    baseline: i64,
    output: &mut Vec<UiPositionedTextGlyphRecord>,
) {
    if unit.kind != UnitKind::Glyphs {
        return;
    }
    let run = logical_runs[unit.logical_run_index as usize];
    let style = shaped.styles()[unit.style_index as usize].style();
    let mut x = positioned.x_millipoints;
    for glyph_index in unit.glyph_range.clone() {
        let glyph = logical_glyphs[glyph_index as usize];
        let advance = scale_font_units(
            glyph.x_advance_font_units(),
            style.font_size_millipoints(),
            run.units_per_em(),
        );
        let x_offset = scale_font_units(
            glyph.x_offset_font_units(),
            style.font_size_millipoints(),
            run.units_per_em(),
        );
        let y_offset = scale_font_units(
            glyph.y_offset_font_units(),
            style.font_size_millipoints(),
            run.units_per_em(),
        );
        let origin_x = x + x_offset;
        let origin_y = baseline - y_offset;
        output.push(UiPositionedTextGlyphRecord::from_text_mechanics(
            UiPositionedTextGlyphInput {
                source_glyph_index: glyph_index,
                line_index: u32::try_from(line_index).expect("line cap fits u32"),
                visual_run_index,
                origin_x_millipoints: origin_x,
                origin_y_millipoints: origin_y,
                advance_x_millipoints: advance,
                ink_bounds: positioned_ink_bounds(glyph, run, style, origin_x, origin_y),
            },
        ));
        x += advance.abs();
    }
}

fn positioned_ink_bounds(
    glyph: UiQualifiedTextGlyphRecord,
    run: UiQualifiedTextRunRecord,
    style: &crate::UiTextStyle,
    origin_x: i64,
    origin_y: i64,
) -> UiTextRect {
    let bounds = glyph.ink_bounds_font_units();
    let scale = |value| scale_font_units(value, style.font_size_millipoints(), run.units_per_em());
    UiTextRect::from_text_mechanics(
        origin_x + scale(bounds.x_min()),
        origin_y - scale(bounds.y_max()),
        origin_x + scale(bounds.x_max()),
        origin_y - scale(bounds.y_min()),
    )
    .expect("scaled font-derived glyph bounds are ordered")
}

fn aggregate_ink_bounds(
    glyphs: &[UiPositionedTextGlyphRecord],
    empty_x: i64,
    empty_y: i64,
) -> UiTextRect {
    let Some(first) = glyphs.first() else {
        return UiTextRect::from_text_mechanics(empty_x, empty_y, empty_x, empty_y)
            .expect("empty ink bounds are ordered");
    };
    glyphs
        .iter()
        .skip(1)
        .fold(first.ink_bounds(), |bounds, glyph| {
            let ink = glyph.ink_bounds();
            UiTextRect::from_text_mechanics(
                bounds.left_millipoints().min(ink.left_millipoints()),
                bounds.top_millipoints().min(ink.top_millipoints()),
                bounds.right_millipoints().max(ink.right_millipoints()),
                bounds.bottom_millipoints().max(ink.bottom_millipoints()),
            )
            .expect("ink union is ordered")
        })
}

fn baseline(
    shaped: &UiShapedTextParagraph,
    logical_runs: &[UiQualifiedTextRunRecord],
    units: &[LayoutUnit],
    plan: &LinePlan,
    top: i64,
    height: i64,
) -> i64 {
    let mut ascent = 0i64;
    let mut descent = 0i64;
    for unit in &units[plan.unit_start..plan.unit_end] {
        if unit.logical_run_index == u32::MAX {
            continue;
        }
        let run = logical_runs[unit.logical_run_index as usize];
        let size = shaped.styles()[unit.style_index as usize]
            .style()
            .font_size_millipoints();
        ascent = ascent.max(scale_font_units(
            i32::from(run.ascender_font_units()),
            size,
            run.units_per_em(),
        ));
        descent = descent.max(-scale_font_units(
            i32::from(run.descender_font_units()),
            size,
            run.units_per_em(),
        ));
    }
    top + ascent + (height - ascent - descent) / 2
}

fn line_range(
    shaped: &UiShapedTextParagraph,
    units: &[LayoutUnit],
    plan: &LinePlan,
) -> UiTextOriginalRange {
    if plan.unit_start == plan.unit_end {
        let end = u32::try_from(shaped.source().len()).expect("admitted source fits u32");
        return UiTextOriginalRange::from_text_mechanics(end, end).expect("empty end range");
    }
    UiTextOriginalRange::from_text_mechanics(
        units[plan.unit_start].original_range.start(),
        units[plan.unit_end - 1].original_range.end(),
    )
    .expect("ordered line range")
}

fn scale_font_units(value: i32, size: u32, units_per_em: u16) -> i64 {
    i64::from(value) * i64::from(size) / i64::from(units_per_em)
}
