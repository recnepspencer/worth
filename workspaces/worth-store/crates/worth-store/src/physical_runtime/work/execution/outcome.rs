use worth_store_io_scheduler::QueueExecutionOutcome;
use worth_store_physical_backend::{
    ArtifactTreeFailure, CompletedArtifactAppend, CompletedArtifactMetadataRead,
    CompletedArtifactNewWrite, CompletedArtifactRangeRead, CompletedArtifactRangeWrite,
    CompletedArtifactTreePublicationEffect, IndeterminateArtifactAppend,
    IndeterminateArtifactNewWrite, IndeterminateArtifactRangeWrite,
    IndeterminateArtifactTreePublicationEffect,
};
#[cfg(feature = "recovery-runtime-owner")]
use worth_store_physical_backend::{
    CompletedRecoveryStagingWrite, IndeterminateRecoveryStagingWrite,
};
use worth_store_physical_format::RecordArtifactFile;

use super::PhysicalPublicationEffect;

mod checkpoint;
mod residency_writeback;
mod wal_reclamation;

pub use checkpoint::CompletedPhysicalCheckpointAction;
pub(in crate::physical_runtime) use checkpoint::IndeterminatePhysicalCheckpointAction;
pub(in crate::physical_runtime) use residency_writeback::PhysicalResidencyWritebackCompletion;
pub use wal_reclamation::CompletedPhysicalWalReclamationAction;
pub(in crate::physical_runtime) use wal_reclamation::IndeterminatePhysicalWalReclamationAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalEffectRecoveryObligation {
    Cleared,
    Retained,
}

pub(in crate::physical_runtime) struct PhysicalExecutorDispatch {
    dispatched: super::super::DispatchedPhysicalWork,
    outcome: PhysicalExecutorOutcome,
    recovery: PhysicalEffectRecoveryObligation,
    residency_writeback: Option<PhysicalResidencyWritebackCompletion>,
}

pub struct CompletedPhysicalPublicationEffect {
    physical: CompletedArtifactTreePublicationEffect,
    artifact: RecordArtifactFile,
    effect: PhysicalPublicationEffect,
}

pub(in crate::physical_runtime) struct IndeterminatePhysicalPublicationEffect {
    physical: IndeterminateArtifactTreePublicationEffect,
    artifact: RecordArtifactFile,
    effect: PhysicalPublicationEffect,
}

pub struct CompletedPhysicalWalBarrier {
    physical: CompletedArtifactTreePublicationEffect,
    artifact: worth_store_physical_backend::ArtifactTreeFile,
}

pub(in crate::physical_runtime) struct IndeterminatePhysicalWalBarrier {
    physical: IndeterminateArtifactTreePublicationEffect,
    artifact: worth_store_physical_backend::ArtifactTreeFile,
}

pub(in crate::physical_runtime) enum PhysicalExecutorOutcome {
    InspectionObserved {
        physical: worth_store_physical_backend::ObservedArtifactInspectionRead,
        bytes: Box<[u8]>,
        scheduler: QueueExecutionOutcome,
    },
    InspectionDenied(worth_store_physical_backend::ArtifactTreeFailure),
    DeniedBeforeEffect {
        failure: ArtifactTreeFailure,
        retry: super::PhysicalRetryPayload,
    },
    MetadataCompleted {
        physical: CompletedArtifactMetadataRead,
        scheduler: QueueExecutionOutcome,
    },
    ReadCompleted {
        physical: CompletedArtifactRangeRead,
        bytes: Box<[u8]>,
        scheduler: QueueExecutionOutcome,
    },
    WriteCompleted {
        physical: CompletedArtifactRangeWrite,
        scheduler: QueueExecutionOutcome,
    },
    ResidencyWritebackCompleted {
        physical: CompletedArtifactRangeWrite,
        scheduler: QueueExecutionOutcome,
    },
    PublicationCompleted {
        physical: CompletedArtifactRangeWrite,
        scheduler: QueueExecutionOutcome,
    },
    NewArtifactCompleted {
        physical: CompletedArtifactNewWrite,
        coordinate: worth_store_physical_format::RecordFrameCoordinate,
        scheduler: QueueExecutionOutcome,
    },
    #[cfg(feature = "recovery-runtime-owner")]
    RecoveryStagingCompleted {
        physical: CompletedRecoveryStagingWrite,
        scheduler: QueueExecutionOutcome,
    },
    PublicationEffectCompleted {
        physical: CompletedPhysicalPublicationEffect,
        scheduler: QueueExecutionOutcome,
    },
    WalAppendCompleted {
        physical: CompletedArtifactAppend,
        scheduler: QueueExecutionOutcome,
    },
    WalSegmentCreateCompleted {
        physical: CompletedArtifactNewWrite,
        scheduler: QueueExecutionOutcome,
    },
    WalBarrierCompleted {
        physical: CompletedPhysicalWalBarrier,
        scheduler: QueueExecutionOutcome,
    },
    CheckpointCompleted {
        physical: CompletedPhysicalCheckpointAction,
        scheduler: QueueExecutionOutcome,
    },
    WalReclamationCompleted {
        physical: CompletedPhysicalWalReclamationAction,
        scheduler: QueueExecutionOutcome,
    },
    Indeterminate(IndeterminateArtifactRangeWrite),
    NewArtifactIndeterminate {
        physical: IndeterminateArtifactNewWrite,
        coordinate: worth_store_physical_format::RecordFrameCoordinate,
    },
    #[cfg(feature = "recovery-runtime-owner")]
    RecoveryStagingIndeterminate(IndeterminateRecoveryStagingWrite),
    PublicationEffectIndeterminate(IndeterminatePhysicalPublicationEffect),
    WalAppendIndeterminate(IndeterminateArtifactAppend),
    WalSegmentCreateIndeterminate(IndeterminateArtifactNewWrite),
    WalBarrierIndeterminate(IndeterminatePhysicalWalBarrier),
    CheckpointIndeterminate(IndeterminatePhysicalCheckpointAction),
    WalReclamationIndeterminate(IndeterminatePhysicalWalReclamationAction),
}

