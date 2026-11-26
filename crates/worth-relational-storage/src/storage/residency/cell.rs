pub mod cold_chunks;
pub mod demand;
#[path = "cell/eviction.rs"]
pub mod eviction;
#[path = "cell/reconstruction_head.rs"]
pub mod reconstruction_head;
#[path = "cell/working_set_charge.rs"]
mod working_set_charge;
pub use reconstruction_head::ReconstructedRegionHead;
#[path = "cell/member_export.rs"]
pub mod member_export;
#[path = "cell/observation.rs"]
pub mod observation;
pub mod resident_full;
pub mod source_retirement;
pub mod state;
use state::RegionState;
#[path = "cell/resident_head.rs"]
pub mod resident_head;
#[path = "cell/resident_leases.rs"]
pub mod resident_leases;
pub mod selected_readmission;
mod shared_region;
pub(super) use shared_region::SharedResidentRegion;
pub mod weak;
pub(super) use weak::RegionCellWeak;

use super::cell_load_cache::{self, RegionLoadCache};
use super::{Full, Head, RelationalResidencyDemandFailure as Failure, RelationalResidencyLease};
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::PartitionId;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::residency::RelationalRegionSchema;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
pub struct RelationalColdRegion {
    pub readmission: &'static super::cold_readmission::ColdRegionReadmission,
    pub kind_authority: super::RelationalRegionKindCommitment,
    pub source: Option<Arc<dyn RelationalDurableColdSource>>,
    pub partition_id: PartitionId,
    pub head: RelationalDurableMemberLocator,
    pub history: RelationalDurableMemberLocator,
    pub history_members:
        Arc<crate::storage::data::history_member_directory::RelationalHistoryMemberDirectory>,
    pub schema: Arc<dyn RelationalRegionSchema>,
    pub schema_carrier_digest: [u8; 32],
    pub symbols: crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub content_digest: [u8; 32],
    pub directory: std::sync::Arc<super::PortableRecordDirectory>,
    pub live_counts: crate::storage::residency::LiveRecordCounts,
    pub entity_slots: usize,
    pub relation_slots: usize,
    pub unavailable_records: usize,
    pub chunks: Arc<super::chunked::region_placement::RegionChunkPlacement>,
}

