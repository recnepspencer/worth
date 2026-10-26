//! Owned Head construction preserves omitted-history readmission authority.
use super::*;

impl RelationalRegionResidency {
    pub(crate) fn resident_head(
        partition: PartitionState,
        commitment: PartitionContentCommitment,
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> RelationalResidencyLease<Head> {
        let readmission =
            super::super::resident_history::ResidentHistoryReadmission::new(schema, symbols);
        let region = Arc::new(ResidentRegion {
            partition,
            commitment: Some(commitment),
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
