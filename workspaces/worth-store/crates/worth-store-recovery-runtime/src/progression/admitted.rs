use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::entry::{
    counter_snapshot, AdmittedPlatformAuthority, PhysicalRecoveryAdmissionCounters,
    PhysicalRecoveryLimits, PhysicalRecoveryOutcome, PhysicalRecoveryRefusal,
    PhysicalRecoveryRefusalKind, PhysicalRecoverySessionIdentity,
};
use crate::orchestration::RecoveryCoordination;

pub struct AdmittedPhysicalRecovery {
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
}

impl AdmittedPhysicalRecovery {
    pub fn discover(self) -> Result<super::DiscoveredPhysicalRecovery, PhysicalRecoveryOutcome> {
        let Self {
            authority,
            coordination,
        } = self;
        match crate::orchestration::discover_sources(authority, coordination) {
            Ok(material) => Ok(super::DiscoveredPhysicalRecovery::from_material(material)),
            Err((authority, coordination, scope, evidence)) => Err(
                crate::handoff::block_unsupported_scope(authority, coordination, scope, evidence),
            ),
        }
    }

    pub(crate) const fn from_admission(
        authority: AdmittedPlatformAuthority,
        coordination: RecoveryCoordination,
    ) -> Self {
        Self {
            authority,
            coordination,
        }
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.authority.media.store_identity()
    }

    pub const fn session_identity(&self) -> PhysicalRecoverySessionIdentity {
        self.authority.session.identity()
    }

    pub const fn limits(&self) -> PhysicalRecoveryLimits {
        self.authority.limits
    }

    pub fn counters(&self) -> PhysicalRecoveryAdmissionCounters {
        counter_snapshot(Some(self.authority.media.recovery_effect_count()))
    }

    pub fn cancel_before_discovery(self) -> PhysicalRecoveryOutcome {
        let Self {
            authority,
            coordination,
        } = self;
        assert!(
            coordination.shutdown_is_quiescent(),
            "Phase 2 recovery coordination must be quiescent before cancellation"
        );
        let recovery_effects = authority.media.recovery_effect_count();
        let AdmittedPlatformAuthority { media, session, .. } = authority;
        drop(media);
        session.refuse();
        PhysicalRecoveryOutcome::Refused(Box::new(PhysicalRecoveryRefusal::new(
            PhysicalRecoveryRefusalKind::CancelledBeforeDiscovery,
            recovery_effects,
        )))
    }

    #[cfg(test)]
    pub(crate) fn proves_live_coordination_for_test(&self) -> bool {
        self.coordination.is_ready()
    }
}
