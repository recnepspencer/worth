use crate::{
    LatchAcquisitionDenial, ManifestEpoch, PhysicalReadProtectedFootprintBasis, RootEpoch,
};
use worth_store_recovery_physics::PhysicalRecoveryResidueKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionReadInterlockDenial {
    EmptyCandidateRangeSet,
    QuarantinedCandidateRange,
    SourceEvidenceMismatch,
    StaleCompactionSourceEpoch {
        expected: RootEpoch,
        observed: RootEpoch,
    },
    InPlaceOverwriteOfProtectedStructure,
    MissingOldRootPreservation,
    BackendResidueCandidateSelection(PhysicalRecoveryResidueKind),
    EarlyReclaimBeforeReadRelease {
        protected: PhysicalReadProtectedFootprintBasis,
    },
    ExpectedMutationLaneDenialNotProduced,
    StaleEpochReuse {
        source_epoch: RootEpoch,
        reused_epoch: RootEpoch,
    },
    StaleManifestEpochReuse {
        source_epoch: ManifestEpoch,
        reused_epoch: ManifestEpoch,
    },
    PublicationRootMismatch,
    PublicationReachabilityFootprintMismatch {
        protected: PhysicalReadProtectedFootprintBasis,
        preserved: PhysicalReadProtectedFootprintBasis,
    },
    LatchAcquisition(LatchAcquisitionDenial),
    MixedRootDuringCompaction,
    PreCutoverReadReceiptMismatch,
    PostCutoverReadReceiptMismatch,
    LsmTombstoneRetentionMissing,
    LsmPublicationBindingMissing,
    LsmCounterBindingMismatch,
    LsmPhysicalTargetMismatch,
}
