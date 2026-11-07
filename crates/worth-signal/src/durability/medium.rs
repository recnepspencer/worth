//! Store-free owner port, also present on wasm32.
use super::{SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{
    ApplicationBuildIdentity, DurableSignalOwnerIdentity, SignalDurableJournal,
};
mod completion;
pub use completion::SignalPublicationCompletion;

#[derive(Clone, Debug)]
pub struct SignalDurableHomeBinding {
    owner: DurableSignalOwnerIdentity,
    build: ApplicationBuildIdentity,
}
impl SignalDurableHomeBinding {
    /// Trusted admitted adapter data, never a filesystem attachment witness.
    pub const fn from_admitted_adapter(
        owner: DurableSignalOwnerIdentity,
        build: ApplicationBuildIdentity,
    ) -> Self {
        Self { owner, build }
    }
    pub const fn owner(&self) -> DurableSignalOwnerIdentity {
        self.owner
    }
    pub const fn build_identity(&self) -> ApplicationBuildIdentity {
        self.build
    }
}
#[derive(Debug)]
pub enum SignalDurableMediumDenial {
    Request(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Frame(super::SignalDurableFrameDenial),
    ForeignOwner,
    Unavailable,
    PublicationBusy,
    PublicationUnresolved,
    Storage(String),
}
#[derive(Debug)]
pub enum SignalPublicationObservation {
    Pending,
    Finished(SignalPublicationCompletion),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalPublicationTaskPosture {
    Busy,
    Unsettled,
    Terminal,
}

/// Original protected source shared by both Signal journals. Reads cannot move
/// to a later physical root partway through owner reconstruction.
pub trait SignalDurableReadSource: std::fmt::Debug + Send + Sync {
    fn owner(&self) -> DurableSignalOwnerIdentity;
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, SignalDurableMediumDenial>;
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, SignalDurableMediumDenial>;
}
/// Owner artifacts are opaque to the adapter. Repeated calls advance the SAME
/// original publication; an unresolved task survives the invoking request.
pub trait SignalDurableMedium: std::fmt::Debug + Send + Sync {
    fn binding(&self) -> &SignalDurableHomeBinding;
    fn publication_task_posture(&self) -> SignalPublicationTaskPosture;
    fn capture_recovery_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn SignalDurableReadSource>, SignalDurableMediumDenial>;
    fn advance_definition(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation;
    fn advance_branch(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation;
    /// Quiesced owner teardown releases this exact serving share. It refuses
    /// unsettled physical work; it never infers absence from a stopped request.
    fn retire(&self, execution: &ExecutionRequest<'_, '_>)
        -> Result<(), SignalDurableMediumDenial>;
}
