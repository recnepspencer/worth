//! Immutable source names and protected arena ranges for image construction.

use super::super::super::*;

pub(super) fn collect(
    selection: &PhysicalSourceSelection,
    source: &RecoverySelectedSourceInventory,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Box<[RecordArtifactFile]>, ExecutionBasisDenial> {
    let facts = selection.page_facts();
    let previous = selection.retained_previous_page_facts();
    let count = 4_usize
        .checked_add(facts.routing_blocks().len())
        .and_then(|n| n.checked_add(facts.placements().len()))
        .and_then(|n| n.checked_add(previous.map_or(0, |facts| facts.routing_blocks().len())))
        .and_then(|n| n.checked_add(previous.map_or(0, |facts| facts.placements().len())))
        .and_then(|n| n.checked_add(source.source_artifacts.len()))
        .ok_or(ExecutionBasisDenial::RecoveryMemoryBytes { observed: u64::MAX })?;
    let mut artifacts = allowance.reserve(count)?;
    let selected = selection.root().selected();
    artifacts.push(match selected.selector().role() {
        worth_store_physical_format::RootSelectorRole::Current => {
            RecordArtifactFile::CurrentRootSelector
        }
        worth_store_physical_format::RootSelectorRole::Previous => {
            RecordArtifactFile::PreviousRootSelector
        }
    });
    artifacts.push(RecordArtifactFile::RootManifest {
        generation: selected.manifest().generation(),
    });
    for facts in std::iter::once(facts).chain(previous) {
        artifacts.extend(facts.routing_blocks().iter().map(|reference| {
            RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
        }));
        artifacts.extend(facts.placements().iter().map(placement_artifact));
    }
    if let Some(previous) = selection.root().retained_previous() {
        artifacts.push(RecordArtifactFile::PreviousRootSelector);
        artifacts.push(RecordArtifactFile::RootManifest {
            generation: previous.manifest().generation(),
        });
    }
    artifacts.extend_from_slice(&source.source_artifacts);
    artifacts.sort_unstable();
    artifacts.dedup();
    allowance.into_box(artifacts).map_err(Into::into)
}

pub(super) fn protected_ranges(
    selection: &PhysicalSourceSelection,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<worth_store_physical_format::ExtentArenaRange>, ExecutionBasisDenial> {
    let placements = || {
        selection.page_facts().placements().iter().chain(
            selection
                .retained_previous_page_facts()
                .into_iter()
                .flat_map(|facts| facts.placements()),
        )
    };
    let count = placements()
        .filter(|placement| matches!(placement, CurrentPhysicalRecordPlacement::Extent(_)))
        .count();
    let mut ranges = allowance.reserve(count)?;
    ranges.extend(placements().filter_map(|placement| match placement {
        CurrentPhysicalRecordPlacement::Extent(extent) => Some(extent.arena_range()),
        _ => None,
    }));
    Ok(ranges)
}

fn placement_artifact(placement: &CurrentPhysicalRecordPlacement) -> RecordArtifactFile {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(inline) => RecordArtifactFile::Segment {
            segment: inline.segment().get(),
            generation: inline.segment_generation(),
        },
        CurrentPhysicalRecordPlacement::Extent(extent) => RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        },
    }
}
