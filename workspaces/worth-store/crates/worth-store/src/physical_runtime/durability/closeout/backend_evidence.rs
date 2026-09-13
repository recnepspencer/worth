use worth_store_physical_backend::{BackendTargetProfile, PhysicalDurabilityAdmissionIdentity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalBackendDurabilityCloseoutEvidence {
    admission: PhysicalDurabilityAdmissionIdentity,
    profile: BackendTargetProfile,
    durable_wal_lsn_end: Option<worth_store_wal::LogSequenceNumber>,
    root_namespace: crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence,
}

impl PhysicalBackendDurabilityCloseoutEvidence {
    pub(in crate::physical_runtime) const fn new(
        admission: PhysicalDurabilityAdmissionIdentity,
        profile: BackendTargetProfile,
        durable_wal_lsn_end: Option<worth_store_wal::LogSequenceNumber>,
        root_namespace: crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence,
    ) -> Self {
        Self {
            admission,
            profile,
            durable_wal_lsn_end,
            root_namespace,
        }
    }

    pub const fn admission_identity(self) -> PhysicalDurabilityAdmissionIdentity {
        self.admission
    }

    pub const fn profile(self) -> BackendTargetProfile {
        self.profile
    }

    pub const fn durable_wal_lsn_end(self) -> Option<worth_store_wal::LogSequenceNumber> {
        self.durable_wal_lsn_end
    }

    pub const fn root_namespace_evidence(
        self,
    ) -> crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence {
        self.root_namespace
    }
}
