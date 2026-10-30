//! Exact physical union without per-root logical or excluded-lane inventories.
use super::*;
use crate::branch::{
    RelationalRootAllocationVisitor, RelationalRootAuthoritativeAllocationObservation,
};
use crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial as Denial;
use crate::storage::{data::RelationalStorageUnavailable, overlay::PartitionState};

impl RelationalAuthoritativeAllocationAccounting {
    pub(in crate::inspection::mvcc) fn observe_census_root(
        &mut self,
        runtime_instance_id: u64,
        root: &RelationalBranchRoot,
    ) -> Result<(), Denial> {
        let start = std::time::Instant::now();
        self.census_work.as_mut().unwrap().roots += 1;
        let result = root.try_visit_authoritative_allocation_union(&mut CensusRootWalk {
            accounting: self,
            runtime_instance_id,
        });
        self.census_work.as_mut().unwrap().root_union_us += start.elapsed().as_micros();
        result
    }
}

struct CensusRootWalk<'a> {
    accounting: &'a mut RelationalAuthoritativeAllocationAccounting,
    runtime_instance_id: u64,
}

impl RelationalRootAllocationVisitor for CensusRootWalk<'_> {
    type Error = Denial;

    fn allocation(&mut self, allocation: RelationalRootAuthoritativeAllocationObservation) -> bool {
        let work = self.accounting.census_work.as_mut().unwrap();
        work.root_allocation_rows += 1;
        let kind = match allocation.kind {
            RelationalRootAuthoritativeAllocationKind::RootMetadata => {
                RelationalAuthoritativeAllocationKind::RootMetadata
            }
            RelationalRootAuthoritativeAllocationKind::SchemaAuthority => {
                work.schema_inventory_computations += 1;
                RelationalAuthoritativeAllocationKind::RootSchemaAuthority
            }
            RelationalRootAuthoritativeAllocationKind::PersistentRegionSetObject => {
                RelationalAuthoritativeAllocationKind::RootReachabilitySetObject
            }
            RelationalRootAuthoritativeAllocationKind::PersistentRegionMapNodeObject => {
                RelationalAuthoritativeAllocationKind::RootReachabilityStructure
            }
            RelationalRootAuthoritativeAllocationKind::PersistentRegionReplacementStorage => {
                RelationalAuthoritativeAllocationKind::RootReplacementStorage
            }
            RelationalRootAuthoritativeAllocationKind::PersistentRegionRemovalStorage => {
                RelationalAuthoritativeAllocationKind::RootRemovalStorage
            }
        };
        let locator = RelationalAuthoritativeAllocationLocator::new(
            self.runtime_instance_id,
            kind,
            allocation.owner_id,
            allocation.owner_id,
            None,
        );
        let unseen = !self.accounting.allocations.contains_key(&locator);
        self.accounting
            .insert(locator, allocation.authoritative_bytes);
        unseen
    }

    fn region(
        &mut self,
        region: crate::branch::RelationalRootRegionObservation,
        partition: Result<Option<&PartitionState>, RelationalStorageUnavailable>,
    ) -> Result<(), Denial> {
        let start = std::time::Instant::now();
        let partition = partition
            .map_err(Denial::StorageUnavailable)?
            .ok_or(Denial::RootUnavailable)?;
        self.accounting.census_work.as_mut().unwrap().region_entries += 1;
        for allocation in
            crate::inspection::mvcc::partition_allocations::authoritative_partition_allocations(
                self.runtime_instance_id,
                partition,
                &mut self.accounting.seen_payload,
                self.accounting.census_work.as_mut(),
            )
        {
            self.accounting
                .insert(allocation.locator(), allocation.authoritative_bytes());
        }
        for (kind, bytes) in [
            (
                RelationalAuthoritativeAllocationKind::RootRegionObject,
                region.root_region_bytes,
            ),
            (
                RelationalAuthoritativeAllocationKind::PartitionStateObject,
                region.partition_state_bytes,
            ),
        ] {
            self.accounting.insert(
                RelationalAuthoritativeAllocationLocator::new(
                    self.runtime_instance_id,
                    kind,
                    region.region_id,
                    region.creation_root_id,
                    Some(region.partition_id),
                ),
                bytes,
            );
        }
        self.accounting
            .census_work
            .as_mut()
            .unwrap()
            .partition_walk_us += start.elapsed().as_micros();
        Ok(())
    }
}
