//! A WAL target whose location the selected root does not route.

use worth_store_physical_format::DurableFreeSpaceManifestHeader;
use worth_store_recovery_physics::{PhysicalRedoTarget, PhysicalRedoTargetIdentity};

/// An inline page can carry several admitted images. Allocation truth judges
/// the first one; a historical classification names the last one. An extent
/// chunk has a single image.
#[derive(Debug, Clone, Copy)]
pub(super) struct AbsentTarget<'a> {
    first: &'a PhysicalRedoTarget,
    last: &'a PhysicalRedoTarget,
}

impl<'a> AbsentTarget<'a> {
    /// The images of one inline page in WAL order.
    pub(super) fn inline_page(images: Vec<&'a PhysicalRedoTarget>) -> Option<Self> {
        let first = *images.first()?;
        let last = images
            .into_iter()
            .max_by_key(|image| generation(image.identity()))?;
        Some(Self { first, last })
    }

    pub(super) const fn extent_chunk(target: &'a PhysicalRedoTarget) -> Self {
        Self {
            first: target,
            last: target,
        }
    }

    pub(super) const fn first(self) -> &'a PhysicalRedoTarget {
        self.first
    }

    pub(super) const fn last(self) -> &'a PhysicalRedoTarget {
        self.last
    }

    /// Whether the selected root already allocated this location. Such a
    /// target is not a fresh allocation above the selected frontier, so only
    /// a historical classification can admit it.
    pub(super) fn allocated_under(self, frontier: SelectedFrontier) -> bool {
        frontier.allocated(self.first.identity())
    }
}

/// The first page and extent the selected root has not allocated.
#[derive(Debug, Clone, Copy)]
pub(super) struct SelectedFrontier {
    pub(super) next_page: u64,
    pub(super) next_extent: u64,
}

impl SelectedFrontier {
    pub(super) fn of(free: &DurableFreeSpaceManifestHeader) -> Self {
        Self {
            next_page: free.next_page(),
            next_extent: free.next_extent(),
        }
    }

    const fn allocated(self, target: PhysicalRedoTargetIdentity) -> bool {
        match target {
            PhysicalRedoTargetIdentity::InlinePage { page, .. } => page < self.next_page,
            PhysicalRedoTargetIdentity::ExtentChunk { extent, .. } => extent < self.next_extent,
        }
    }
}

const fn generation(target: PhysicalRedoTargetIdentity) -> u64 {
    match target {
        PhysicalRedoTargetIdentity::InlinePage { generation, .. }
        | PhysicalRedoTargetIdentity::ExtentChunk { generation, .. } => generation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_location_below_the_selected_frontier_was_already_allocated() {
        let page = |page| PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page,
            generation: 1,
        };
        let extent = |extent| PhysicalRedoTargetIdentity::ExtentChunk {
            extent,
            generation: 1,
            chunk: 0,
        };
        let frontier = |next_page, next_extent| SelectedFrontier {
            next_page,
            next_extent,
        };
        assert!(frontier(27, 9).allocated(page(26)));
        assert!(
            !frontier(27, 9).allocated(page(27)),
            "the frontier page is fresh"
        );
        assert!(
            !frontier(9, 27).allocated(page(26)),
            "pages ignore the extent frontier"
        );
        assert!(frontier(27, 9).allocated(extent(8)));
        assert!(
            !frontier(27, 9).allocated(extent(9)),
            "the frontier extent is fresh"
        );
        assert!(
            !frontier(27, 9).allocated(extent(26)),
            "extents ignore the page frontier"
        );
    }
}
