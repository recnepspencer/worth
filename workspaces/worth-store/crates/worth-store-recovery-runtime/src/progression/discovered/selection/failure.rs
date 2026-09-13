use crate::entry::{
    PhysicalRecoveryBlockEvidence, PhysicalRecoveryBlockKind, PhysicalRecoverySourceDenial,
};

use super::PhysicalRecoveryDiscoveryCounters;

pub(in crate::progression::discovered) struct SelectionFailure {
    pub(in crate::progression::discovered) kind: PhysicalRecoveryBlockKind,
    pub(in crate::progression::discovered) evidence: PhysicalRecoveryBlockEvidence,
}

impl SelectionFailure {
    pub(super) fn with_integrity_trace(
        mut self,
        trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        self.evidence.integrity_trace = trace;
        self
    }
    pub(super) fn new(
        kind: PhysicalRecoveryBlockKind,
        counters: PhysicalRecoveryDiscoveryCounters,
        artifact: &str,
    ) -> Self {
        Self {
            kind,
            evidence: PhysicalRecoveryBlockEvidence {
                counters,
                artifact: Some(artifact.to_owned()),
                ..PhysicalRecoveryBlockEvidence::default()
            },
        }
    }

    pub(super) fn with_generation(mut self, generation: u64) -> Self {
        self.evidence.source_generation = Some(generation);
        self
    }

    pub(super) fn with_lsn(mut self, lsn: u64) -> Self {
        self.evidence.lsn = Some(lsn);
        self
    }

    pub(super) fn with_source_denials(
        mut self,
        denials: Vec<PhysicalRecoverySourceDenial>,
    ) -> Self {
        self.evidence.source_denials = denials;
        self
    }

    pub(super) fn with_integrity_observations(
        mut self,
        wal: Vec<crate::entry::PhysicalRecoveryWalIntegrityObservation>,
    ) -> Self {
        self.evidence.integrity_observations =
            crate::entry::PhysicalRecoveryIntegrityObservations::new(wal);
        self
    }

    pub(super) fn with_root_protocol_denials(
        mut self,
        denials: &[PhysicalRecoverySourceDenial],
    ) -> Self {
        let mut combined = denials.to_vec();
        combined.append(&mut self.evidence.source_denials);
        self.evidence.source_denials = combined;
        self
    }
}
