use super::{region_load, Full, Head, RelationalResidencyLease};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::PartitionId;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::symbols::data::StringInterner;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
pub(crate) struct RelationalColdRegion {
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) kind_authority: Arc<super::kind_authority::RelationalRegionKindAuthority>,
    pub(crate) partition_id: PartitionId,
    pub(crate) head: RelationalDurableMemberLocator,
    pub(crate) history: RelationalDurableMemberLocator,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) schema_carrier_digest: [u8; 32],
    pub(crate) symbols: Arc<StringInterner>,
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
    Cold(RelationalColdRegion),
    Head {
        cold: RelationalColdRegion,
        region: Arc<ResidentRegion>,
    },
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
    raw_head: OnceLock<Result<Arc<ResidentRegion>, Cause>>,
    raw_full: OnceLock<Result<Arc<ResidentRegion>, Cause>>,
    pub(super) leases: AtomicUsize,
}

impl RelationalRegionResidency {
    pub(crate) fn cold(cold: RelationalColdRegion) -> Arc<Self> {
        Arc::new(Self {
            partition_id: cold.partition_id,
            checkpoint_bindings: Mutex::new(std::collections::BTreeMap::from([(
                cold.schema_carrier_digest,
                crate::durability::data::RelationalCheckpointRegion {
                    partition_id: cold.partition_id,
                    content_digest: cold.content_digest,
                    entity_slots: cold.entity_slots,
                    relation_slots: cold.relation_slots,
                    unavailable_records: cold.unavailable_records,
                    head: cold.head,
                    history: cold.history,
                },
            )])),
            backing: Some(cold.clone()),
            raw_head: OnceLock::new(),
            raw_full: OnceLock::new(),
            state: Mutex::new(ResidencyState::Cold(cold)),
            leases: AtomicUsize::new(0),
        })
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
            raw_head: OnceLock::new(),
            raw_full: OnceLock::new(),
            state: Mutex::new(ResidencyState::Full(region.clone())),
            leases: AtomicUsize::new(1),
        });
        RelationalResidencyLease::accepted(cell, region)
    }

    /// Explicit I/O boundary. A single cell lock covers each load and its
    /// publication, so concurrent demands share one read and one refusal.
    pub(crate) fn demand_head(
        self: &Arc<Self>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Head>, RelationalStorageUnavailable> {
        let region = self.demand(ResidencyDemand::Head, limit)?;
        Ok(RelationalResidencyLease::accepted(self.clone(), region))
    }

    pub(crate) fn demand_full(
        self: &Arc<Self>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Full>, RelationalStorageUnavailable> {
        let region = self.demand(ResidencyDemand::Full, limit)?;
        Ok(RelationalResidencyLease::accepted(self.clone(), region))
    }

    fn demand(
        &self,
        demand: ResidencyDemand,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, RelationalStorageUnavailable> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let loaded = (|| {
            if let ResidencyState::Damaged(failure) = &*state {
                return Err(failure.cause.clone());
            }
            if let ResidencyState::Cold(cold) = &*state {
                let region = self.raw_head(limit)?;
                cold.kind_authority.verify_head(&region, limit)?;
                *state = ResidencyState::Head {
                    cold: cold.clone(),
                    region,
                };
            }
            if matches!(demand, ResidencyDemand::Full)
                && let ResidencyState::Head { cold, .. } = &*state
            {
                let region = self.raw_full(limit)?;
                let region = cold.kind_authority.admit_full(&region, limit)?;
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
            Err(cause) => {
                let failure = self.failure(cause);
                *state = ResidencyState::Damaged(failure.clone());
                Err(failure)
            }
        }
    }

    pub(crate) fn resident_inventory(
        &self,
    ) -> crate::storage::overlay::RelationalPartitionAllocationInventory {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => {
                region.partition.allocation_inventory()
            }
            _ => Default::default(),
        }
    }

    pub(crate) fn residency_allocation_bytes(&self) -> u64 {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let resident = matches!(
            &*state,
            ResidencyState::Head { .. } | ResidencyState::Full(_)
        );
        (std::mem::size_of::<Self>()
            + 2 * std::mem::size_of::<usize>()
            + if resident {
                std::mem::size_of::<ResidentRegion>() + 2 * std::mem::size_of::<usize>()
            } else {
                0
            }) as u64
    }

    pub(crate) fn resident_commitment_bytes(&self) -> u64 {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => region
                .commitment
                .as_ref()
                .map_or(0, |commitment| commitment.allocation_bytes()),
            _ => 0,
        }
    }

    pub(crate) fn lease_resident_full(
        self: &Arc<Self>,
    ) -> Result<Option<RelationalResidencyLease<Full>>, RelationalStorageUnavailable> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*state {
            ResidencyState::Full(region) => {
                self.leases
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        count.checked_add(1)
                    })
                    .map_err(|_| self.failure(Cause::MemoryExhausted))?;
                Ok(Some(RelationalResidencyLease::accepted(
                    self.clone(),
                    region.clone(),
                )))
            }
            ResidencyState::Damaged(failure) => Err(failure.clone()),
            _ => Ok(None),
        }
    }

    pub(crate) fn into_partition(self) -> PartitionState {
        drop(self.raw_head.into_inner());
        drop(self.raw_full.into_inner());
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

    pub(crate) fn partition_mut(&mut self) -> &mut PartitionState {
        // Only an exclusive mirror writer can reach this method. Old roots and
        // editions retain their old cell; no accepted lease sees a mutation.
        self.raw_head.take();
        self.raw_full.take();
        self.backing = None;
        self.checkpoint_bindings
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let ResidencyState::Full(region) = state else {
            panic!("writer requires full residency")
        };
        let region = Arc::get_mut(region).expect("exclusive cell has no readers");
        region.commitment = None;
        &mut region.partition
    }

    // Raw readmission authenticates record content independently of derived
    // buckets. Keeping it separate breaks cross-partition adjacency cycles:
    // bucket verification never waits for another region's serving admission.
    pub(super) fn raw_head(&self, limit: u64) -> Result<Arc<ResidentRegion>, Cause> {
        self.raw_head
            .get_or_init(|| {
                let cold = self.backing.as_ref().ok_or(Cause::SourceUnavailable)?;
                region_load::load_head(cold, limit)
            })
            .clone()
    }

    pub(super) fn raw_full(&self, limit: u64) -> Result<Arc<ResidentRegion>, Cause> {
        self.raw_full
            .get_or_init(|| {
                let cold = self.backing.as_ref().ok_or(Cause::SourceUnavailable)?;
                let head = self.raw_head(limit)?;
                region_load::load_full(cold, &head, limit)
            })
            .clone()
    }

    fn failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
