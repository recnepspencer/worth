#[path = "cell/member_export.rs"]
mod member_export;
#[path = "cell/observation.rs"]
mod observation;
#[path = "cell/resident_head.rs"]
mod resident_head;
#[path = "cell/resident_leases.rs"]
mod resident_leases;

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
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
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
    pub(super) commitment: Option<PartitionContentCommitment>,
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
        Mutex<std::collections::BTreeMap<[u8; 32], super::RecordedRegionCheckpoint>>,
    state: Mutex<ResidencyState>,
    damage: OnceLock<RelationalStorageUnavailable>,
    backing: Option<RelationalColdRegion>,
    head_readmission: Option<super::resident_history::ResidentHistoryReadmission>,
    _backing_custody: Option<crate::durability::medium::RelationalDurableMemberCustody>,
    raw_head: RegionLoadCache,
    raw_full: RegionLoadCache,
    pub(super) leases: AtomicUsize,
}

impl RelationalRegionResidency {
    pub(crate) fn cold(
        cold: RelationalColdRegion,
    ) -> Result<Arc<Self>, RelationalStorageUnavailable> {
        let mut members = cold.history_members.members(cold.history);
        members.insert(cold.head);
        let custody = cold
            .source
            .retain_members(&members.into_iter().collect::<Vec<_>>())
            .map_err(|cause| RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(cold.partition_id),
                cause,
            })?;
        Ok(Arc::new(Self {
            partition_id: cold.partition_id,
            checkpoint_bindings: Mutex::new(std::collections::BTreeMap::from([(
                cold.schema_carrier_digest,
                super::RecordedRegionCheckpoint {
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
            )])),
            backing: Some(cold),
            head_readmission: None,
            _backing_custody: Some(custody),
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(ResidencyState::Cold),
            damage: OnceLock::new(),
            leases: AtomicUsize::new(0),
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
        });
        let cell = Arc::new(Self {
            partition_id: region.partition.partition_id,
            checkpoint_bindings: Mutex::new(Default::default()),
            backing: None,
            head_readmission: None,
            _backing_custody: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(ResidencyState::Full(region.clone())),
            damage: OnceLock::new(),
            leases: AtomicUsize::new(1),
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

    fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        demand: ResidencyDemand,
        limit: u64,
        authority: Option<&super::RelationalRegionKindAuthority>,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        if let Some(authority) = authority {
            // A foreign root's proof is a refused demand, not damage to this
            // cell's independently authenticated members.
            authority
                .require_cell(self)
                .map_err(|cause| self.failure(cause))?;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let loaded = (|| {
            self.check_damage()?;
            if let ResidencyState::Cold = &*state {
                let region = self.raw_head(execution, limit)?;
                *state = ResidencyState::Head { region };
            }
            if matches!(demand, ResidencyDemand::Full)
                && let ResidencyState::Head { region } = &*state
            {
                let region = self.raw_full(region, execution, limit)?;
                *state = ResidencyState::Full(region);
            }
            match &*state {
                ResidencyState::Head { region, .. } | ResidencyState::Full(region) => {
                    Ok(region.clone())
                }
                _ => unreachable!("load either publishes a region or refuses"),
            }
        })();
        match loaded {
            Ok(region) => {
                self.leases
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        count.checked_add(1)
                    })
                    .map_err(|_| self.failure(Cause::MemoryExhausted))?;
                Ok(region)
            }
            Err(Failure::StorageUnavailable(failure)) => {
                if failure.cause.is_proven_damage()
                    && failure.subject == RelationalStorageSubject::Region(self.partition_id)
                {
                    let slot_counts = match &*state {
                        ResidencyState::Head { region } | ResidencyState::Full(region) => (
                            region.partition.entity_arena.slot_count(),
                            region.partition.relation_arena.slot_count(),
                        ),
                        ResidencyState::Cold => self
                            .backing
                            .as_ref()
                            .map_or((0, 0), |cold| (cold.entity_slots, cold.relation_slots)),
                        ResidencyState::Damaged { slot_counts } => *slot_counts,
                    };
                    let _ = self.damage.set(failure);
                    *state = ResidencyState::Damaged { slot_counts };
                }
                Err(Failure::StorageUnavailable(failure))
            }
            Err(Failure::Request(cause)) => Err(Failure::Request(cause)),
        }
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

    // Raw readmission authenticates record content independently of derived
    // buckets. Keeping it separate breaks cross-partition adjacency cycles:
    // bucket verification never waits for another region's serving admission.
    pub(super) fn raw_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        cell_load_cache::load(&self.raw_head, self.partition_id, || {
            let cold = self
                .backing
                .as_ref()
                .ok_or_else(|| self.failure(Cause::SourceUnavailable))?;
            region_load::load_head(cold, execution, limit)
        })
    }

    pub(super) fn raw_full(
        &self,
        head: &Arc<ResidentRegion>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        cell_load_cache::load(&self.raw_full, self.partition_id, || match &self.backing {
            Some(cold) => region_load::load_full(cold, head, execution, limit),
            None => self
                .head_readmission
                .as_ref()
                .ok_or_else(|| self.failure(Cause::SourceUnavailable))?
                .load_full(head, execution, limit),
        })
    }

    fn failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
