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
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub(crate) struct RelationalColdRegion {
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
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
    pub(super) commitment: PartitionContentCommitment,
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
    partition_id: PartitionId,
    state: Mutex<ResidencyState>,
    pub(super) leases: AtomicUsize,
}

impl RelationalRegionResidency {
    pub(crate) fn cold(cold: RelationalColdRegion) -> Arc<Self> {
        Arc::new(Self {
            partition_id: cold.partition_id,
            state: Mutex::new(ResidencyState::Cold(cold)),
            leases: AtomicUsize::new(0),
        })
    }

    pub(crate) fn resident(
        partition: PartitionState,
        commitment: PartitionContentCommitment,
    ) -> RelationalResidencyLease<Full> {
        let region = Arc::new(ResidentRegion {
            partition,
            commitment,
        });
        let cell = Arc::new(Self {
            partition_id: region.partition.partition_id,
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
                let region = region_load::load_head(cold, limit)?;
                *state = ResidencyState::Head {
                    cold: cold.clone(),
                    region,
                };
            }
            if matches!(demand, ResidencyDemand::Full)
                && let ResidencyState::Head { cold, region } = &*state
            {
                let region = region_load::load_full(cold, region, limit)?;
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

    fn failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
