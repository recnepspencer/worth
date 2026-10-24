use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_wal::WalFramePlanningDenial;

use crate::physical_runtime::record_serving::PreparedPhysicalMutation;
use crate::physical_runtime::{PhysicalSignalProfileIdentity, RuntimeIdentity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalWalReservationDenial {
    PublicationAuthorityReleased,
    ForeignStore,
    StaleRuntime,
    SignalProfileMismatch,
    DuplicateUnresolved,
    AppendInFlight,
    InspectionRequired,
    LsnExhausted,
    DataPlanning(crate::physical_runtime::RecordAppendDenial),
    DataPlanBinding(crate::physical_runtime::durability::PhysicalDataPlanBindingDenial),
    FramePlanning(WalFramePlanningDenial),
    GroupFrameBytesExceedSegmentLimit {
        admitted: u64,
        requested: u64,
    },
    SegmentInventoryLimitReached {
        admitted: u32,
        retained: u32,
    },
    SegmentIdentityExhausted,
    /// Two members of one group advance the same record anchor. Refused before
    /// the group's WAL append.
    RecordAnchorConflict,
}

pub(super) struct PhysicalWalPreparationAdmission {
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    signal_profile: PhysicalSignalProfileIdentity,
}

pub(super) struct AdmittedWalPreparedMutation(PreparedPhysicalMutation);

impl PhysicalWalPreparationAdmission {
    pub(super) const fn new(
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        signal_profile: PhysicalSignalProfileIdentity,
    ) -> Self {
        Self {
            store,
            runtime,
            signal_profile,
        }
    }

    pub(super) fn admit(
        &self,
        prepared: PreparedPhysicalMutation,
    ) -> Result<AdmittedWalPreparedMutation, (PreparedPhysicalMutation, PhysicalWalReservationDenial)>
    {
        let identity = prepared.mutation_identity();
        if identity.store_identity() != self.store {
            return Err((prepared, PhysicalWalReservationDenial::ForeignStore));
        }
        if identity.runtime_identity() != self.runtime {
            return Err((prepared, PhysicalWalReservationDenial::StaleRuntime));
        }
        if prepared.signal_profile() != self.signal_profile {
            return Err((
                prepared,
                PhysicalWalReservationDenial::SignalProfileMismatch,
            ));
        }
        Ok(AdmittedWalPreparedMutation(prepared))
    }
}

impl AdmittedWalPreparedMutation {
    pub(super) fn into_prepared(self) -> PreparedPhysicalMutation {
        self.0
    }
}
