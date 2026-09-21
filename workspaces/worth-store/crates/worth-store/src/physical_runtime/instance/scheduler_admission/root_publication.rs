use super::{PhysicalSchedulerAdmissionOwner, RecordSchedulerReservationDenial};
use worth_store_io_scheduler::foreground_reservation::{
    ForegroundLaneDeclaration, ForegroundLatencyEnvelope, PhysicalInstanceForegroundReservation,
};
use worth_store_io_scheduler::{
    IoSchedulerBackendCapabilityAdmission, IoSchedulerSecurityScopeAdmission,
};

pub(in crate::physical_runtime) struct RootCandidateSyncSchedulerAdmission {
    reservation: PhysicalInstanceForegroundReservation,
    backend: IoSchedulerBackendCapabilityAdmission,
}

#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) struct RootCandidateMaterializationSchedulerAdmission {
    reservation: PhysicalInstanceForegroundReservation,
    backend: IoSchedulerBackendCapabilityAdmission,
}

pub(in crate::physical_runtime) struct RootCatalogReplacementSchedulerAdmission {
    reservation: PhysicalInstanceForegroundReservation,
    backend: IoSchedulerBackendCapabilityAdmission,
}

pub(in crate::physical_runtime) struct RootNamespaceSyncSchedulerAdmission {
    reservation: PhysicalInstanceForegroundReservation,
    backend: IoSchedulerBackendCapabilityAdmission,
}

impl PhysicalSchedulerAdmissionOwner {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn root_candidate_materialization(
        &self,
        security: &IoSchedulerSecurityScopeAdmission,
    ) -> Result<RootCandidateMaterializationSchedulerAdmission, RecordSchedulerReservationDenial>
    {
        let lane = ForegroundLaneDeclaration::root_candidate_materialization()
            .expect("root candidate materialization is a Store-owned lane");
        let reservation = self.reserve_root_lane(lane, &self.buffered_file, security)?;
        Ok(RootCandidateMaterializationSchedulerAdmission {
            reservation,
            backend: self.buffered_file,
        })
    }

    pub(in crate::physical_runtime) fn root_candidate_sync(
        &self,
        security: &IoSchedulerSecurityScopeAdmission,
    ) -> Result<RootCandidateSyncSchedulerAdmission, RecordSchedulerReservationDenial> {
        let lane = ForegroundLaneDeclaration::root_candidate_synchronization()
            .expect("root candidate synchronization is a Store-owned lane");
        let reservation = self.reserve_root_lane(lane, &self.fsync, security)?;
        Ok(RootCandidateSyncSchedulerAdmission {
            reservation,
            backend: self.fsync,
        })
    }

    pub(in crate::physical_runtime) fn root_catalog_replacement(
        &self,
        security: &IoSchedulerSecurityScopeAdmission,
    ) -> Result<RootCatalogReplacementSchedulerAdmission, RecordSchedulerReservationDenial> {
        let lane = ForegroundLaneDeclaration::root_catalog_replacement()
            .expect("root catalog replacement is a Store-owned lane");
        let reservation = self.reserve_root_lane(lane, &self.durable_rename, security)?;
        Ok(RootCatalogReplacementSchedulerAdmission {
            reservation,
            backend: self.durable_rename,
        })
    }

    pub(in crate::physical_runtime) fn root_namespace_sync(
        &self,
        security: &IoSchedulerSecurityScopeAdmission,
    ) -> Result<RootNamespaceSyncSchedulerAdmission, RecordSchedulerReservationDenial> {
        let lane = ForegroundLaneDeclaration::root_namespace_synchronization()
            .expect("root namespace synchronization is a Store-owned lane");
        let reservation = self.reserve_root_lane(lane, &self.directory_sync, security)?;
        Ok(RootNamespaceSyncSchedulerAdmission {
            reservation,
            backend: self.directory_sync,
        })
    }

    fn reserve_root_lane(
        &self,
        lane: ForegroundLaneDeclaration,
        backend: &IoSchedulerBackendCapabilityAdmission,
        security: &IoSchedulerSecurityScopeAdmission,
    ) -> Result<PhysicalInstanceForegroundReservation, RecordSchedulerReservationDenial> {
        self.reserve_selected_foreground(
            lane.with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
                "physical-root-publication",
                2,
            ))
            .with_budget(super::wal_barrier_budget()),
            backend,
            security,
        )
    }
}

#[cfg(feature = "recovery-runtime-owner")]
impl RootCandidateMaterializationSchedulerAdmission {
    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        PhysicalInstanceForegroundReservation,
        IoSchedulerBackendCapabilityAdmission,
    ) {
        (self.reservation, self.backend)
    }
}

impl RootCandidateSyncSchedulerAdmission {
    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        PhysicalInstanceForegroundReservation,
        IoSchedulerBackendCapabilityAdmission,
    ) {
        (self.reservation, self.backend)
    }
}

impl RootCatalogReplacementSchedulerAdmission {
    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        PhysicalInstanceForegroundReservation,
        IoSchedulerBackendCapabilityAdmission,
    ) {
        (self.reservation, self.backend)
    }
}

impl RootNamespaceSyncSchedulerAdmission {
    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        PhysicalInstanceForegroundReservation,
        IoSchedulerBackendCapabilityAdmission,
    ) {
        (self.reservation, self.backend)
    }
}
