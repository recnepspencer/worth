use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, PhysicalExtentId, PhysicalPageId, PhysicalSegmentId,
};

#[derive(Clone)]
pub(in crate::physical_runtime) struct RecordAllocationFrontier {
    next_segment: u64,
    next_page: u64,
    next_extent: u64,
    segment_limit: u64,
    page_limit: u64,
    extent_limit: u64,
}

impl RecordAllocationFrontier {
    pub(in crate::physical_runtime::record_serving) fn new(
        free_space: &DurableFreeSpaceManifestHeader,
    ) -> Self {
        Self {
            next_segment: free_space.next_segment(),
            next_page: free_space.next_page(),
            next_extent: free_space.next_extent(),
            segment_limit: u64::MAX,
            page_limit: u64::MAX,
            extent_limit: u64::MAX,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn reserve(
        &mut self,
        segments: u64,
        pages: u64,
        extents: u64,
    ) -> Option<Self> {
        let segment_limit = reserve_limit(self.next_segment, segments, self.segment_limit)?;
        let page_limit = reserve_limit(self.next_page, pages, self.page_limit)?;
        let extent_limit = reserve_limit(self.next_extent, extents, self.extent_limit)?;
        let reservation = Self {
            next_segment: self.next_segment,
            next_page: self.next_page,
            next_extent: self.next_extent,
            segment_limit,
            page_limit,
            extent_limit,
        };
        self.next_segment = segment_limit;
        self.next_page = page_limit;
        self.next_extent = extent_limit;
        Some(reservation)
    }

    pub(in crate::physical_runtime::record_serving) fn allocate_segment(
        &mut self,
    ) -> Option<PhysicalSegmentId> {
        if self.next_segment >= self.segment_limit {
            return None;
        }
        let value = PhysicalSegmentId::from_raw(self.next_segment).ok()?;
        self.next_segment = self.next_segment.checked_add(1)?;
        Some(value)
    }

    pub(in crate::physical_runtime::record_serving) fn allocate_page(
        &mut self,
    ) -> Option<PhysicalPageId> {
        if self.next_page >= self.page_limit {
            return None;
        }
        let value = PhysicalPageId::from_raw(self.next_page).ok()?;
        self.next_page = self.next_page.checked_add(1)?;
        Some(value)
    }

    pub(in crate::physical_runtime::record_serving) fn allocate_extent(
        &mut self,
    ) -> Option<PhysicalExtentId> {
        if self.next_extent >= self.extent_limit {
            return None;
        }
        let value = PhysicalExtentId::from_raw(self.next_extent).ok()?;
        self.next_extent = self.next_extent.checked_add(1)?;
        Some(value)
    }

    #[cfg(test)]
    pub(in crate::physical_runtime::record_serving) const fn next_segment(&self) -> u64 {
        self.next_segment
    }
    #[cfg(test)]
    pub(in crate::physical_runtime::record_serving) const fn next_page(&self) -> u64 {
        self.next_page
    }
    #[cfg(test)]
    pub(in crate::physical_runtime::record_serving) const fn next_extent(&self) -> u64 {
        self.next_extent
    }
}

fn reserve_limit(next: u64, count: u64, limit: u64) -> Option<u64> {
    let reserved = next.checked_add(count)?;
    (reserved <= limit).then_some(reserved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_identity_value_is_reserved_and_never_advertised_as_allocatable() {
        let free = DurableFreeSpaceManifestHeader::new(
            1,
            1,
            2,
            4,
            0,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            1,
            None,
        )
        .unwrap();
        let mut frontier = RecordAllocationFrontier::new(&free);
        assert_eq!(frontier.allocate_segment(), None);
        assert_eq!(frontier.allocate_page(), None);
        assert_eq!(frontier.allocate_extent(), None);
    }

    #[test]
    fn disjoint_reservations_never_reuse_abandoned_identity_ranges() {
        let free = DurableFreeSpaceManifestHeader::new(1, 1, 2, 4, 0, 7, 11, 13, 1, None).unwrap();
        let mut frontier = RecordAllocationFrontier::new(&free);
        let mut first = frontier.reserve(2, 3, 1).unwrap();
        let mut second = frontier.reserve(1, 1, 2).unwrap();
        assert_eq!(first.allocate_segment().unwrap().get(), 7);
        assert_eq!(first.allocate_segment().unwrap().get(), 8);
        assert_eq!(first.allocate_segment(), None);
        assert_eq!(second.allocate_segment().unwrap().get(), 9);
        assert_eq!(second.allocate_page().unwrap().get(), 14);
        assert_eq!(second.allocate_extent().unwrap().get(), 14);
        assert_eq!(second.allocate_extent().unwrap().get(), 15);
        assert_eq!(frontier.next_segment(), 10);
        assert_eq!(frontier.next_page(), 15);
        assert_eq!(frontier.next_extent(), 16);
    }
}
