//! Closed reasons that checkpoint description cannot become owner evidence.
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolUnsupportedVersion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalCheckpointAdmissionRefusal {
    MissingArtifact,
    PredecessorFormat {
        protocol: BoundaryProtocolIdentity,
        version: BoundaryProtocolUnsupportedVersion,
    },
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    InvalidVersion,
    Malformed,
    BindingMismatch(CheckpointAdmissionBindingMismatch),
    AuthenticationMismatch(CheckpointAdmissionAuthenticationMismatch),
    MissingRootImage,
    InvalidProjectedMeaning,
    MissingCaptureOrigin,
    OwnerUnavailable,
    CapacityOverflow,
    Request(crate::execution::RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Buffer(worth_execution::ExecutionRequestArrayDenial),
    Native(crate::durability::data::DurabilityError),
    Content(crate::content::RelationalContentDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointAdmissionBindingMismatch {
    Scope,
    Home,
    NativeOrigin,
    Cut,
    Source,
    Checkpoint,
    WorldImage,
    Registry,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointAdmissionAuthenticationMismatch {
    Directory,
    Meaning {
        position: u64,
        meaning: CheckpointAdmissionMeaning,
    },
    CaptureOrigin,
}

/// Each role is authenticated independently of the original canonical leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CheckpointAdmissionMeaning {
    Semantic = 1,
    Axes = 2,
    Association = 3,
    Coordinates = 4,
    Schema = 5,
    Visibility = 6,
    Creations = 7,
    Outputs = 8,
    RootDirectory = 9,
}

pub(super) const CANONICAL_MEANINGS: [CheckpointAdmissionMeaning; 8] = [
    CheckpointAdmissionMeaning::Semantic,
    CheckpointAdmissionMeaning::Axes,
    CheckpointAdmissionMeaning::Association,
    CheckpointAdmissionMeaning::Coordinates,
    CheckpointAdmissionMeaning::Schema,
    CheckpointAdmissionMeaning::Visibility,
    CheckpointAdmissionMeaning::Creations,
    CheckpointAdmissionMeaning::Outputs,
];

pub(crate) fn native_stop(
    cause: crate::durability::data::RelationalRecoveryStop,
) -> RelationalCheckpointAdmissionRefusal {
    use crate::durability::data::RelationalRecoveryStop as Stop;
    match cause {
        Stop::AdmissionImage(cause) => cause,
        Stop::Content(cause) => RelationalCheckpointAdmissionRefusal::Content(cause),
        Stop::Request(cause) => RelationalCheckpointAdmissionRefusal::Request(cause),
        Stop::Allocation(cause) => RelationalCheckpointAdmissionRefusal::Allocation(cause),
        Stop::Durability(cause) => RelationalCheckpointAdmissionRefusal::Native(cause),
    }
}
