use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ManifestBlockReference, PersistedRecordIdentity,
    PhysicalRootRoutingBlock,
};

use super::{ManifestDiscoveryCounterSnapshot, ManifestLookupFailure, ManifestReader};

pub(in crate::physical_runtime::record_serving) struct ManifestRangeCursor<'reader> {
    reader: ManifestReader<'reader>,
    parents: Vec<ParentPosition>,
    leaf: Vec<CurrentPhysicalRecordPlacement>,
    next_index: usize,
    initialized: bool,
    counters: ManifestDiscoveryCounterSnapshot,
}

struct ParentPosition {
    reference: ManifestBlockReference,
    next_child: usize,
}

impl<'reader> ManifestRangeCursor<'reader> {
    pub(in crate::physical_runtime::record_serving) fn new(
        reader: ManifestReader<'reader>,
    ) -> Self {
        Self {
            reader,
            parents: Vec::new(),
            leaf: Vec::new(),
            next_index: 0,
            initialized: false,
            counters: ManifestDiscoveryCounterSnapshot::default(),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn seek(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        root: Option<ManifestBlockReference>,
        first: Option<PersistedRecordIdentity>,
    ) -> Result<bool, ManifestLookupFailure> {
        self.parents.clear();
        self.leaf.clear();
        self.next_index = 0;
        self.initialized = true;
        let Some(mut reference) = root else {
            return Ok(first.is_none());
        };
        if let Some(first) = first
            && (first < reference.first() || first > reference.last())
        {
            return Ok(false);
        }
        loop {
            match self
                .reader
                .read_block(allocation, reference, &mut self.counters)?
            {
                PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                    self.next_index = first.map_or(0, |first| {
                        let (index, comparisons) =
                            super::super::counted_search::partition_point(&entries, |entry| {
                                entry.record() < first
                            });
                        self.counters.observe_comparisons(comparisons);
                        index
                    });
                    self.leaf = entries;
                    return Ok(self.next_index < self.leaf.len());
                }
                PhysicalRootRoutingBlock::Branch { children, .. } => {
                    let index = first.map_or(0, |first| {
                        let (index, comparisons) =
                            super::super::counted_search::partition_point(&children, |child| {
                                child.last() < first
                            });
                        self.counters.observe_comparisons(comparisons);
                        index
                    });
                    let Some(child) = children.get(index).copied() else {
                        return Ok(false);
                    };
                    self.parents.push(ParentPosition {
                        reference,
                        next_child: index + 1,
                    });
                    reference = child;
                }
            }
        }
    }

    pub(in crate::physical_runtime::record_serving) fn next(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<Option<CurrentPhysicalRecordPlacement>, ManifestLookupFailure> {
        if !self.initialized {
            return Err(ManifestLookupFailure::Damaged);
        }
        if let Some(value) = self.leaf.get(self.next_index).copied() {
            self.next_index += 1;
            return Ok(Some(value));
        }
        if !self.advance_leaf(allocation)? {
            return Ok(None);
        }
        self.next(allocation)
    }

    pub(in crate::physical_runtime::record_serving) const fn counters(
        &self,
    ) -> ManifestDiscoveryCounterSnapshot {
        self.counters
    }

    fn advance_leaf(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<bool, ManifestLookupFailure> {
        while let Some(mut parent) = self.parents.pop() {
            let block = self
                .reader
                .read_block(allocation, parent.reference, &mut self.counters)?;
            let children = block.children().ok_or(ManifestLookupFailure::Damaged)?;
            let Some(next) = children.get(parent.next_child).copied() else {
                continue;
            };
            parent.next_child += 1;
            self.parents.push(parent);
            self.descend_left(allocation, next)?;
            return Ok(true);
        }
        self.leaf.clear();
        self.next_index = 0;
        Ok(false)
    }

    fn descend_left(
        &mut self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        mut reference: ManifestBlockReference,
    ) -> Result<(), ManifestLookupFailure> {
        loop {
            match self
                .reader
                .read_block(allocation, reference, &mut self.counters)?
            {
                PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                    self.leaf = entries;
                    self.next_index = 0;
                    return Ok(());
                }
                PhysicalRootRoutingBlock::Branch { children, .. } => {
                    let first = children
                        .first()
                        .copied()
                        .ok_or(ManifestLookupFailure::Damaged)?;
                    self.parents.push(ParentPosition {
                        reference,
                        next_child: 1,
                    });
                    reference = first;
                }
            }
        }
    }
}
