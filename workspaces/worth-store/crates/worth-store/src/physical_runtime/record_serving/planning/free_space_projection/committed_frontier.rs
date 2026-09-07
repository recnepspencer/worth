use worth_store_physical_format::{CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader};

use super::InlineSegmentAllocation;

/// Durable allocation history, never the live allocator's outstanding reservations.
/// Only this publication's settled materialization can advance the prior root's frontier.
pub(super) struct CommittedAllocationFrontier {
    pub(super) next_segment: u64,
    pub(super) next_page: u64,
    pub(super) next_extent: u64,
}

impl CommittedAllocationFrontier {
    pub(super) fn from_publication(
        current: &DurableFreeSpaceManifestHeader,
        allocations: &[InlineSegmentAllocation],
        placements: impl Iterator<Item = CurrentPhysicalRecordPlacement>,
    ) -> Option<Self> {
        let mut next = Self {
            next_segment: current.next_segment(),
            next_page: current.next_page(),
            next_extent: current.next_extent(),
        };
        for allocation in allocations {
            advance(
                &mut next.next_segment,
                allocation.segment().segment_id().get(),
            )?;
        }
        for placement in placements {
            match placement {
                CurrentPhysicalRecordPlacement::Inline(inline) => {
                    advance(&mut next.next_page, inline.page().get())?;
                }
                CurrentPhysicalRecordPlacement::Extent(extent) => {
                    advance(&mut next.next_extent, extent.extent().get())?;
                }
            }
        }
        Some(next)
    }
}

fn advance(next: &mut u64, materialized: u64) -> Option<()> {
    *next = (*next).max(materialized.checked_add(1)?);
    Some(())
}

#[cfg(test)]
mod tests;
