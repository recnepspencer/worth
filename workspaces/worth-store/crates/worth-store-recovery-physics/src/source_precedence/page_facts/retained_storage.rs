use super::*;

fn vector_bytes<T>(capacity: usize) -> Option<u64> {
    u64::try_from(capacity)
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

impl SelectedPhysicalPageFacts {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let blocks = vector_bytes::<ManifestBlockReference>(self.routing_blocks.capacity())?;
        let topology = vector_bytes::<(ManifestBlockReference, PhysicalRootRoutingBlock)>(
            self.routing_topology.capacity(),
        )?;
        let placements =
            vector_bytes::<CurrentPhysicalRecordPlacement>(self.placements.capacity())?;
        self.routing_topology.iter().try_fold(
            blocks.checked_add(topology)?.checked_add(placements)?,
            |sum, (_, block)| sum.checked_add(block.owned_heap_bytes()?),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_but_reserved_route_arrays_still_consume_heap() {
        let facts = SelectedPhysicalPageFacts {
            root_generation: 1,
            manifest_block_count: 0,
            distinct_pages_and_extents: 0,
            routing_blocks: Vec::with_capacity(3),
            routing_topology: Vec::with_capacity(2),
            placements: Vec::with_capacity(5),
        };
        let expected = facts.routing_blocks.capacity()
            * std::mem::size_of::<ManifestBlockReference>()
            + facts.routing_topology.capacity()
                * std::mem::size_of::<(ManifestBlockReference, PhysicalRootRoutingBlock)>()
            + facts.placements.capacity() * std::mem::size_of::<CurrentPhysicalRecordPlacement>();
        assert_eq!(facts.owned_heap_bytes(), Some(expected as u64));
        assert!(expected > 0);
    }
}
