//! Persistent replacement of only the regions named by publication.
use super::*;
impl RelationalPersistentRegionSet {
    pub(crate) fn empty(set_id: u64) -> Arc<Self> {
        Arc::new(Self {
            set_id,
            index_root: None,
            count: 0,
            live_counts: Default::default(),
            kinds: Default::default(),
            materialization_unavailable_records: 0,
            _memory: None,
        })
    }
    pub(crate) fn initial(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        set_id: u64,
        regions: BTreeMap<PartitionId, Arc<RelationalRootRegion>>,
        issuer: &RelationalBranchRootIdentityIssuer,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let count = regions.len();
        let live_counts = regions
            .values()
            .try_fold(
                crate::storage::residency::LiveRecordCounts::default(),
                |sum, region| sum.checked_add(region.live_counts),
            )
            .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
        let materialization_unavailable_records = regions
            .values()
            .map(|region| region.materialization_unavailable_records())
            .sum();
        let mut kinds = crate::storage::residency::KindRegionDirectory::default();
        let mut index_root = None;
        for (partition_id, region) in regions {
            kinds.replaced(partition_id, None, Some(&region.directory));
            index_root = Some(insert_leaf(
                index_root.as_ref(),
                partition_id,
                RelationalPersistentRegionLeaf::Present(Box::new(RelationalRootRegionReference {
                    region,
                    _memory: None,
                })),
                0,
                issuer,
                None,
            )?);
        }
        Ok(Arc::new(Self {
            set_id,
            index_root,
            count,
            live_counts,
            kinds,
            materialization_unavailable_records,
            _memory: None,
        }))
    }

    pub(crate) fn replace(
        set_id: u64,
        parent: &Arc<Self>,
        replacements: BTreeMap<PartitionId, Arc<RelationalRootRegion>>,
        removed: BTreeSet<PartitionId>,
        issuer: &RelationalBranchRootIdentityIssuer,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let mut index_root = parent.index_root.clone();
        let mut kinds = parent.kinds.clone();
        let mut count = parent.count;
        let mut live_counts = parent.live_counts;
        let mut materialization_unavailable_records = parent.materialization_unavailable_records;
        for (partition_id, region) in replacements {
            kinds.replaced(
                partition_id,
                get_region(index_root.as_ref(), partition_id)
                    .map(|region| region.directory.as_ref()),
                Some(&region.directory),
            );
            if let Some(previous) = get_region(index_root.as_ref(), partition_id) {
                live_counts = live_counts
                    .checked_sub(previous.live_counts)
                    .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
                materialization_unavailable_records = materialization_unavailable_records
                    .saturating_sub(previous.materialization_unavailable_records());
            } else {
                count = count.saturating_add(1);
            }
            live_counts = live_counts
                .checked_add(region.live_counts)
                .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
            materialization_unavailable_records = materialization_unavailable_records
                .saturating_add(region.materialization_unavailable_records());
            index_root = Some(insert_leaf(
                index_root.as_ref(),
                partition_id,
                RelationalPersistentRegionLeaf::Present(Box::new(RelationalRootRegionReference {
                    region,
                    _memory: None,
                })),
                0,
                issuer,
                None,
            )?);
        }
        for partition_id in removed {
            kinds.replaced(
                partition_id,
                get_region(index_root.as_ref(), partition_id)
                    .map(|region| region.directory.as_ref()),
                None,
            );
            if let Some(previous) = get_region(index_root.as_ref(), partition_id) {
                live_counts = live_counts
                    .checked_sub(previous.live_counts)
                    .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
                count = count.saturating_sub(1);
                materialization_unavailable_records = materialization_unavailable_records
                    .saturating_sub(previous.materialization_unavailable_records());
            }
            index_root = Some(insert_leaf(
                index_root.as_ref(),
                partition_id,
                RelationalPersistentRegionLeaf::Removed(Box::new(RelationalRemovedPartition {
                    partition_id,
                    _memory: None,
                })),
                0,
                issuer,
                None,
            )?);
        }
        Ok(Arc::new(Self {
            set_id,
            index_root,
            count,
            live_counts,
            kinds,
            materialization_unavailable_records,
            _memory: None,
        }))
    }
}
