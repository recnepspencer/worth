use super::units::{advance_at, LayoutUnit, UnitKind};
use crate::{UiShapedTextParagraph, UiTextWrap};

mod widths;

pub(super) use widths::{FitWidths, ReflowWidths};

#[derive(Clone, Debug)]
pub(super) struct LinePlan {
    pub(super) unit_start: usize,
    pub(super) unit_end: usize,
    pub(super) width_millipoints: i64,
    pub(super) hard_break: bool,
    pub(super) overflowed: bool,
}

pub(super) fn fit(
    shaped: &UiShapedTextParagraph,
    units: &[LayoutUnit],
) -> (Vec<LinePlan>, FitWidths) {
    let constraints = shaped.constraints();
    let maximum_width = i64::from(constraints.width_millipoints());
    let maximum_lines = constraints.maximum_lines() as usize;
    let mut widths = FitWidths::default();
    if units.is_empty() {
        let empty = LinePlan {
            unit_start: 0,
            unit_end: 0,
            width_millipoints: 0,
            hard_break: false,
            overflowed: false,
        };
        return (vec![empty], widths);
    }
    let mut lines = Vec::new();
    let mut start = 0usize;
    while start < units.len() && lines.len() < maximum_lines {
        let mut index = start;
        let mut width = 0i64;
        let mut last_break = None;
        // The widest width the scan compared, and the widest it compared up
        // to its last break with the width fitted there. Only comparisons
        // that chose the line's end bound it, so any width that reaches the
        // same end bounds it alike.
        let mut compared = 0i64;
        let mut compared_to_break = (0i64, 0i64);
        let mut closed = false;
        while index < units.len() {
            if units[index].kind == UnitKind::HardBreak {
                widths.hold(compared);
                lines.push(LinePlan {
                    unit_start: start,
                    unit_end: index + 1,
                    width_millipoints: width,
                    hard_break: true,
                    overflowed: width > maximum_width,
                });
                start = index + 1;
                closed = true;
                break;
            }
            let advance = advance_at(&units[index], width, constraints.tab_interval_millipoints());
            let wraps = constraints.wrap() != UiTextWrap::None;
            if wraps && index > start && width + advance > maximum_width {
                let end = match last_break.filter(|end| *end > start) {
                    Some(end) => {
                        let (held, break_width) = compared_to_break;
                        widths.hold(held);
                        widths.wrap_at(rewrap_width(shaped, &units[end..], break_width));
                        end
                    }
                    None => {
                        widths.hold(compared);
                        widths.wrap_at(width + advance);
                        index
                    }
                };
                let fitted_width = measure(shaped, &units[start..end]);
                lines.push(LinePlan {
                    unit_start: start,
                    unit_end: end,
                    width_millipoints: fitted_width,
                    hard_break: false,
                    overflowed: fitted_width > maximum_width,
                });
                start = end;
                closed = true;
                break;
            }
            if wraps && index > start {
                compared = compared.max(width + advance);
            }
            width += advance;
            if break_allowed(shaped, &units[index]) {
                last_break = Some(index + 1);
                compared_to_break = (compared, width);
            }
            index += 1;
        }
        if !closed {
            widths.hold(compared);
            lines.push(LinePlan {
                unit_start: start,
                unit_end: units.len(),
                width_millipoints: width,
                hard_break: false,
                overflowed: width > maximum_width,
            });
            start = units.len();
        }
    }
    if start < units.len() {
        if let Some(last) = lines.last_mut() {
            last.overflowed = true;
        }
    } else if units
        .last()
        .is_some_and(|unit| unit.kind == UnitKind::HardBreak)
        && lines.len() < maximum_lines
    {
        lines.push(LinePlan {
            unit_start: units.len(),
            unit_end: units.len(),
            width_millipoints: 0,
            hard_break: false,
            overflowed: false,
        });
    }
    for line in &lines {
        widths.hold(line.width_millipoints);
    }
    (lines, widths)
}

/// The least maximum width at which a line that wrapped back to a break,
/// with `width` fitted before the `rest` after it, would end anywhere else.
/// Any narrower maximum still stops the scan before it passes the next break
/// opportunity, a hard break, or the end of the paragraph, so the line keeps
/// its end.
fn rewrap_width(shaped: &UiShapedTextParagraph, rest: &[LayoutUnit], mut width: i64) -> i64 {
    let tab_interval = shaped.constraints().tab_interval_millipoints();
    let mut limit = width;
    for unit in rest {
        if unit.kind == UnitKind::HardBreak {
            break;
        }
        width += advance_at(unit, width, tab_interval);
        limit = limit.max(width);
        if break_allowed(shaped, unit) {
            break;
        }
    }
    limit
}

fn break_allowed(shaped: &UiShapedTextParagraph, unit: &LayoutUnit) -> bool {
    match shaped.constraints().wrap() {
        UiTextWrap::None => false,
        UiTextWrap::Grapheme => true,
        UiTextWrap::UnicodeWord => shaped
            .line_opportunities()
            .binary_search(&unit.original_range.end())
            .is_ok(),
    }
}

pub(super) fn measure(shaped: &UiShapedTextParagraph, units: &[LayoutUnit]) -> i64 {
    units.iter().fold(0, |width, unit| {
        width + advance_at(unit, width, shaped.constraints().tab_interval_millipoints())
    })
}
