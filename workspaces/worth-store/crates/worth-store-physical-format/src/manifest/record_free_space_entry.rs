use crate::ExtentArenaRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RecordAllocationClass {
    InlinePage = 1,
    ExtentArena = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlinePageFreeFrontier {
    owner: u64,
    first_unallocated: u64,
    unallocated_count: u64,
}

impl InlinePageFreeFrontier {
    pub const fn owner(self) -> u64 {
        self.owner
    }
    pub const fn first_unallocated(self) -> u64 {
        self.first_unallocated
    }
    pub const fn unallocated_count(self) -> u64 {
        self.unallocated_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordFreeSpaceRegion {
    Inline(InlinePageFreeFrontier),
    Arena(ExtentArenaRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordFreeSpaceManifestEntry {
    region: RecordFreeSpaceRegion,
    generation: u64,
}

impl RecordFreeSpaceManifestEntry {
    pub fn inline_frontier(
        owner: u64,
        first_unallocated: u64,
        unallocated_count: u64,
        generation: u64,
    ) -> Option<Self> {
        if owner == 0
            || first_unallocated == 0
            || unallocated_count == 0
            || generation == 0
            || first_unallocated.checked_add(unallocated_count).is_none()
        {
            return None;
        }
        Some(Self {
            region: RecordFreeSpaceRegion::Inline(InlinePageFreeFrontier {
                owner,
                first_unallocated,
                unallocated_count,
            }),
            generation,
        })
    }
    pub fn arena_range(range: ExtentArenaRange, generation: u64) -> Option<Self> {
        (generation > 0).then_some(Self {
            region: RecordFreeSpaceRegion::Arena(range),
            generation,
        })
    }
    pub const fn region(self) -> RecordFreeSpaceRegion {
        self.region
    }
    pub const fn class(self) -> RecordAllocationClass {
        match self.region {
            RecordFreeSpaceRegion::Inline(_) => RecordAllocationClass::InlinePage,
            RecordFreeSpaceRegion::Arena(_) => RecordAllocationClass::ExtentArena,
        }
    }
    pub const fn owner(self) -> u64 {
        match self.region {
            RecordFreeSpaceRegion::Inline(value) => value.owner(),
            RecordFreeSpaceRegion::Arena(range) => range.arena().get(),
        }
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
    pub const fn arena_free_range(self) -> Option<ExtentArenaRange> {
        match self.region {
            RecordFreeSpaceRegion::Arena(range) => Some(range),
            _ => None,
        }
    }
    pub const fn inline_free_frontier(self) -> Option<InlinePageFreeFrontier> {
        match self.region {
            RecordFreeSpaceRegion::Inline(value) => Some(value),
            _ => None,
        }
    }
}
