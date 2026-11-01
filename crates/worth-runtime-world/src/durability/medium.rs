use super::{WorldDurableArtifact, WorldDurableFrameDenial};
use std::fmt::Debug;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{
    DurablePublicationFate, DurableWorldOwnerIdentity, HomeIncarnation,
};

#[derive(Clone, Debug)]
pub struct WorldDurableHomeBinding {
    owner: DurableWorldOwnerIdentity,
    incarnation: HomeIncarnation,
    program_revision: [u8; 32],
}
impl WorldDurableHomeBinding {
    /// Data provided by the trusted admitted adapter contract, not physical
    /// authentication. Only the restricted Home composition attaches it.
    pub fn from_admitted_adapter(
        owner: DurableWorldOwnerIdentity,
        incarnation: HomeIncarnation,
        program_revision: [u8; 32],
    ) -> Self {
        Self {
            owner,
            incarnation,
            program_revision,
        }
    }
    pub const fn owner(&self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub const fn incarnation(&self) -> HomeIncarnation {
        self.incarnation
    }
    pub const fn program_revision(&self) -> [u8; 32] {
        self.program_revision
    }
}
#[derive(Debug)]
pub enum WorldDurableMediumDenial {
    Allocation(worth_execution::ExecutionAllocationDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Ordered(worth_execution::ExecutionOrderedMapDenial),
    Request(WorkCeilingDenial),
    Frame(WorldDurableFrameDenial),
    NativeDescription(Box<worth_relational::facade::branch::RelationalRecordedComponentRefusal>),
    SignalDescription(worth_signal::facade::branch::SignalInitialEmptyRootRefusal),
    BridgeDescription(worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal),
    Unavailable,
    PublicationBusy,
    PublicationUnresolved,
    StaleCheckpoint,
    CheckpointSelectionRequired,
    Storage(String),
}
impl From<WorldDurableFrameDenial> for WorldDurableMediumDenial {
    fn from(cause: WorldDurableFrameDenial) -> Self {
        Self::Frame(cause)
    }
}
#[derive(Debug)]
pub enum WorldPublicationObservation {
    Pending,
    /// Selection is known, but the actual owner-issued committed carrier has
    /// not been readmitted. This does not describe an unknown physical fate.
    EvidenceRequired,
    Finished(DurablePublicationFate<WorldDurableMediumDenial>),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorldPublicationTaskPosture {
    Busy,
    Unsettled,
    Terminal,
}
/// Trusted owner port implemented by Store Application's private adapter.
/// An arbitrary implementation does not authenticate filesystem ownership.
pub trait RuntimeWorldDurableMedium: Debug + Send + Sync {
    fn binding(&self) -> &WorldDurableHomeBinding;
    /// A no-I/O snapshot of the actual retained task slots. Terminal includes
    /// terminal indeterminate custody: it permits teardown, never serving or
    /// a NoEffect conclusion. Home retains those tasks through physical close.
    fn publication_task_posture(&self) -> WorldPublicationTaskPosture;
    /// Acquire one actual protected physical W root without payload I/O.
    /// Called only while the actual durable publication reservation is held.
    /// The same original request funds metadata and nonwaiting admission.
    fn capture_checkpoint_source(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn super::checkpoint::WorldCheckpointReadSource>, WorldDurableMediumDenial>;
    /// Query/advance the actual retained publication for this original subject.
    /// Implementations must never pair a caller-supplied fate with another receipt.
    fn advance_publication(
        &self,
        subject: &WorldDurableArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> WorldPublicationObservation;
}
