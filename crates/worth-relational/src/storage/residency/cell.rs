#[path = "cell/observation.rs"]
mod observation;
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
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub(crate) struct RelationalColdRegion {
    pub(crate) kind_authority: super::RelationalRegionKindCommitment,
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) partition_id: PartitionId,
    pub(crate) head: RelationalDurableMemberLocator,
    pub(crate) history: RelationalDurableMemberLocator,
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
    Damaged(RelationalStorageUnavailable),
}

#[derive(Debug)]
pub(crate) struct RelationalRegionResidency {
    pub(super) partition_id: PartitionId,
    pub(super) checkpoint_bindings: Mutex<
        std::collections::BTreeMap<[u8; 32], crate::durability::data::RelationalCheckpointRegion>,
    >,
    state: Mutex<ResidencyState>,
    backing: Option<RelationalColdRegion>,
    backing_custody: Option<crate::durability::medium::RelationalDurableMemberCustody>,
    raw_head: RegionLoadCache,
    raw_full: RegionLoadCache,
    pub(super) leases: AtomicUsize,
}

impl RelationalRegionResidency {
    pub(crate) fn cold(
        cold: RelationalColdRegion,
    ) -> Result<Arc<Self>, RelationalStorageUnavailable> {
        let custody = cold
            .source
            .retain_members(&[cold.head, cold.history])
            .map_err(|cause| RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(cold.partition_id),
                cause,
            })?;
        Ok(Arc::new(Self {
            partition_id: cold.partition_id,
            checkpoint_bindings: Mutex::new(std::collections::BTreeMap::from([(
                cold.schema_carrier_digest,
                crate::durability::data::RelationalCheckpointRegion {
                    kind_authority: cold.kind_authority,
                    partition_id: cold.partition_id,
                    content_digest: cold.content_digest,
                    entity_slots: cold.entity_slots,
                    relation_slots: cold.relation_slots,
                    unavailable_records: cold.unavailable_records,
                    head: cold.head,
                    history: cold.history,
                },
            )])),
            backing: Some(cold),
            backing_custody: Some(custody),
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(ResidencyState::Cold),
            leases: AtomicUsize::new(0),
        }))
    }

    pub(crate) fn resident(
        partition: PartitionState,
        commitment: PartitionContentCommitment,
    ) -> RelationalResidencyLease<Full> {
        Self::resident_with_commitment(partition, Some(commitment))
    }

    pub(crate) fn resident_uncommitted(
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
            backing_custody: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(ResidencyState::Full(region.clone())),
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
            if let ResidencyState::Damaged(failure) = &*state {
                return Err(Failure::StorageUnavailable(*failure));
            }
            if let ResidencyState::Cold = &*state {
                let region = self.raw_head(execution, limit)?;
                *state = ResidencyState::Head { region };
            }
            if matches!(demand, ResidencyDemand::Full)
                && let ResidencyState::Head { .. } = &*state
            {
                let region = self.raw_full(execution, limit)?;
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
                    *state = ResidencyState::Damaged(failure);
                }
                Err(Failure::StorageUnavailable(failure))
            }
            Err(Failure::Request(cause)) => Err(Failure::Request(cause)),
        }
    }

    pub(crate) fn has_cold_backing(&self) -> bool {
        self.backing.is_some()
    }

    pub(crate) fn into_partition(self) -> PartitionState {
        drop(self.backing_custody);
        drop(
            self.raw_head
                .into_inner()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        drop(
            self.raw_full
                .into_inner()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        let state = self
            .state
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let ResidencyState::Full(region) = state else {
            panic!("publication requires full residency")
        };
        match Arc::try_unwrap(region) {
            Ok(region) => region.partition,
            Err(region) => region.partition.clone(),
        }
    }

    pub(super) fn partition_mut(&mut self) -> Option<&mut PartitionState> {
        // Only an exclusive mirror writer can reach this method. Old roots and
        // editions retain their old cell; no accepted lease sees a mutation.
        self.raw_head
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        self.raw_full
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        self.backing = None;
        self.backing_custody.take();
        self.checkpoint_bindings
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let ResidencyState::Full(region) = state else {
            return None;
        };
        let region = Arc::get_mut(region)?;
        region.commitment = None;
        Some(&mut region.partition)
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        cell_load_cache::load(&self.raw_full, self.partition_id, || {
            let cold = self
                .backing
                .as_ref()
                .ok_or_else(|| self.failure(Cause::SourceUnavailable))?;
            let head = self.raw_head(execution, limit)?;
            region_load::load_full(cold, &head, execution, limit)
        })
    }

    fn failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
