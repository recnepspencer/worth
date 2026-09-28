use std::collections::{HashMap, HashSet};

use crate::native::presentation::text::mechanic_contains_run;
use worth_ui_host_contract::{
    UiGlyphRasterKey, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedPresentationWorkView, UiMountedSemanticTextMechanic,
    UiMountedTextRasterWork, UiQualifiedTextLayoutIdentity, UiTextOriginalRange,
};

/// Every run must answer a demand and a text command, and every demand a run;
/// each side is indexed once so admission stays linear in the frame's text.
pub(in crate::native::mechanics_adapter) fn admits(
    view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    raster: &UiMountedTextRasterWork<'_>,
) -> bool {
    let demands = raster
        .demands()
        .iter()
        .flat_map(|batch| {
            batch.records().iter().map(move |record| {
                (
                    batch.layout_identity(),
                    record.key(),
                    record.attribution().original_range(),
                )
            })
        })
        .collect::<HashSet<DemandedRun>>();
    let demand_records = raster
        .demands()
        .iter()
        .map(|batch| batch.records().len())
        .sum();
    if !demand_shape_admits(raster.glyph_runs().len(), demand_records) {
        return false;
    }
    let mechanics = semantic_mechanics(view.presentation_work());
    let runs_admitted = raster.glyph_runs().iter().all(|run| {
        demands.contains(&(
            run.layout_identity(),
            run.raster_key(),
            run.original_range(),
        )) && mechanics.get(&run.mechanic()).is_some_and(|mechanic| {
            view.qualified_text_layout(mechanic)
                .is_some_and(|layout| mechanic_contains_run(mechanic, layout, *run))
        })
    });
    let run_keys = raster
        .glyph_runs()
        .iter()
        .map(|run| run.raster_key())
        .collect::<HashSet<_>>();
    runs_admitted && demands.iter().all(|(_, key, _)| run_keys.contains(key))
}

type DemandedRun = (
    UiQualifiedTextLayoutIdentity,
    UiGlyphRasterKey,
    UiTextOriginalRange,
);

fn demand_shape_admits(glyph_run_count: usize, demand_record_count: usize) -> bool {
    (glyph_run_count == 0) == (demand_record_count == 0)
}

/// The first text mechanic each command identity names in this work.
fn semantic_mechanics(
    presentation: UiMountedPresentationWorkView<'_>,
) -> HashMap<UiMountedPaintCommandIdentity, &UiMountedSemanticTextMechanic> {
    let commands: Box<dyn Iterator<Item = &UiMountedPaintCommand>> = match presentation {
        UiMountedPresentationWorkView::Initial(initial) => Box::new(initial.commands().iter()),
        UiMountedPresentationWorkView::Reconstruction(reconstruction) => {
            Box::new(reconstruction.commands().iter())
        }
        UiMountedPresentationWorkView::Delta(delta) => {
            Box::new(delta.changes().iter().filter_map(|change| match change {
                UiMountedPaintCommandChange::Insert(command)
                | UiMountedPaintCommandChange::Replace {
                    successor: command, ..
                } => Some(command),
                UiMountedPaintCommandChange::Remove(_) => None,
            }))
        }
        UiMountedPresentationWorkView::Sample(_) | UiMountedPresentationWorkView::Unchanged(_) => {
            Box::new(std::iter::empty())
        }
    };
    let mut mechanics = HashMap::new();
    for (identity, mechanic) in commands.filter_map(text_mechanic) {
        mechanics.entry(identity).or_insert(mechanic);
    }
    mechanics
}

fn text_mechanic(
    command: &UiMountedPaintCommand,
) -> Option<(
    UiMountedPaintCommandIdentity,
    &UiMountedSemanticTextMechanic,
)> {
    match command {
        UiMountedPaintCommand::SemanticText { identity, mechanic } => Some((*identity, mechanic)),
        UiMountedPaintCommand::PortalOverlay { .. } => None,
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