impl PhysicalEffectRecoveryObligation {
    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) const fn join(self, other: Self) -> Self {
        if matches!(self, Self::Retained) || matches!(other, Self::Retained) {
            Self::Retained
        } else {
            Self::Cleared
        }
    }

    pub(in crate::physical_runtime) const fn is_retained(self) -> bool {
        matches!(self, Self::Retained)
    }
}

impl PhysicalExecutorDispatch {
    pub(in crate::physical_runtime) const fn new(
        dispatched: super::super::DispatchedPhysicalWork,
        outcome: PhysicalExecutorOutcome,
        recovery: PhysicalEffectRecoveryObligation,
    ) -> Self {
        Self {
            dispatched,
            outcome,
            recovery,
            residency_writeback: None,
        }
    }

    pub(in crate::physical_runtime) const fn with_residency_writeback_completion(
        dispatched: super::super::DispatchedPhysicalWork,
        outcome: PhysicalExecutorOutcome,
        recovery: PhysicalEffectRecoveryObligation,
        residency_writeback: PhysicalResidencyWritebackCompletion,
    ) -> Self {
        Self {
            dispatched,
            outcome,
            recovery,
            residency_writeback: Some(residency_writeback),
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) const fn from_parts(
        dispatched: super::super::DispatchedPhysicalWork,
        outcome: PhysicalExecutorOutcome,
        recovery: PhysicalEffectRecoveryObligation,
        residency_writeback: Option<PhysicalResidencyWritebackCompletion>,
    ) -> Self {
        Self {
            dispatched,
            outcome,
            recovery,
            residency_writeback,
        }
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        super::super::DispatchedPhysicalWork,
        PhysicalExecutorOutcome,
        PhysicalEffectRecoveryObligation,
        Option<PhysicalResidencyWritebackCompletion>,
    ) {
        (
            self.dispatched,
            self.outcome,
            self.recovery,
            self.residency_writeback,
        )
    }
}

impl CompletedPhysicalPublicationEffect {
    pub(in crate::physical_runtime) const fn new(
        physical: CompletedArtifactTreePublicationEffect,
        artifact: RecordArtifactFile,
        effect: PhysicalPublicationEffect,
    ) -> Self {
        Self {
            physical,
            artifact,
            effect,
        }
    }

    pub const fn physical(&self) -> &CompletedArtifactTreePublicationEffect {
        &self.physical
    }

    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }

    pub const fn effect(&self) -> PhysicalPublicationEffect {
        self.effect
    }

    pub const fn recovery_target(&self) -> crate::physical_runtime::PhysicalWorkRecoveryTarget {
        publication_recovery_target(self.effect, self.artifact)
    }
}

impl IndeterminatePhysicalPublicationEffect {
    pub(in crate::physical_runtime) const fn new(
        physical: IndeterminateArtifactTreePublicationEffect,
        artifact: RecordArtifactFile,
        effect: PhysicalPublicationEffect,
    ) -> Self {
        Self {
            physical,
            artifact,
            effect,
        }
    }

    pub(in crate::physical_runtime) const fn physical(
        &self,
    ) -> &IndeterminateArtifactTreePublicationEffect {
        &self.physical
    }

    pub(in crate::physical_runtime) const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }

    pub(in crate::physical_runtime) const fn effect(&self) -> PhysicalPublicationEffect {
        self.effect
    }

    pub(in crate::physical_runtime) const fn recovery_target(
        &self,
    ) -> crate::physical_runtime::PhysicalWorkRecoveryTarget {
        publication_recovery_target(self.effect, self.artifact)
    }
}

impl CompletedPhysicalWalBarrier {
    pub(in crate::physical_runtime) const fn new(
        physical: CompletedArtifactTreePublicationEffect,
        artifact: worth_store_physical_backend::ArtifactTreeFile,
    ) -> Self {
        Self { physical, artifact }
    }

    pub const fn physical(&self) -> &CompletedArtifactTreePublicationEffect {
        &self.physical
    }

    pub const fn artifact(&self) -> &worth_store_physical_backend::ArtifactTreeFile {
        &self.artifact
    }
}

impl IndeterminatePhysicalWalBarrier {
    pub(in crate::physical_runtime) const fn new(
        physical: IndeterminateArtifactTreePublicationEffect,
        artifact: worth_store_physical_backend::ArtifactTreeFile,
    ) -> Self {
        Self { physical, artifact }
    }

    pub(in crate::physical_runtime) const fn physical(
        &self,
    ) -> &IndeterminateArtifactTreePublicationEffect {
        &self.physical
    }

    pub(in crate::physical_runtime) const fn artifact(
        &self,
    ) -> &worth_store_physical_backend::ArtifactTreeFile {
        &self.artifact
    }
}

const fn publication_recovery_target(
    effect: PhysicalPublicationEffect,
    artifact: RecordArtifactFile,
) -> crate::physical_runtime::PhysicalWorkRecoveryTarget {
    match effect {
        PhysicalPublicationEffect::SynchronizeArtifact => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(
                artifact,
            )
        }
        PhysicalPublicationEffect::SynchronizeArtifactParent => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(
                artifact,
            )
        }
        PhysicalPublicationEffect::ReplaceCatalog => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::CatalogReplacement(artifact)
        }
        PhysicalPublicationEffect::SynchronizeRecordFamily => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization
        }
    }
}
