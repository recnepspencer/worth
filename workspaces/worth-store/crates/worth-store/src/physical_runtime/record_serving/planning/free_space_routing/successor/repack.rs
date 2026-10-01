use std::collections::btree_map;
use std::iter::Peekable;

mod writer;
use writer::PackedWriter;
mod budget;
use budget::LocalRepackBudget;

use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, FreeSpaceBlockReference, FreeSpaceKey,
    PhysicalFreeSpaceMembershipBlock, RecordArtifactFile, RecordFreeSpaceManifestEntry,
};

use super::super::super::super::RecordAppendDenial;
use super::{
    AdmittedPhysicalRecordFormat, FreeSpaceBlockSource, FreeSpacePublicationPlan,
    FreeSpaceSuccessorRequest, FreeSpaceUpdate, ManifestDiscoveryCounterSnapshot,
    ManifestLookupFailure, RecordAppendError,
};

fn metadata_pressure() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::from_residency(
        worth_store_buffer_pool::PhysicalResidencyDenial::MetadataBudgetExceeded,
    ))
}

fn damaged() -> RecordAppendError {
    super::super::super::inline_plan_failure::manifest_lookup_failure(
        ManifestLookupFailure::Damaged,
    )
}

fn divide_ceil(value: u64, divisor: u64) -> Option<u64> {
    value
        .checked_div(divisor)?
        .checked_add(u64::from(value % divisor != 0))
}

/// A local bound for rebuilding this tree, not an aggregate mutation heap grant.
/// Every staged frame is separately required to fit the admitted page width.
pub(super) fn local_peak_bound(
    entry_count: u64,
    capacity: u16,
    source_level: u16,
    source_capacity: u16,
    page_bytes: u64,
) -> Option<(usize, u64)> {
    let fanout = u64::from(capacity);
    if fanout < 2 || page_bytes == 0 {
        return None;
    }
    let mut level_count = entry_count;
    let mut blocks = 0_u64;
    let mut levels = 0_u64;
    while level_count != 0 {
        level_count = divide_ceil(level_count, fanout)?;
        blocks = blocks.checked_add(level_count)?;
        levels = levels.checked_add(1)?;
        if level_count == 1 {
            break;
        }
    }
    let block_slots = usize::try_from(blocks).ok()?;
    let block_headers = blocks
        .checked_mul(u64::try_from(std::mem::size_of::<(RecordArtifactFile, Vec<u8>)>()).ok()?)?;
    let frame_backing = blocks.checked_mul(page_bytes)?;
    let leaf_backing = fanout
        .checked_mul(u64::try_from(std::mem::size_of::<RecordFreeSpaceManifestEntry>()).ok()?)?
        .checked_mul(2)?;
    let level_slots = levels.max(1).checked_add(1)?;
    let level_headers = level_slots
        .checked_mul(u64::try_from(std::mem::size_of::<Vec<FreeSpaceBlockReference>>()).ok()?)?;
    let reference_backing = level_slots
        .checked_mul(fanout)?
        .checked_mul(u64::try_from(std::mem::size_of::<FreeSpaceBlockReference>()).ok()?)?
        .checked_mul(2)?;
    let source_fanout = u64::from(source_capacity);
    let source_node_backing = source_fanout.checked_mul(
        u64::try_from(
            std::mem::size_of::<FreeSpaceBlockReference>()
                .max(std::mem::size_of::<RecordFreeSpaceManifestEntry>()),
        )
        .ok()?,
    )?;
    let source_ancestors = u64::from(source_level)
        .checked_mul(source_fanout)?
        .checked_mul(u64::try_from(std::mem::size_of::<FreeSpaceBlockReference>()).ok()?)?;
    let source_read_window = page_bytes.checked_add(source_node_backing)?;
    let required = frame_backing
        .checked_add(block_headers)?
        .checked_add(leaf_backing)?
        .checked_add(level_headers)?
        .checked_add(reference_backing)?
        .checked_add(source_ancestors)?
        .checked_add(source_read_window)?;
    Some((block_slots, required))
}

