//! World owns the normalized checkpoint image and its complete graph.
pub const WORLD_CHECKPOINT_PROTOCOL: worth_foundational::facade::BoundaryProtocolIdentity =
    worth_foundational::facade::BoundaryProtocolIdentity::new("worth.world.checkpoint");
pub const WORLD_CHECKPOINT_VERSION: worth_foundational::facade::BoundaryProtocolVersion =
    worth_foundational::facade::BoundaryProtocolVersion::new(3);
mod acknowledgement;
mod coordinates;
mod coverage;
mod decoding;
mod encoding;
mod folding;
mod framing;
mod graph;
mod home_admission;
mod image;
mod origin;
#[cfg(test)]
pub(crate) use origin::WorldCheckpointOrigin;
pub(crate) use origin::{PreparedCheckpointOccurrence, WorldCheckpointOriginState};
pub use origin::{WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial};
mod original_observation;
mod original_state;
pub use original_state::WorldOriginalStateObservation;
mod port;
pub use original_observation::{WorldOriginalObservation, WorldOriginalObservationDenial};
mod program_activation;
pub use program_activation::{WorldProgramActivationHistory, WorldProgramActivationHistoryEntry};
mod source;
mod subject;
#[cfg(test)]
mod tests;
pub(in crate::durability) use acknowledgement::WorldCheckpointAcknowledgement;
pub(crate) use coverage::{WorldCheckpointCoverage, WorldJournalCoverage};
pub(crate) use image::{
    WorldBranchImageKey, WorldCheckpointImage, WorldImageHead, WorldOwnerRecordLocation,
};
pub use port::{PreparedWorldCheckpoint, WorldCheckpointPort};
pub use source::{
    WorldCheckpointJournalRecord, WorldCheckpointReadSource, WorldCheckpointReadView,
};
pub use subject::{WorldCheckpointSelection, WorldCheckpointSubject};
