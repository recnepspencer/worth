use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    FreeSpaceKey, RecordAllocationClass, RecordFreeSpaceManifestEntry,
};
use worth_store_recovery_physics::{
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, RecoveryPageObservation,
};

use super::PageObservationFailure;
use crate::progression::RecoverySelectedSourceInventory;

mod inline_allocation;
#[cfg(test)]
mod tests;

use inline_allocation::admit_inline_allocations;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct AdmittedAbsentTargets {
    pub(super) observations: Vec<RecoveryPageObservation>,
    pub(super) inline_truth: Option<InlineAllocationTruth>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning) struct InlineAllocationTruth {
    pub(in crate::orchestration::planning) next_segment: u64,
    pub(in crate::orchestration::planning) page_capacity: u32,
}

pub(super) fn admit_absent_targets(
    root: &DurablePhysicalRootManifest,
    placements: &[CurrentPhysicalRecordPlacement],
    targets: Vec<&PhysicalRedoTarget>,
    selected_source: &RecoverySelectedSourceInventory,
    absence_identity: [u8; 32],
    admitted_redo: &worth_store_recovery_physics::AdmittedPhysicalRedoMembers,
) -> Result<AdmittedAbsentTargets, PageObservationFailure> {
    for target in &targets {
        if !admitted_redo.contains_exact_observation_target(target) {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        }
    }
    if targets.is_empty() {
        return Ok(AdmittedAbsentTargets {
            observations: Vec::new(),
            inline_truth: None,
        });
    }
    admit_inline_allocations(
        root,
        placements,
        &targets,
        &selected_source.free_space,
        &selected_source.free_entries,
    )?;
    admit_extent_allocations(&targets, &selected_source.free_space)?;
    Ok(AdmittedAbsentTargets {
        observations: targets
            .into_iter()
            .map(|target| RecoveryPageObservation::absent(target, absence_identity))
            .collect(),
        inline_truth: Some(InlineAllocationTruth {
            next_segment: selected_source.free_space.next_segment(),
            page_capacity: selected_source.free_space.segment_page_capacity(),
        }),
    })
}

fn admit_extent_allocations(
    targets: &[&PhysicalRedoTarget],
    header: &DurableFreeSpaceManifestHeader,
) -> Result<(), PageObservationFailure> {
    let mut extents = BTreeMap::new();
    for target in targets {
        let PhysicalRedoTargetIdentity::ExtentChunk {
            extent, generation, ..
        } = target.identity()
        else {
            continue;
        };
        if generation != 1 {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        }
        extents.entry(extent).or_insert(*target);
    }
    if !allocation_sequence_above(extents.keys().copied(), header.next_extent()) {
        return Err(PageObservationFailure::InvalidTarget(
            extents
                .into_values()
                .next()
                .expect("nonempty failed sequence")
                .identity(),
        ));
    }
    Ok(())
}

fn allocation_sequence_above(values: impl IntoIterator<Item = u64>, first: u64) -> bool {
    let mut prior = None;
    values.into_iter().all(|value| {
        let admissible = value >= first && prior.is_none_or(|prior| value > prior);
        prior = Some(value);
        admissible
    })
}

fn reusable_capacity(
    entry: RecordFreeSpaceManifestEntry,
    selected_generation: u64,
    selected_pages: u64,
    capacity: u32,
) -> Option<u64> {
    (entry.generation() == selected_generation
        && entry.first_unallocated() == selected_pages.saturating_add(1)
        && entry
            .first_unallocated()
            .checked_add(entry.unallocated_count())
            == Some(u64::from(capacity) + 1))
    .then_some(entry.unallocated_count())
}
