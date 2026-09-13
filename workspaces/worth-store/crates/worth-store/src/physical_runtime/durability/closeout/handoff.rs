use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{
    PhysicalArtifactResidueClassification, PhysicalBackendDurabilityCloseoutEvidence,
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryCheckpointBasis,
    PhysicalRecoveryOperationFates, PhysicalRecoveryRootBasis, PhysicalRecoveryWalTail,
};

pub struct PhysicalDurabilityRecoveryHandoff {
    policy: crate::physical_runtime::AdmittedPhysicalDurabilityPolicy,
    backend: PhysicalBackendDurabilityCloseoutEvidence,
    roots: PhysicalRecoveryRootBasis,
    checkpoint: PhysicalRecoveryCheckpointBasis,
    wal_tail: PhysicalRecoveryWalTail,
    operations: PhysicalRecoveryOperationFates,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    residue: PhysicalArtifactResidueClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDurabilityCloseoutDenial {
    LiveIdempotencyCompactionAuthority,
}

pub enum PhysicalDurabilityCloseoutOutcome {
    RecoveryHandoff(PhysicalDurabilityRecoveryHandoff),
    InspectionRequired(PhysicalDurabilityCloseoutDenial),
    NotProducedForAbort,
}

impl PhysicalDurabilityRecoveryHandoff {
    pub(in crate::physical_runtime) fn finalize(
        policy: crate::physical_runtime::AdmittedPhysicalDurabilityPolicy,
        roots: PhysicalRecoveryRootBasis,
        checkpoint: PhysicalRecoveryCheckpointBasis,
        wal_tail: PhysicalRecoveryWalTail,
        operations: PhysicalRecoveryOperationFates,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        residue: PhysicalArtifactResidueClassification,
    ) -> Self {
        assert_eq!(
            policy.store_identity(),
            recovery_allocation.store_identity(),
            "closeout facts must remain bound to one Store"
        );
        let backend = PhysicalBackendDurabilityCloseoutEvidence::new(
            policy.admission_basis_identity(),
            policy.profile(),
            wal_tail.durable_lsn_end(),
            roots.namespace_evidence(),
        );
        Self {
            policy,
            backend,
            roots,
            checkpoint,
            wal_tail,
            operations,
            recovery_allocation,
            residue,
        }
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.policy.store_identity()
    }

    pub const fn durability_policy(
        &self,
    ) -> &crate::physical_runtime::AdmittedPhysicalDurabilityPolicy {
        &self.policy
    }

    pub const fn backend_evidence(&self) -> PhysicalBackendDurabilityCloseoutEvidence {
        self.backend
    }

    pub const fn roots(&self) -> &PhysicalRecoveryRootBasis {
        &self.roots
    }

    pub const fn checkpoint(&self) -> &PhysicalRecoveryCheckpointBasis {
        &self.checkpoint
    }

    pub const fn wal_tail(&self) -> &PhysicalRecoveryWalTail {
        &self.wal_tail
    }

    pub const fn operation_fates(&self) -> &PhysicalRecoveryOperationFates {
        &self.operations
    }

    pub const fn recovery_allocation(&self) -> PhysicalRecoveryAllocationAdmission {
        self.recovery_allocation
    }

    pub const fn residue(&self) -> PhysicalArtifactResidueClassification {
        self.residue
    }

    pub fn requires_inspection(&self) -> bool {
        self.wal_tail.requires_inspection()
            || self.residue.requires_inspection()
            || self.operations.counts().indeterminate() != 0
    }
}

impl PhysicalDurabilityCloseoutOutcome {
    pub const fn recovery_handoff(&self) -> Option<&PhysicalDurabilityRecoveryHandoff> {
        match self {
            Self::RecoveryHandoff(handoff) => Some(handoff),
            Self::InspectionRequired(_) | Self::NotProducedForAbort => None,
        }
    }

    pub fn requires_inspection(&self) -> bool {
        match self {
            Self::RecoveryHandoff(handoff) => handoff.requires_inspection(),
            Self::InspectionRequired(_) => true,
            Self::NotProducedForAbort => false,
        }
    }

    pub fn into_recovery_handoff(self) -> Option<PhysicalDurabilityRecoveryHandoff> {
        match self {
            Self::RecoveryHandoff(handoff) => Some(handoff),
            Self::InspectionRequired(_) | Self::NotProducedForAbort => None,
        }
    }
}