impl RelationalColdRegion {
    /// The recorded leaf this cold region readmits.
    pub fn leaf(&self) -> crate::storage::data::checkpoint_region::RelationalCheckpointRegion {
        crate::storage::data::checkpoint_region::RelationalCheckpointRegion {
            kind_authority: self.kind_authority,
            partition_id: self.partition_id,
            content_digest: self.content_digest,
            directory: self.directory.clone(),
            live_counts: self.live_counts,
            entity_slots: self.entity_slots,
            relation_slots: self.relation_slots,
            unavailable_records: self.unavailable_records,
            head: self.head,
            history: self.history,
            chunks: self.chunks.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ResidentRegion {
    pub symbol_requirements: OnceLock<Arc<super::required_symbols::CapturedSymbolRequirements>>,
    pub history_presence: super::history_presence::HistoryPresence,
    pub partition: PartitionState,
    /// Exclusive upper bounds on carrier and foundational `birth_seq` values
    /// reachable from this immutable `partition` (captured at region sealing).
    pub sealed_watermarks: crate::storage::substrate::SealedAllocationWatermarks,
    pub _history_custody: Option<Arc<crate::storage::substrate::RecordHistoryCustody>>,
    pub commitment: Option<PartitionContentCommitment>,
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub kind_indexes: Option<Arc<super::kind_incremental::RegionKindIndexes>>,
    /// Working-set credit for this exact allocation, returned on its last drop.
    pub working_set: OnceLock<super::working_set::WorkingSetCharge>,
}

#[derive(Debug, Clone, Copy)]
enum ResidencyDemand {
    Head,
    Full,
}

#[derive(Debug)]
enum ResidencyState {
    Cold,
    Head { region: SharedResidentRegion },
    Full(SharedResidentRegion),
    Damaged { slot_counts: (usize, usize) },
}

#[derive(Debug)]
pub struct RelationalRegionResidency {
    pub(super) partition_id: PartitionId,
    pub checkpoint_bindings:
        Mutex<worth_execution::ExecutionOrderedMap<[u8; 32], super::RecordedRegionCheckpoint>>,
    pub(in crate::internal_port::storage::residency) symbol_aliases:
        OnceLock<super::symbol_admission_identity::SymbolAdmissionAliases>,
    pub(in crate::internal_port::storage::residency) symbol_inheritance:
        OnceLock<Arc<super::symbol_inheritance::SymbolAdmissionInheritance>>,
    state: Mutex<RegionState>,
    damage: OnceLock<RelationalStorageUnavailable>,
    backing: Option<RelationalColdRegion>,
    head_readmission: Option<super::resident_history::ResidentHistoryReadmission>,
    raw_head: RegionLoadCache,
    raw_full: RegionLoadCache,
    pub leases: AtomicUsize,
    working_set: OnceLock<eviction::WorkingSetBinding>,
    _control_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl RelationalRegionResidency {
    pub fn cold(mut cold: RelationalColdRegion) -> Result<Arc<Self>, RelationalStorageUnavailable> {
        let mut members = cold.history_members.members(cold.history);
        members.insert(cold.head);
        members.extend(cold.chunks.members.iter().copied());
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
                            leaf: cold.leaf(),
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
            symbol_inheritance: OnceLock::new(),
            symbol_aliases: OnceLock::new(),
            leases: AtomicUsize::new(0),
            working_set: OnceLock::new(),
            _control_memory: None,
        }))
    }

    pub(super) fn resident(
        partition: PartitionState,
        commitment: PartitionContentCommitment,
        kind_indexes: Arc<super::kind_incremental::RegionKindIndexes>,
        requirements: Option<super::required_symbols::CapturedSymbolRequirements>,
    ) -> RelationalResidencyLease<Full> {
        Self::resident_with_commitment(
            partition,
            Some(commitment),
            Some(kind_indexes),
            requirements,
        )
    }

    pub fn resident_uncommitted(partition: PartitionState) -> RelationalResidencyLease<Full> {
        Self::resident_with_commitment(partition, None, None, None)
    }

    fn resident_with_commitment(
        partition: PartitionState,
        commitment: Option<PartitionContentCommitment>,
        kind_indexes: Option<Arc<super::kind_incremental::RegionKindIndexes>>,
        requirements: Option<super::required_symbols::CapturedSymbolRequirements>,
    ) -> RelationalResidencyLease<Full> {
        let region = Arc::new(ResidentRegion {
            symbol_requirements: requirements
                .map(|value| OnceLock::from(Arc::new(value)))
                .unwrap_or_default(),
            history_presence: super::history_presence::HistoryPresence::complete(),
            partition,
            sealed_watermarks: crate::storage::substrate::SealedAllocationWatermarks::capture(),
            commitment,
            _history_custody: None,
            working_set: OnceLock::new(),
            _memory: None,
            kind_indexes,
        });
        let cell = Arc::new(Self {
            partition_id: region.partition.partition_id,
            checkpoint_bindings: Mutex::new(Default::default()),
            backing: None,
            head_readmission: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(RegionState::new(
                ResidencyState::Full(SharedResidentRegion::from_arc(region.clone())),
                None,
            )),
            damage: OnceLock::new(),
            symbol_inheritance: OnceLock::new(),
            symbol_aliases: OnceLock::new(),
            leases: AtomicUsize::new(1),
            working_set: OnceLock::new(),
            _control_memory: None,
        });
        RelationalResidencyLease::accepted(cell, region)
    }

    /// Explicit I/O boundary. A single cell lock covers each load and its
    /// publication, so concurrent demands share one read and one refusal.
    pub fn demand_head(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
        authority: Option<&super::RelationalRegionKindAuthority>,
    ) -> Result<RelationalResidencyLease<Head>, Failure> {
        let region = self.demand(execution, ResidencyDemand::Head, limit, authority)?;
        Ok(RelationalResidencyLease::accepted(self.clone(), region))
    }

    pub fn demand_full(
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
    pub fn check_damage(&self) -> Result<(), RelationalStorageUnavailable> {
        self.damage.get().copied().map_or(Ok(()), Err)
    }

    fn failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
