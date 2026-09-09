use crate::native::presentation::text::mechanic_contains_run;
use worth_ui_host_contract::{
    UiGlyphRunView, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiMountedPresentationWorkView, UiMountedSemanticTextMechanic, UiMountedTextRasterWork,
};

pub(in crate::native::mechanics_adapter) fn admits(
    view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    raster: &UiMountedTextRasterWork<'_>,
) -> bool {
    let shape = demand_shape_admits(
        raster.glyph_runs().len(),
        raster
            .demands()
            .iter()
            .map(|batch| batch.records().len())
            .sum(),
    );
    let runs = raster
        .glyph_runs()
        .iter()
        .all(|run| admits_run(view, raster, *run));
    let demands = raster.demands().iter().all(|batch| {
        batch.records().iter().all(|record| {
            raster
                .glyph_runs()
                .iter()
                .any(|run| run.raster_key() == record.key())
        })
    });
    shape && runs && demands
}

fn demand_shape_admits(glyph_run_count: usize, demand_record_count: usize) -> bool {
    (glyph_run_count == 0) == (demand_record_count == 0)
}

fn admits_run(
    view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    raster: &UiMountedTextRasterWork<'_>,
    run: UiGlyphRunView,
) -> bool {
    demand_contains_run(raster, run)
        && semantic_mechanic(view.presentation_work(), run).is_some_and(|mechanic| {
            view.qualified_text_layout(mechanic)
                .is_some_and(|layout| mechanic_contains_run(mechanic, layout, run))
        })
}

fn demand_contains_run(raster: &UiMountedTextRasterWork<'_>, run: UiGlyphRunView) -> bool {
    raster.demands().iter().any(|batch| {
        batch.layout_identity() == run.layout_identity()
            && batch.records().iter().any(|record| {
                record.key() == run.raster_key()
                    && record.attribution().original_range() == run.original_range()
            })
    })
}

fn semantic_mechanic(
    presentation: UiMountedPresentationWorkView<'_>,
    run: UiGlyphRunView,
) -> Option<&UiMountedSemanticTextMechanic> {
    match presentation {
        UiMountedPresentationWorkView::Initial(initial) => initial
            .commands()
            .iter()
            .find_map(|command| matching_mechanic(command, run)),
        UiMountedPresentationWorkView::Reconstruction(reconstruction) => reconstruction
            .commands()
            .iter()
            .find_map(|command| matching_mechanic(command, run)),
        UiMountedPresentationWorkView::Delta(delta) => {
            delta.changes().iter().find_map(|change| match change {
                UiMountedPaintCommandChange::Insert(command)
                | UiMountedPaintCommandChange::Replace {
                    successor: command, ..
                } => matching_mechanic(command, run),
                UiMountedPaintCommandChange::Remove(_) => None,
            })
        }
        UiMountedPresentationWorkView::Sample(_) | UiMountedPresentationWorkView::Unchanged(_) => {
            None
        }
    }
}

fn matching_mechanic(
    command: &UiMountedPaintCommand,
    run: UiGlyphRunView,
) -> Option<&UiMountedSemanticTextMechanic> {
    match command {
        UiMountedPaintCommand::SemanticText { identity, mechanic }
            if *identity == run.mechanic() =>
        {
            Some(mechanic)
        }
        UiMountedPaintCommand::FilledRect { .. }
        | UiMountedPaintCommand::PortalOverlay { .. }
        | UiMountedPaintCommand::SemanticText { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::demand_shape_admits;

    #[test]
    fn release_only_work_admits_no_runs_and_no_demands() {
        assert!(demand_shape_admits(0, 0));
        assert!(demand_shape_admits(2, 2));
        assert!(!demand_shape_admits(0, 1));
        assert!(!demand_shape_admits(1, 0));
    }
}
