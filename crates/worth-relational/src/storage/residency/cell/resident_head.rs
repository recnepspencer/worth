//! Owned Head construction preserves omitted-history readmission authority.
use super::*;

impl RelationalRegionResidency {
    pub(crate) fn current_readmission(&self) -> Option<super::super::ResidentHistoryReadmission> {
        self.head_readmission.clone().or_else(|| {
            self.backing.as_ref().map(|cold| {
                super::super::ResidentHistoryReadmission::new(cold.schema.clone(), &cold.symbols)
            })
        })
    }
    pub(in crate::storage::residency) fn resident_head(
        partition: PartitionState,
        commitment: PartitionContentCommitment,
        kind_indexes: Arc<super::super::kind_incremental::RegionKindIndexes>,
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> RelationalResidencyLease<Head> {
        let readmission =
            super::super::resident_history::ResidentHistoryReadmission::new(schema, symbols);
        Self::owned_head(partition, Some(commitment), Some(kind_indexes), readmission)
    }

    pub(in super::super) fn resident_current(
        partition: PartitionState,
        readmission: super::super::resident_history::ResidentHistoryReadmission,
    ) -> RelationalResidencyLease<Head> {
        Self::owned_head(partition, None, None, readmission)
    }

    fn owned_head(
        partition: PartitionState,
        commitment: Option<PartitionContentCommitment>,
        kind_indexes: Option<Arc<super::super::kind_incremental::RegionKindIndexes>>,
        readmission: super::super::resident_history::ResidentHistoryReadmission,
    ) -> RelationalResidencyLease<Head> {
        let region = Arc::new(ResidentRegion {
            partition,
            commitment,
            kind_indexes,
        });
        let cell = Arc::new(Self {
            partition_id: region.partition.partition_id,
            checkpoint_bindings: Mutex::new(Default::default()),
            backing: None,
            head_readmission: Some(readmission),
            _backing_custody: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(ResidencyState::Head {
                region: region.clone(),
            }),
            damage: OnceLock::new(),
            leases: AtomicUsize::new(1),
        });
        RelationalResidencyLease::accepted(cell, region)
    }
}
