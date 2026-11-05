mod cadence;
pub(crate) mod close_settlement;
mod input_memory;
pub(crate) mod inspection;
mod limits;
mod maintenance;
pub use cadence::ApplicationHomeCheckpointJournalObservation;
pub use inspection::ApplicationHomeCheckpointInspection;
pub use limits::ApplicationHomeCheckpointLimits;
pub use maintenance::ApplicationHomeCheckpointMaintenanceDenial;
pub(crate) mod journey;
mod manifest;
mod operation;
pub use operation::{
    ApplicationHomeCheckpointOperation, RetiredApplicationHomeCheckpointOperation,
};
pub(crate) mod owner;
mod physical_checkpoint;
mod physical_publication;
mod physical_start;
mod preparation;
mod selected_source;
pub(crate) use selected_source::SelectedHomeSource;
pub(crate) mod publication;
pub(crate) mod step_custody;

pub(crate) use manifest::SelectedHomeManifest;
pub use manifest::{HomeManifestDenial, HOME_CHECKPOINT_PROTOCOL, HOME_CHECKPOINT_VERSION};
pub use owner::ApplicationHomeCheckpointRecovery;
pub use publication::{ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial};

pub(crate) fn read(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<SelectedHomeManifest>, ApplicationHomeCheckpointDenial> {
    SelectedHomeSource::capture(serving, request)?.selected(serving, request)
}
