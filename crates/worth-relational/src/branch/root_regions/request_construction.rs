//! A stack-owned region directory acquires its actual backing before each growth.
use super::*;

pub(in crate::branch) struct PreparedRelationalRegionSet {
    set: RelationalPersistentRegionSet,
}
impl PreparedRelationalRegionSet {
    pub(in crate::branch) fn new(
        set_id: u64,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalBranchRootCaptureDenial> {
        let memory =
            super::super::root::reserve_arc_memory::<RelationalPersistentRegionSet>(Some(request))?;
        Ok(Self {
            set: RelationalPersistentRegionSet {
                set_id,
                index_root: None,
                count: 0,
                materialization_unavailable_records: 0,
                _memory: memory,
            },
        })
    }

    pub(in crate::branch) fn insert(
        &mut self,
        partition_id: PartitionId,
        region: Arc<RelationalRootRegion>,
        issuer: &RelationalBranchRootIdentityIssuer,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalBranchRootCaptureDenial> {
        if self.set.get(partition_id).is_some() {
            return Err(RelationalBranchRootCaptureDenial::DuplicateRegion(
                partition_id,
            ));
        }
        let count = self
            .set
            .count
            .checked_add(1)
            .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
        let unavailable = self
            .set
            .materialization_unavailable_records
            .checked_add(region.materialization_unavailable_records())
            .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
        let memory =
            super::super::root::reserve_box_memory::<RelationalRootRegionReference>(Some(request))?;
        let leaf =
            RelationalPersistentRegionLeaf::Present(Box::new(RelationalRootRegionReference {
                region,
                _memory: memory,
            }));
        self.set.index_root = Some(insert_leaf(
            self.set.index_root.as_ref(),
            partition_id,
            leaf,
            0,
            issuer,
            Some(request),
        )?);
        self.set.count = count;
        self.set.materialization_unavailable_records = unavailable;
        Ok(())
    }

    pub(in crate::branch) fn finish(self) -> Arc<RelationalPersistentRegionSet> {
        Arc::new(self.set)
    }
}

// An ordinary by-value copy owns a new ordinary header; actual shared nodes
// and their existing tickets stay in their original immutable allocations.
impl Clone for RelationalPersistentRegionSet {
    fn clone(&self) -> Self {
        Self {
            set_id: self.set_id,
            index_root: self.index_root.clone(),
            count: self.count,
            materialization_unavailable_records: self.materialization_unavailable_records,
            _memory: None,
        }
    }
}