pub(super) fn repack<S: FreeSpaceBlockSource>(
    source: &S,
    format: AdmittedPhysicalRecordFormat,
    current: &DurableFreeSpaceManifestHeader,
    request: &FreeSpaceSuccessorRequest,
    prior_discovery: ManifestDiscoveryCounterSnapshot,
    expected_changes: (u64, u64),
    local_metadata_limit: u64,
) -> Result<FreeSpacePublicationPlan, RecordAppendError> {
    let entry_count = current
        .entry_count()
        .checked_add(expected_changes.0)
        .and_then(|count| count.checked_sub(expected_changes.1))
        .ok_or_else(damaged)?;
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    let source_level = current.root().map_or(0, |root| root.level());
    let (max_blocks, required) = local_peak_bound(
        entry_count,
        request.node_capacity,
        source_level,
        current.node_capacity(),
        page_bytes,
    )
    .ok_or(metadata_pressure())?;
    let mut budget = LocalRepackBudget::new(local_metadata_limit, required)?;
    let mut writer = PackedWriter::new(
        format,
        current.tree_identity(),
        request.generation,
        request.node_capacity,
        current.next_block(),
        entry_count,
        max_blocks,
        page_bytes,
        &mut budget,
    )?;
    let mut traversal = RepackTraversal {
        source,
        writer: &mut writer,
        updates: request.updates.iter().peekable(),
        discovery: prior_discovery,
        budget: &mut budget,
        page_bytes,
        source_node_bytes: u64::from(current.node_capacity())
            .checked_mul(
                u64::try_from(
                    std::mem::size_of::<FreeSpaceBlockReference>()
                        .max(std::mem::size_of::<RecordFreeSpaceManifestEntry>()),
                )
                .map_err(|_| metadata_pressure())?,
            )
            .ok_or_else(metadata_pressure)?,
        last_source: None,
        source_count: 0,
        inserted: 0,
        removed: 0,
    };
    if let Some(root) = current.root() {
        traversal.walk(root)?;
    }
    traversal.finish_updates()?;
    let source_count = traversal.source_count;
    let observed_changes = (traversal.inserted, traversal.removed);
    let discovery = traversal.discovery;
    drop(traversal);
    if source_count != current.entry_count()
        || observed_changes != expected_changes
        || writer.emitted != entry_count
    {
        return Err(damaged());
    }
    let root = writer.finish(&mut budget)?;
    let header = DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        request.generation,
        current.tree_identity(),
        request.node_capacity,
        request.segment_page_capacity,
        entry_count,
        request.next_segment,
        request.next_page,
        request.next_extent,
        request.next_arena,
        current.tier_epoch_start(),
        current.arena_capacity(),
        current.arena_alignment(),
        writer.next_block,
        root,
    )
    .ok_or_else(damaged)?;
    Ok(FreeSpacePublicationPlan {
        header,
        blocks: writer.blocks,
        discovery,
    })
}

struct RepackTraversal<'a, S: FreeSpaceBlockSource> {
    source: &'a S,
    writer: &'a mut PackedWriter,
    budget: &'a mut LocalRepackBudget,
    page_bytes: u64,
    source_node_bytes: u64,
    updates: Peekable<btree_map::Iter<'a, FreeSpaceKey, FreeSpaceUpdate>>,
    discovery: ManifestDiscoveryCounterSnapshot,
    last_source: Option<FreeSpaceKey>,
    source_count: u64,
    inserted: u64,
    removed: u64,
}

impl<S: FreeSpaceBlockSource> RepackTraversal<'_, S> {
    fn walk(&mut self, reference: FreeSpaceBlockReference) -> Result<(), RecordAppendError> {
        self.budget
            .source_read_window(self.page_bytes, self.source_node_bytes)?;
        let block = self
            .source
            .read_block(reference, &mut self.discovery)
            .map_err(super::super::super::inline_plan_failure::manifest_lookup_failure)?;
        if block.owned_heap_bytes().ok_or_else(metadata_pressure)? > self.source_node_bytes {
            return Err(metadata_pressure());
        }
        match block {
            PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                self.budget.retain(&entries)?;
                let capacity = entries.capacity();
                for entry in entries.iter().copied() {
                    self.merge_source(entry)?;
                }
                drop(entries);
                self.budget
                    .release_capacity::<RecordFreeSpaceManifestEntry>(capacity);
            }
            PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                self.budget.retain(&children)?;
                let capacity = children.capacity();
                let mut children = children.into_iter();
                for child in children.by_ref() {
                    self.walk(child)?;
                }
                drop(children);
                self.budget
                    .release_capacity::<FreeSpaceBlockReference>(capacity);
            }
        }
        Ok(())
    }

    fn merge_source(
        &mut self,
        existing: RecordFreeSpaceManifestEntry,
    ) -> Result<(), RecordAppendError> {
        let key = FreeSpaceKey::from(existing);
        if self.last_source.is_some_and(|last| last >= key) {
            return Err(damaged());
        }
        self.last_source = Some(key);
        self.source_count = self.source_count.checked_add(1).ok_or_else(damaged)?;
        while self.updates.peek().is_some_and(|(next, _)| **next < key) {
            let (_, update) = self.updates.next().expect("peeked update");
            if let FreeSpaceUpdate::Available(entry) = update {
                self.writer.push(*entry, self.budget)?;
                self.inserted = self.inserted.checked_add(1).ok_or_else(damaged)?;
            }
        }
        if self.updates.peek().is_some_and(|(next, _)| **next == key) {
            let (_, update) = self.updates.next().expect("peeked update");
            match update {
                FreeSpaceUpdate::Available(entry) => self.writer.push(*entry, self.budget)?,
                FreeSpaceUpdate::Exhausted => {
                    self.removed = self.removed.checked_add(1).ok_or_else(damaged)?;
                }
            }
        } else {
            self.writer.push(existing, self.budget)?;
        }
        Ok(())
    }

    fn finish_updates(&mut self) -> Result<(), RecordAppendError> {
        for (_, update) in self.updates.by_ref() {
            if let FreeSpaceUpdate::Available(entry) = update {
                self.writer.push(*entry, self.budget)?;
                self.inserted = self.inserted.checked_add(1).ok_or_else(damaged)?;
            }
        }
        Ok(())
    }
}
