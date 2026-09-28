use harfrust::{Direction, Language};
use worth_ui_host_contract::{
    UiQualifiedTextGlyphInput, UiQualifiedTextGlyphRecord, UiQualifiedTextRunInput,
    UiQualifiedTextRunRecord, UiTextFontUnitRect, UiTextOriginalRange,
};

use super::{
    line_fitting::{measure, LinePlan},
    units::{scale_advance, LayoutUnit, UnitKind},
};
use crate::{UiGlobalFontCollection, UiShapedTextParagraph, UiTextLayoutDenial};

pub(super) fn apply(
    shaped: &UiShapedTextParagraph,
    fonts: &UiGlobalFontCollection,
    units: &mut Vec<LayoutUnit>,
    plans: &mut [LinePlan],
    runs: &mut Vec<UiQualifiedTextRunRecord>,
    glyphs: &mut Vec<UiQualifiedTextGlyphRecord>,
) -> Result<(), UiTextLayoutDenial> {
    let Some(line) = plans.last_mut().filter(|line| line.overflowed) else {
        return Ok(());
    };
    let maximum_width = i64::from(shaped.constraints().width_millipoints());
    let mut end = line.unit_end;
    while end > line.unit_start && units[end - 1].kind == UnitKind::HardBreak {
        end -= 1;
    }
    let style_index = units
        .get(end.saturating_sub(1))
        .or_else(|| units.get(end))
        .map(|unit| unit.style_index)
        .unwrap_or(0);
    let bidi_level = units
        .get(end.saturating_sub(1))
        .or_else(|| units.get(end))
        .map_or(0, |unit| unit.bidi_level);
    let style = shaped.styles()[style_index as usize].style();
    let language = Language::new(style.language()).expect("admitted language");
    let direction = if bidi_level.is_multiple_of(2) {
        Direction::LeftToRight
    } else {
        Direction::RightToLeft
    };
    let (slot, script_tag) = fonts
        .fallback_slots(false, style)
        .find_map(|slot| {
            let probe = fonts.probe(slot, "\u{2026}", direction, &language, style, false);
            (probe.variation_qualified && !probe.has_notdef)
                .then_some((slot, probe.script.tag().to_be_bytes()))
        })
        .ok_or(UiTextLayoutDenial::EllipsisRequiresQualifiedGlyph)?;
    let ellipsis = fonts.shape_run(slot, "\u{2026}", 0, direction, &language, script_tag, style);
    if runs.len() >= shaped.capacity().runs() as usize {
        return Err(UiTextLayoutDenial::RunCapacityExceeded);
    }
    if glyphs
        .len()
        .checked_add(ellipsis.glyphs.len())
        .is_none_or(|count| count > shaped.capacity().glyphs() as usize)
    {
        return Err(UiTextLayoutDenial::GlyphCapacityExceeded);
    }
    let raw_advance = ellipsis
        .glyphs
        .iter()
        .map(|glyph| i64::from(glyph.x_advance).unsigned_abs())
        .sum();
    // Spacing may pull the ellipsis back over its own glyphs, but like every
    // other unit it never advances backward.
    let advance = (scale_advance(raw_advance, style, ellipsis.units_per_em)
        + i64::from(style.letter_spacing_millipoints()))
    .max(0);
    while end > line.unit_start
        && measure(shaped, &units[line.unit_start..end]) + advance > maximum_width
    {
        end -= 1;
    }
    let boundary = units
        .get(end)
        .map(|unit| unit.original_range.start())
        .or_else(|| {
            units
                .get(end.saturating_sub(1))
                .map(|unit| unit.original_range.end())
        })
        .unwrap_or(0);
    let empty_range = UiTextOriginalRange::from_text_mechanics(boundary, boundary)
        .expect("ellipsis maps to a source boundary");
    let glyph_start = u32::try_from(glyphs.len()).expect("glyph cap fits u32");
    for glyph in ellipsis.glyphs {
        glyphs.push(UiQualifiedTextGlyphRecord::from_text_mechanics(
            UiQualifiedTextGlyphInput {
                glyph_id: glyph.glyph_id,
                original_range: empty_range,
                x_advance_font_units: glyph.x_advance,
                y_advance_font_units: glyph.y_advance,
                x_offset_font_units: glyph.x_offset,
                y_offset_font_units: glyph.y_offset,
                ink_bounds_font_units: UiTextFontUnitRect::from_text_mechanics(
                    glyph.ink_bounds.x_min,
                    glyph.ink_bounds.y_min,
                    glyph.ink_bounds.x_max,
                    glyph.ink_bounds.y_max,
                )
                .expect("font-derived ellipsis bounds are ordered"),
            },
        ));
    }
    let glyph_end = u32::try_from(glyphs.len()).expect("glyph cap fits u32");
    let run_index = u32::try_from(runs.len()).expect("run cap fits u32");
    runs.push(UiQualifiedTextRunRecord::from_text_mechanics(
        UiQualifiedTextRunInput {
            original_range: empty_range,
            glyph_start,
            glyph_end,
            face: fonts.face_identity(slot),
            script_tag,
            bidi_level,
            units_per_em: ellipsis.units_per_em,
            style_index,
            ascender_font_units: ellipsis.ascender_font_units,
            descender_font_units: ellipsis.descender_font_units,
            line_gap_font_units: ellipsis.line_gap_font_units,
        },
    ));
    units.truncate(end);
    units.push(LayoutUnit {
        original_range: empty_range,
        glyph_range: glyph_start..glyph_end,
        logical_run_index: run_index,
        bidi_level,
        style_index,
        advance_millipoints: advance,
        kind: UnitKind::Glyphs,
    });
    line.unit_end = end + 1;
    line.hard_break = false;
    line.width_millipoints = measure(shaped, &units[line.unit_start..line.unit_end]);
    Ok(())
}
