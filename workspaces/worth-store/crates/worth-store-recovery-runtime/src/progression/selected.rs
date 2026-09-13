use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalCheckpointIdentity,
};
use worth_store_recovery_physics::{PhysicalSourceSelection, SelectedPhysicalRootRole};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryOutcome, PhysicalRecoveryRefusal,
    PhysicalRecoveryRefusalKind, PhysicalRecoverySourceDenial,
};
use crate::orchestration::RecoveryCoordination;

use super::{PhysicalRecoveryDiscoveryCounters, RecoveryIntegrityEvidence};

pub struct SelectedPhysicalRecovery {
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    selection: PhysicalSourceSelection,
    integrity: RecoveryIntegrityEvidence,
    counters: PhysicalRecoveryDiscoveryCounters,
    root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl SelectedPhysicalRecovery {
    pub fn plan(self) -> Result<super::PlannedPhysicalRecovery, PhysicalRecoveryOutcome> {
        crate::orchestration::plan_recovery(self)
    }

    pub(crate) const fn new(
        authority: AdmittedPlatformAuthority,
        coordination: RecoveryCoordination,
        selection: PhysicalSourceSelection,
        integrity: RecoveryIntegrityEvidence,
        counters: PhysicalRecoveryDiscoveryCounters,
        root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
        integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        Self {
            authority,
            coordination,
            selection,
            integrity,
            counters,
            root_protocol_denials,
            integrity_trace,
        }
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.authority.media.store_identity()
    }

    pub const fn root_generation(&self) -> u64 {
        self.selection
            .root()
            .selected()
            .selector()
            .root_generation()
    }

    pub const fn root_role(&self) -> SelectedPhysicalRootRole {
        self.selection.root().role()
    }

    pub fn checkpoint_identity(&self) -> Option<PhysicalCheckpointIdentity> {
        self.selection
            .checkpoint()
            .map(|checkpoint| checkpoint.checkpoint().source().identity())
    }

    pub fn selected_page_fact_count(&self) -> u64 {
        self.selection.page_facts().placements().len() as u64
    }

    pub const fn distinct_page_and_extent_count(&self) -> u64 {
        self.selection.page_facts().distinct_pages_and_extents()
    }

    pub fn wal_segment_count(&self) -> u64 {
        self.selection.wal_tail().segments().len() as u64
    }

    pub const fn wal_frame_count(&self) -> u64 {
        self.selection.wal_tail().frame_count()
    }

    pub fn residue_count(&self) -> u64 {
        self.selection.residue().len() as u64
    }

    pub const fn source_trace(&self) -> worth_store_recovery_physics::PhysicalSourceSelectionTrace {
        self.selection.trace()
    }

    pub fn compaction_generation(&self) -> Option<u64> {
        self.selection
            .compaction()
            .map(|compaction| compaction.cutover().product_generation())
    }

    pub const fn discovery_counters(&self) -> PhysicalRecoveryDiscoveryCounters {
        self.counters
    }

    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.root_protocol_denials
    }

    pub const fn integrity_observation_count(&self) -> u64 {
        self.integrity_trace.counters().attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.integrity_trace.observations()
    }

    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.integrity.observations().wal()
    }

    pub fn cancel_before_reconstruction(self) -> PhysicalRecoveryOutcome {
        let Self {
            authority,
            coordination,
            integrity,
            root_protocol_denials,
            integrity_trace,
            ..
        } = self;
        assert!(coordination.shutdown_is_quiescent());
        let recovery_effects = authority.media.recovery_effect_count();
        let AdmittedPlatformAuthority { media, session, .. } = authority;
        drop(media);
        session.refuse();
        PhysicalRecoveryOutcome::Refused(
            PhysicalRecoveryRefusal::new(
                PhysicalRecoveryRefusalKind::CancelledBeforeReconstruction,
                recovery_effects,
            )
            .with_root_protocol_denials(root_protocol_denials)
            .with_integrity_trace(integrity_trace)
            .with_integrity_observations(integrity.into_observations()),
        )
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        AdmittedPlatformAuthority,
        RecoveryCoordination,
        PhysicalSourceSelection,
        RecoveryIntegrityEvidence,
        PhysicalRecoveryDiscoveryCounters,
        Vec<PhysicalRecoverySourceDenial>,
        crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) {
        (
            self.authority,
            self.coordination,
            self.selection,
            self.integrity,
            self.counters,
            self.root_protocol_denials,
            self.integrity_trace,
        )
    }
}
