//! Separately authored exact cost model for the production locality matrix.

use worth_ui_host_native::UiNativeTextAtlasPlanObservation as AtlasPlan;
use worth_ui_native_platform::UiNativeClientTextPresentationWorkObservation as TextWork;

use super::case::{Phase5LocalityAxis as Axis, Phase5LocalityCase};

pub(super) fn adjudicate(
    case: Phase5LocalityCase,
    work: &[TextWork],
    plans: &[AtlasPlan],
) -> Result<(), String> {
    let final_work = work
        .last()
        .ok_or_else(|| "dependency model received no text work".to_owned())?;
    match case.axis() {
        Axis::Content => {
            require_plan_count(plans, 3)?;
            require_local_text(final_work, 2, 2, 21)?;
            require_performed_layout(final_work)?;
            require_zero_raster(final_work)?;
        }
        Axis::Width => {
            require_plan_count(plans, 3)?;
            require_local_text(final_work, 2, 2, 35)?;
            require_performed_layout(final_work)?;
            require_counts(
                final_work,
                WorkCounts {
                    pin_additions: 35,
                    pin_releases: 35,
                    ..WorkCounts::ZERO
                },
            )?;
        }
        Axis::PaintBoundary => {
            require_plan_count(plans, 3)?;
            require_local_text(final_work, 2, 3, 23)?;
            require_performed_layout(final_work)?;
            require_counts(
                final_work,
                WorkCounts {
                    pin_additions: 4,
                    pin_releases: 4,
                    ..WorkCounts::ZERO
                },
            )?;
        }
        Axis::Dpi => {
            require_plan_count(plans, 5)?;
            require_dpi_reconstruction(plans)?;
            require_local_text(final_work, twice(case), twice(case), 23 * retained(case))?;
            require_zero_layout(final_work)?;
            require_zero_raster(final_work)?;
        }
        Axis::AtlasMiss => {
            require_plan_count(plans, 4)?;
            require_one_miss(plans)?;
            let produced = penultimate(work)?;
            require_local_text(produced, 2, 2, 23)?;
            require_performed_layout(produced)?;
            require_one_raster(produced)?;
            require_local_text(final_work, 2, 2, 23)?;
            require_zero_layout(final_work)?;
            require_zero_raster(final_work)?;
        }
        Axis::UploadCompletion => {
            require_plan_count(plans, 4)?;
            require_one_miss(plans)?;
            let produced = penultimate(work)?;
            require_local_text(produced, twice(case), twice(case), 23 * retained(case))?;
            require_performed_layout(produced)?;
            require_one_raster(produced)?;
            require_local_text(final_work, twice(case), twice(case), 23 * retained(case))?;
            require_zero_layout(final_work)?;
            require_zero_raster(final_work)?;
        }
        Axis::PinRelease => {
            require_plan_count(plans, 3)?;
            require_local_text(final_work, 0, 0, 0)?;
            require_zero_layout(final_work)?;
            require_counts(
                final_work,
                WorkCounts {
                    pin_releases: if case.retained_paragraphs() == 1 {
                        21
                    } else {
                        2
                    },
                    removed_mechanics: 2,
                    ..WorkCounts::ZERO
                },
            )?;
            require(
                penultimate(work)?.binding_pins() > final_work.binding_pins(),
                "pin-release successor did not lower the retained pin inventory",
            )?;
        }
    }
    Ok(())
}

fn require_performed_layout(work: &TextWork) -> Result<(), String> {
    require(
        work.analyzed_bytes() > 0,
        "layout work omitted analyzed bytes",
    )?;
    require(
        work.bidi_contexts() > 0,
        "layout work omitted bidi contexts",
    )?;
    require(
        work.fallback_clusters() > 0,
        "layout work omitted fallback clusters",
    )?;
    require(work.shaped_runs() > 0, "layout work omitted shaped runs")?;
    require(
        work.emitted_glyphs() > 0,
        "layout work omitted emitted glyphs",
    )
}

fn require_zero_layout(work: &TextWork) -> Result<(), String> {
    let performed = [
        work.analyzed_bytes(),
        work.graphemes(),
        work.word_boundaries(),
        work.line_opportunities(),
        work.bidi_contexts(),
        work.fallback_clusters(),
        work.coverage_index_queries(),
        work.face_shape_attempts(),
        work.probed_glyphs(),
        work.shaped_runs(),
        work.shaped_scalars(),
        work.emitted_glyphs(),
        work.fitted_units(),
        work.emitted_lines(),
        work.emitted_visual_runs(),
        work.positioned_glyphs(),
        work.emitted_carets(),
    ];
    require(
        performed.iter().all(|count| *count == 0),
        "layout reuse reported qualification work",
    )
}

