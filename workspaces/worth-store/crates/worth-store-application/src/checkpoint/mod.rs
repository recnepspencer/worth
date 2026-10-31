pub(crate) mod close_settlement;
mod input_memory;
pub(crate) mod journey;
mod manifest;
pub(crate) mod owner;
mod physical_checkpoint;
mod physical_publication;
mod physical_start;
mod preparation;
pub(crate) mod publication;
pub(crate) mod step_custody;

pub(crate) use manifest::SelectedHomeManifest;
pub use manifest::{HomeManifestDenial, HOME_CHECKPOINT_PROTOCOL, HOME_CHECKPOINT_VERSION};
pub use owner::ApplicationHomeCheckpointRecovery;
pub use publication::{ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial};

pub(crate) fn read(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> Result<Option<SelectedHomeManifest>, ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let anchor =
        worth_store::physical_runtime::AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR)
            .expect("static home checkpoint anchor");
    records
        .read(anchor, 1 << 30)
        .map_err(Denial::Read)?
        .map(SelectedHomeManifest::from_record)
        .transpose()
        .map_err(Denial::Manifest)
}
