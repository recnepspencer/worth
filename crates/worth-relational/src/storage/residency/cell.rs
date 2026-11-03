mod demand;
#[path = "cell/member_export.rs"]
mod member_export;
#[path = "cell/observation.rs"]
mod observation;
mod resident_full;
mod source_retirement;
mod state;
use state::RegionState;
#[path = "cell/resident_head.rs"]
mod resident_head;
#[path = "cell/resident_leases.rs"]
mod resident_leases;
mod weak;
pub(super) use weak::RegionCellWeak;

use super::cell_load_cache::{self, RegionLoadCache};
use super::{
    region_load, Full, Head, RelationalResidencyDemandFailure as Failure, RelationalResidencyLease,
};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::PartitionId;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
pub(crate) struct RelationalColdRegion {
    pub(crate) kind_authority: super::RelationalRegionKindCommitment,
    pub(crate) source: Option<Arc<dyn RelationalDurableColdSource>>,
    pub(crate) partition_id: PartitionId,
    pub(crate) head: RelationalDurableMemberLocator,
    pub(crate) history: RelationalDurableMemberLocator,
    pub(crate) history_members: Arc<crate::durability::data::RelationalHistoryMemberDirectory>,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) schema_carrier_digest: [u8; 32],
    pub(crate) symbols: crate::runtime::RuntimeSymbolTable,
    pub(crate) content_digest: [u8; 32],
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) unavailable_records: usize,
}

#[derive(Debug)]
pub(crate) struct ResidentRegion {
    pub(super) partition: PartitionState,
    pub(super) _history_custody: Option<Arc<crate::storage::substrate::RecordHistoryCustody>>,
    pub(super) commitment: Option<PartitionContentCommitment>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug, Clone, Copy)]
enum ResidencyDemand {
    Head,
    Full,
}

#[derive(Debug)]
enum ResidencyState {
    Cold,
    Head { region: Arc<ResidentRegion> },
    Full(Arc<ResidentRegion>),
    Damaged { slot_counts: (usize, usize) },
}

#[derive(Debug)]
pub(crate) struct RelationalRegionResidency {
    pub(super) partition_id: PartitionId,
    pub(super) checkpoint_bindings:
        Mutex<worth_execution::ExecutionOrderedMap<[u8; 32], super::RecordedRegionCheckpoint>>,
    state: Mutex<RegionState>,
    damage: OnceLock<RelationalStorageUnavailable>,
    backing: Option<RelationalColdRegion>,
    head_readmission: Option<super::resident_history::ResidentHistoryReadmission>,
    raw_head: RegionLoadCache,
    raw_full: RegionLoadCache,
    pub(super) leases: AtomicUsize,
    _control_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl RelationalRegionResidency {
    pub(crate) fn cold(
        mut cold: RelationalColdRegion,
    ) -> Result<Arc<Self>, RelationalStorageUnavailable> {
        let mut members = cold.history_members.members(cold.history);
        members.insert(cold.head);
        let custody = cold
            .source
            .as_ref()
            .ok_or(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(cold.partition_id),
                cause: Cause::SourceUnavailable,
            })?
            .retain_members(&members.into_iter().collect::<Vec<_>>())
            .map_err(|cause| RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(cold.partition_id),
                cause,
            })?;
        let physical = crate::durability::medium::NativeSourceMemberState::new(
            cold.source.take().expect("same original producer source"),
            custody,
        );
        Ok(Arc::new(Self {
            partition_id: cold.partition_id,
            checkpoint_bindings: Mutex::new({
                let mut bindings = worth_execution::ExecutionOrderedMap::default();
                bindings
                    .insert_cloned_native_in(
                        &cold.schema_carrier_digest,
                        &super::RecordedRegionCheckpoint {
                            history_members: cold.history_members.clone(),
                            leaf: crate::durability::data::RelationalCheckpointRegion {
                                kind_authority: cold.kind_authority,
                                partition_id: cold.partition_id,
                                content_digest: cold.content_digest,
                                entity_slots: cold.entity_slots,
                                relation_slots: cold.relation_slots,
                                unavailable_records: cold.unavailable_records,
                                head: cold.head,
                                history: cold.history,
                            },
                        },
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("ordinary region binding uses unbounded system allocation");
                bindings
            }),
            backing: Some(cold),
            head_readmission: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(RegionState::new(ResidencyState::Cold, Some(physical))),
            damage: OnceLock::new(),
            leases: AtomicUsize::new(0),
            _control_memory: None,
        }))
    }

    pub(super) fn resident(
        partition: PartitionState,
        commitment: PartitionContentCommitment,
    ) -> RelationalResidencyLease<Full> {
        Self::resident_with_commitment(partition, Some(commitment))
    }

    pub(super) fn resident_uncommitted(
        partition: PartitionState,
    ) -> RelationalResidencyLease<Full> {
        Self::resident_with_commitment(partition, None)
    }

    fn resident_with_commitment(
        partition: PartitionState,
        commitment: Option<PartitionContentCommitment>,
    ) -> RelationalResidencyLease<Full> {
        let region = Arc::new(ResidentRegion {
            partition,
            commitment,
            _history_custody: None,
            _memory: None,
        });
        let cell = Arc::new(Self {
            partition_id: region.partition.partition_id,
            checkpoint_bindings: Mutex::new(Default::default()),
            backing: None,
            head_readmission: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(RegionState::new(ResidencyState::Full(region.clone()), None)),
            damage: OnceLock::new(),
            leases: AtomicUsize::new(1),
            _control_memory: None,
        });
        RelationalResidencyLease::accepted(cell, region)
    }

    /// Explicit I/O boundary. A single cell lock covers each load and its
    /// publication, so concurrent demands share one read and one refusal.
    pub(crate) fn demand_head(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
        authority: Option<&super::RelationalRegionKindAuthority>,
    ) -> Result<RelationalResidencyLease<Head>, Failure> {
        let region = self.demand(execution, ResidencyDemand::Head, limit, authority)?;
        Ok(RelationalResidencyLease::accepted(self.clone(), region))
    }

    pub(crate) fn demand_full(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
        authority: Option<&super::RelationalRegionKindAuthority>,
    ) -> Result<RelationalResidencyLease<Full>, Failure> {
        let region = self.demand(execution, ResidencyDemand::Full, limit, authority)?;
        Ok(RelationalResidencyLease::accepted(self.clone(), region))
    }

    pub(super) fn content_symbols(
        &self,
        fallback: &crate::symbols::data::StringInterner,
    ) -> crate::symbols::data::StringInterner {
        if let Some(cold) = &self.backing {
            cold.symbols.interner_snapshot()
        } else if let Some(readmission) = &self.head_readmission {
            readmission.symbols().interner_snapshot()
        } else {
            fallback.clone()
        }
    }

    /// The same terminal refusal applies to fresh guards even when their
    /// retention already holds an older, valid immutable lease.
    pub(crate) fn check_damage(&self) -> Result<(), RelationalStorageUnavailable> {
        self.damage.get().copied().map_or(Ok(()), Err)
    }

    fn failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