fn require_dpi_reconstruction(plans: &[AtlasPlan]) -> Result<(), String> {
    let produced = plans
        .get(plans.len().saturating_sub(2))
        .ok_or_else(|| "DPI model has no reconstruction plan".to_owned())?;
    let completed = plans
        .last()
        .ok_or_else(|| "DPI model has no completion plan".to_owned())?;
    require_equal("DPI reconstruction lookups", produced.key_lookups(), 22)?;
    require_equal("DPI reconstruction hits", produced.hits(), 0)?;
    require_equal("DPI reconstruction misses", produced.misses(), 22)?;
    require(
        produced.physical_staged_bytes() > 0,
        "DPI reconstruction staged zero physical bytes",
    )?;
    require_equal("DPI completion hits", completed.hits(), 22)?;
    require_equal("DPI completion misses", completed.misses(), 0)
}

fn require_local_text(
    work: &TextWork,
    layouts: u64,
    paint_spans: u64,
    key_checks: u64,
) -> Result<(), String> {
    require_equal("layouts", work.layout_count(), layouts)?;
    require_equal("paint spans", work.paint_span_count(), paint_spans)?;
    require_equal("demand records", work.demand_records(), key_checks)?;
    require_equal("raster key checks", work.key_checks(), key_checks)
}

fn require_one_miss(plans: &[AtlasPlan]) -> Result<(), String> {
    let produced = plans
        .get(plans.len().saturating_sub(2))
        .ok_or_else(|| "atlas model has no production plan".to_owned())?;
    let completed = plans
        .last()
        .ok_or_else(|| "atlas model has no completion plan".to_owned())?;
    require_equal("atlas production lookups", produced.key_lookups(), 23)?;
    require_equal("atlas production hits", produced.hits(), 22)?;
    require_equal("atlas production misses", produced.misses(), 1)?;
    require(produced.staged_bytes() > 0, "atlas miss staged zero bytes")?;
    require(
        produced.physical_staged_bytes() > 0,
        "atlas miss staged zero physical bytes",
    )?;
    require_equal("atlas completion hits", completed.hits(), 23)?;
    require_equal("atlas completion misses", completed.misses(), 0)
}

fn require_one_raster(work: &TextWork) -> Result<(), String> {
    require_equal("rasterized glyphs", work.rasterized_glyphs(), 1)?;
    require(
        work.rasterized_texels() > 0,
        "one miss produced zero texels",
    )?;
    require(work.produced_bytes() > 0, "one miss produced zero bytes")
}

fn require_zero_raster(work: &TextWork) -> Result<(), String> {
    require_equal("rasterized glyphs", work.rasterized_glyphs(), 0)?;
    require_equal("rasterized texels", work.rasterized_texels(), 0)?;
    require_equal("produced bytes", work.produced_bytes(), 0)
}

#[derive(Clone, Copy)]
struct WorkCounts {
    pin_additions: u64,
    pin_releases: u64,
    removed_mechanics: u64,
}

impl WorkCounts {
    const ZERO: Self = Self {
        pin_additions: 0,
        pin_releases: 0,
        removed_mechanics: 0,
    };
}

fn require_counts(work: &TextWork, expected: WorkCounts) -> Result<(), String> {
    require_equal(
        "pin additions",
        work.pin_additions(),
        expected.pin_additions,
    )?;
    require_equal("pin releases", work.pin_releases(), expected.pin_releases)?;
    require_equal(
        "removed mechanics",
        work.removed_mechanics(),
        expected.removed_mechanics,
    )
}

fn require_plan_count(plans: &[AtlasPlan], expected: usize) -> Result<(), String> {
    require_equal("atlas plan count", plans.len() as u64, expected as u64)
}

fn penultimate(work: &[TextWork]) -> Result<&TextWork, String> {
    work.get(work.len().saturating_sub(2))
        .ok_or_else(|| "dependency model has no production work row".to_owned())
}

fn retained(case: Phase5LocalityCase) -> u64 {
    case.retained_paragraphs() as u64
}

fn twice(case: Phase5LocalityCase) -> u64 {
    retained(case) * 2
}

fn require_equal(label: &str, actual: u64, expected: u64) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "dependency model {label}: expected {expected}, observed {actual}"
        ))
    }
}

fn require(condition: bool, message: &str) -> Result<(), String> {
    condition.then_some(()).ok_or_else(|| message.to_owned())
}
