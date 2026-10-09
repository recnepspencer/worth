use worth_proof::ProofOutcome;

use crate::physical_runtime::durability::checkpoint::PhysicalCheckpointHandle;

pub type PhysicalCheckpointStartOutcome = ProofOutcome<
    PhysicalCheckpointHandle,
    PhysicalCheckpointStartDenial,
    PhysicalCheckpointStartDeferred,
    PhysicalCheckpointStartStale,
    PhysicalCheckpointStartRebindRequired,
    PhysicalCheckpointStartFailure,
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCheckpointStartDenial {
    NoDurableWalSource,
    DeadlineElapsed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCheckpointStartDeferred {
    CaptureAlreadyActive,
    ResidencyUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCheckpointStartStale {
    RuntimeClosing,
    WorkOwnerReleased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCheckpointStartRebindRequired {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalCheckpointStartFailure {
    Capture(crate::physical_runtime::PhysicalCheckpointCaptureFailureKind),
    WorkerSpawnFailed,
}
