use super::*;
use worth_store_physical_format::RecordArtifactFile;

type AbsentInlineTarget<'a> = (u64, u64, u64, &'a PhysicalRedoTarget);

pub(super) fn admit_inline_allocations(
    root: &DurablePhysicalRootManifest,
    placements: &[CurrentPhysicalRecordPlacement],
    targets: &[&PhysicalRedoTarget],
    header: &DurableFreeSpaceManifestHeader,
    free_entries: &[RecordFreeSpaceManifestEntry],
) -> Result<(), PageObservationFailure> {
    let absent = absent_inline_targets(targets);
    if absent.is_empty() {
        return Ok(());
    }
    admit_initial_page_sequence(&absent, header.next_page())?;
    let Some(last) = root.last_inline_segment() else {
        return admit_new_segments(
            &absent,
            header.next_segment(),
            header.segment_page_capacity(),
        );
    };
    let selected_target = absent[0].3.identity();
    let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, last.segment_id().get())
        .expect("selected segment identity is nonzero");
    let reusable_entry = free_entries
        .binary_search_by_key(&(key.class() as u8, key.owner()), |entry| {
            (entry.class() as u8, entry.owner())
        })
        .ok()
        .map(|index| free_entries[index]);
    let reusable_pages = selected_reusable_pages(
        placements,
        last.segment_id().get(),
        last.generation().get(),
        header.segment_page_capacity(),
        reusable_entry,
        selected_target,
    )?;
    let required_reused = absent
        .len()
        .min(usize::try_from(reusable_pages).unwrap_or(usize::MAX));
    let (reused, new) = absent.split_at(required_reused);
    admit_reused_targets(reused, last.segment_id().get(), last.generation().get())?;
    for (segment, _, _, target) in new {
        if *segment == last.segment_id().get() {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        }
    }
    admit_new_segments(new, header.next_segment(), header.segment_page_capacity())
}

fn absent_inline_targets<'a>(targets: &[&'a PhysicalRedoTarget]) -> Vec<AbsentInlineTarget<'a>> {
    let mut absent = targets
        .iter()
        .filter_map(|target| match target.identity() {
            PhysicalRedoTargetIdentity::InlinePage {
                segment,
                page,
                generation,
            } => Some((segment, page, generation, *target)),
            PhysicalRedoTargetIdentity::ExtentChunk { .. } => None,
        })
        .collect::<Vec<_>>();
    absent.sort_unstable_by_key(|(segment, page, generation, _)| (*page, *segment, *generation));
    absent.dedup_by_key(|(segment, page, generation, _)| (*segment, *page, *generation));
    absent
}

fn admit_initial_page_sequence(
    absent: &[AbsentInlineTarget<'_>],
    next_page: u64,
) -> Result<(), PageObservationFailure> {
    for (_, _, generation, target) in absent {
        if *generation != 1 {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        }
    }
    if !allocation_sequence_above(absent.iter().map(|(_, page, _, _)| *page), next_page) {
        return Err(PageObservationFailure::InvalidTarget(
            absent[0].3.identity(),
        ));
    }
    Ok(())
}

fn selected_reusable_pages(
    placements: &[CurrentPhysicalRecordPlacement],
    segment: u64,
    generation: u64,
    expected_capacity: u32,
    entry: Option<RecordFreeSpaceManifestEntry>,
    target: PhysicalRedoTargetIdentity,
) -> Result<u64, PageObservationFailure> {
    let Some(entry) = entry else {
        return Ok(0);
    };
    let selected = placements.iter().filter_map(|placement| match placement {
        CurrentPhysicalRecordPlacement::Inline(value) if value.segment().get() == segment => {
            Some((value.page().get(), value.segment_page_capacity()))
        }
        _ => None,
    });
    let (pages, capacities): (BTreeSet<_>, BTreeSet<_>) = selected.unzip();
    let [capacity] = capacities.into_iter().collect::<Vec<_>>()[..] else {
        return Err(PageObservationFailure::InvalidTarget(target));
    };
    if capacity != expected_capacity {
        return Err(PageObservationFailure::InvalidTarget(target));
    }
    reusable_capacity(entry, generation, pages.len() as u64, capacity)
        .ok_or(PageObservationFailure::InvalidTarget(target))
}

fn admit_reused_targets(
    targets: &[AbsentInlineTarget<'_>],
    selected_segment: u64,
    selected_generation: u64,
) -> Result<(), PageObservationFailure> {
    for (candidate_segment, _, _, target) in targets {
        let RecordArtifactFile::Segment {
            segment: artifact_segment,
            generation: artifact_generation,
        } = target.artifact()
        else {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        };
        if *candidate_segment != selected_segment
            || artifact_segment != selected_segment
            || selected_generation.checked_add(1) != Some(artifact_generation)
        {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        }
    }
    Ok(())
}

fn admit_new_segments(
    targets: &[AbsentInlineTarget<'_>],
    next_segment: u64,
    page_capacity: u32,
) -> Result<(), PageObservationFailure> {
    let mut expected_segment = next_segment;
    let mut previous_segment = None;
    let mut pages_in_segment = 0_u32;
    for (segment, _, _, target) in targets {
        if previous_segment != Some(*segment) {
            if previous_segment.is_some() && pages_in_segment != page_capacity {
                return Err(PageObservationFailure::InvalidTarget(target.identity()));
            }
            if *segment < expected_segment {
                return Err(PageObservationFailure::InvalidTarget(target.identity()));
            }
            expected_segment = segment
                .checked_add(1)
                .ok_or(PageObservationFailure::InvalidTarget(target.identity()))?;
            previous_segment = Some(*segment);
            pages_in_segment = 0;
        }
        pages_in_segment = pages_in_segment
            .checked_add(1)
            .filter(|count| *count <= page_capacity)
            .ok_or(PageObservationFailure::InvalidTarget(target.identity()))?;
        let RecordArtifactFile::Segment {
            segment: artifact_segment,
            generation,
        } = target.artifact()
        else {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        };
        if artifact_segment != *segment || generation != 1 {
            return Err(PageObservationFailure::InvalidTarget(target.identity()));
        }
    }
    Ok(())
}
