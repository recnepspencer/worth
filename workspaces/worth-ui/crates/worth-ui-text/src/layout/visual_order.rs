use unicode_bidi::{BidiInfo, Level};

use super::{
    line_fitting::LinePlan,
    units::{advance_at, LayoutUnit, UnitKind},
};
use crate::{UiShapedTextParagraph, UiTextAlignment, UiTextBaseDirection};

#[derive(Clone, Debug)]
pub(super) struct PositionedUnit {
    pub(super) unit_index: usize,
    pub(super) x_millipoints: i64,
    pub(super) width_millipoints: i64,
}

#[derive(Clone, Debug)]
pub(super) struct VisualGroup {
    pub(super) unit_start: usize,
    pub(super) unit_end: usize,
    pub(super) logical_run_index: u32,
    pub(super) bidi_level: u8,
    pub(super) x_start_millipoints: i64,
    pub(super) x_end_millipoints: i64,
}

#[derive(Clone, Debug)]
pub(super) struct VisualLine {
    pub(super) units: Vec<PositionedUnit>,
    pub(super) groups: Vec<VisualGroup>,
    pub(super) x_offset_millipoints: i64,
    pub(super) base_level: u8,
}

pub(super) fn order(
    shaped: &UiShapedTextParagraph,
    all_units: &[LayoutUnit],
    line: &LinePlan,
) -> VisualLine {
    let units = &all_units[line.unit_start..line.unit_end];
    let visible_len = match units
        .iter()
        .position(|unit| unit.kind == UnitKind::HardBreak)
    {
        // A line that holds nothing but its break is an empty line, and an
        // empty line is carried by its anchor. Ordering the break here would
        // give that line a run and take its anchor, and with it the only caret
        // the reader has to land on.
        Some(0) => 0,
        // A hard break paints nothing and advances nothing, but it belongs to
        // the line it ends: the line's original range reaches it and an
        // authored span may style it. Ordering it keeps the runs of a line
        // covering the whole of what that line spans.
        Some(index) => index + 1,
        None => units.len(),
    };
    let visible = &units[..visible_len];
    let probe = visible
        .first()
        .or_else(|| units.first())
        .map(|unit| unit.original_range.start())
        .or_else(|| {
            line.unit_start
                .checked_sub(1)
                .and_then(|index| all_units.get(index))
                .map(|unit| unit.original_range.end())
        })
        .unwrap_or(0);
    let base_level = paragraph_level(shaped, probe);
    let mut levels = visible
        .iter()
        .map(|unit| Level::new(unit.bidi_level).expect("Unicode bidi level"))
        .collect::<Vec<_>>();
    for (unit, level) in visible.iter().zip(levels.iter_mut()).rev() {
        let source = &shaped.source()
            [unit.original_range.start() as usize..unit.original_range.end() as usize];
        if source.chars().all(char::is_whitespace) {
            *level = Level::new(base_level).expect("paragraph bidi level");
        } else {
            break;
        }
    }
    let order = BidiInfo::reorder_visual(&levels);
    let widths = logical_widths(shaped, visible);
    let x_offset = alignment_offset(shaped, line.width_millipoints, base_level);
    let mut x = x_offset;
    let mut positioned = Vec::with_capacity(order.len());
    for logical_index in order {
        let width = widths[logical_index];
        positioned.push(PositionedUnit {
            unit_index: line.unit_start + logical_index,
            x_millipoints: x,
            width_millipoints: width,
        });
        x += width;
    }
    let groups = groups(all_units, &positioned);
    VisualLine {
        units: positioned,
        groups,
        x_offset_millipoints: x_offset,
        base_level,
    }
}

fn logical_widths(shaped: &UiShapedTextParagraph, units: &[LayoutUnit]) -> Vec<i64> {
    let mut x = 0i64;
    units
        .iter()
        .map(|unit| {
            let width = advance_at(unit, x, shaped.constraints().tab_interval_millipoints());
            x += width;
            width
        })
        .collect()
}

fn groups(all_units: &[LayoutUnit], units: &[PositionedUnit]) -> Vec<VisualGroup> {
    let mut groups = Vec::new();
    let mut start = 0usize;
    while start < units.len() {
        let first = &all_units[units[start].unit_index];
        let mut end = start + 1;
        while end < units.len() {
            let current = &all_units[units[end].unit_index];
            if current.logical_run_index != first.logical_run_index
                || current.bidi_level != first.bidi_level
            {
                break;
            }
            end += 1;
        }
        groups.push(VisualGroup {
            unit_start: start,
            unit_end: end,
            logical_run_index: first.logical_run_index,
            bidi_level: first.bidi_level,
            x_start_millipoints: units[start].x_millipoints,
            x_end_millipoints: units[end - 1].x_millipoints + units[end - 1].width_millipoints,
        });
        start = end;
    }
    groups
}

fn paragraph_level(shaped: &UiShapedTextParagraph, offset: u32) -> u8 {
    let paragraphs = shaped.bidi_paragraphs();
    paragraphs
        .iter()
        .find(|paragraph| (paragraph.start..paragraph.end).contains(&offset))
        .or_else(|| {
            paragraphs
                .iter()
                .rev()
                .find(|paragraph| paragraph.start <= offset)
        })
        .map_or_else(
            || match shaped.constraints().base_direction() {
                UiTextBaseDirection::RightToLeft => 1,
                UiTextBaseDirection::Auto | UiTextBaseDirection::LeftToRight => 0,
            },
            |paragraph| paragraph.level,
        )
}

fn alignment_offset(shaped: &UiShapedTextParagraph, width: i64, base_level: u8) -> i64 {
    let available = i64::from(shaped.constraints().width_millipoints());
    let remaining = (available - width).max(0);
    match shaped.constraints().alignment() {
        UiTextAlignment::Center => remaining / 2,
        UiTextAlignment::Start if base_level.is_multiple_of(2) => 0,
        UiTextAlignment::Start => remaining,
        UiTextAlignment::End if base_level.is_multiple_of(2) => remaining,
        UiTextAlignment::End => 0,
    }
}
