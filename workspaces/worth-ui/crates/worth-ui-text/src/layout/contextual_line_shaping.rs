use super::{
    line_fitting::{self, FitWidths, LinePlan},
    units::{self, LayoutUnit, UnitKind},
    UiTextLayoutDenial,
};
use crate::UiShapedTextParagraph;

pub(super) struct FittedLines {
    pub(super) units: Vec<LayoutUnit>,
    pub(super) plans: Vec<LinePlan>,
    /// The widths every fit on the way to `plans` compared, met together:
    /// any of them reaches the same reshaping and the same lines.
    pub(super) widths: FitWidths,
}

pub(super) fn fit(shaped: &mut UiShapedTextParagraph) -> Result<FittedLines, UiTextLayoutDenial> {
    let mut applied = Vec::<u32>::new();
    let mut previous_plans = Vec::<Vec<u32>>::new();
    let mut widths = FitWidths::default();
    loop {
        let units = units::build(shaped);
        let (plans, fit_widths) = line_fitting::fit(shaped, &units);
        widths = widths.meet(fit_widths);
        let boundaries = contextual_boundaries(shaped, &units, &plans);
        if boundaries == applied {
            return Ok(FittedLines {
                units,
                plans,
                widths,
            });
        }
        if previous_plans
            .iter()
            .any(|previous| previous == &boundaries)
            || previous_plans.len() >= shaped.constraints().maximum_lines() as usize
        {
            return freeze_current_plan(shaped, &units, &plans, &boundaries, widths);
        }
        previous_plans.push(boundaries.clone());
        shaped
            .reshape_at_line_boundaries(&boundaries)
            .map_err(UiTextLayoutDenial::ContextualReshapingFailed)?;
        applied = boundaries;
    }
}

fn freeze_current_plan(
    shaped: &mut UiShapedTextParagraph,
    units: &[LayoutUnit],
    plans: &[LinePlan],
    boundaries: &[u32],
    mut widths: FitWidths,
) -> Result<FittedLines, UiTextLayoutDenial> {
    let frozen = plans
        .iter()
        .map(|plan| {
            let start = units.get(plan.unit_start).map_or_else(
                || u32::try_from(shaped.source().len()).expect("admitted source fits u32"),
                |unit| unit.original_range.start(),
            );
            let end = plan
                .unit_end
                .checked_sub(1)
                .and_then(|index| units.get(index))
                .map_or(start, |unit| unit.original_range.end());
            (start, end, plan.hard_break, plan.overflowed)
        })
        .collect::<Vec<_>>();
    shaped
        .reshape_at_line_boundaries(boundaries)
        .map_err(UiTextLayoutDenial::ContextualReshapingFailed)?;
    let units = units::build(shaped);
    let maximum_width = i64::from(shaped.constraints().width_millipoints());
    let plans = frozen
        .into_iter()
        .map(|(start, end, hard_break, was_overflowed)| {
            let unit_start = units
                .iter()
                .position(|unit| unit.original_range.start() >= start)
                .unwrap_or(units.len());
            let unit_end = units
                .iter()
                .position(|unit| unit.original_range.end() >= end)
                .map_or(units.len(), |index| index + usize::from(end > start));
            let width = line_fitting::measure(shaped, &units[unit_start..unit_end]);
            widths.hold(width);
            LinePlan {
                unit_start,
                unit_end,
                width_millipoints: width,
                hard_break,
                overflowed: was_overflowed || width > maximum_width,
            }
        })
        .collect();
    Ok(FittedLines {
        units,
        plans,
        widths,
    })
}

fn contextual_boundaries(
    shaped: &UiShapedTextParagraph,
    units: &[LayoutUnit],
    plans: &[LinePlan],
) -> Vec<u32> {
    plans
        .iter()
        .filter(|plan| !plan.hard_break && plan.unit_end < units.len())
        .filter(|plan| {
            plan.unit_end > plan.unit_start && units[plan.unit_end - 1].kind != UnitKind::HardBreak
        })
        .map(|plan| units[plan.unit_end].original_range.start())
        .filter(|boundary| shaped.break_is_contextual(*boundary))
        .collect()
}
