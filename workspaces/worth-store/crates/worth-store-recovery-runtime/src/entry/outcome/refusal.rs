use worth_store::physical_runtime::RecoveryFilesystemQualificationError;

use crate::entry::{self, PhysicalRecoveryEntryBindingDrift};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRecoveryRefusal {
    pub kind: PhysicalRecoveryRefusalKind,
    root_protocol_denials: Vec<entry::PhysicalRecoverySourceDenial>,
    root_protocol_counters: entry::PhysicalRecoveryRootProtocolCounters,
    integrity_observations: entry::PhysicalRecoveryIntegrityObservations,
    recovery_effects: u64,
    integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl PhysicalRecoveryRefusal {
    pub(crate) fn new(kind: PhysicalRecoveryRefusalKind, recovery_effects: u64) -> Self {
        Self {
            kind,
            root_protocol_denials: Vec::new(),
            root_protocol_counters: entry::PhysicalRecoveryRootProtocolCounters::default(),
            integrity_observations: entry::PhysicalRecoveryIntegrityObservations::new(Vec::new()),
            recovery_effects,
            integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace::new(),
        }
    }

    pub const fn recovery_effects(&self) -> u64 {
        self.recovery_effects
    }

    pub(crate) fn with_root_protocol_denials(
        mut self,
        denials: Vec<entry::PhysicalRecoverySourceDenial>,
    ) -> Self {
        self.root_protocol_denials = denials;
        self
    }

    pub fn root_protocol_denials(&self) -> &[entry::PhysicalRecoverySourceDenial] {
        &self.root_protocol_denials
    }

    pub(crate) const fn with_root_protocol_counters(
        mut self,
        counters: entry::PhysicalRecoveryRootProtocolCounters,
    ) -> Self {
        self.root_protocol_counters = counters;
        self
    }

    pub const fn root_protocol_counters(&self) -> entry::PhysicalRecoveryRootProtocolCounters {
        self.root_protocol_counters
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.integrity_trace.counters().attempted
    }

    pub const fn integrity_counters(&self) -> crate::PhysicalRecoveryIntegrityCounters {
        self.integrity_trace.counters()
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.integrity_trace.observations()
    }

    pub(crate) fn with_integrity_trace(
        mut self,
        trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        self.integrity_trace = trace;
        self
    }

    pub(crate) fn with_integrity_observations(
        mut self,
        observations: entry::PhysicalRecoveryIntegrityObservations,
    ) -> Self {
        self.integrity_observations = observations;
        self
    }

    pub const fn wal_integrity_observations(
        &self,
    ) -> &entry::PhysicalRecoveryIntegrityObservations {
        &self.integrity_observations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryRefusalKind {
    CancelledBeforeDiscovery,
    CancelledBeforeReconstruction,
    CancelledBeforeExecution,
    EntryBindingDrift(PhysicalRecoveryEntryBindingDrift),
    PersistedStoreAdmission(RecoveryFilesystemQualificationError),
    CoordinationUnavailable,
}
