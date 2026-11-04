//! World owns the normalized checkpoint image and its complete graph.
pub const WORLD_CHECKPOINT_PROTOCOL: worth_foundational::facade::BoundaryProtocolIdentity =
    worth_foundational::facade::BoundaryProtocolIdentity::new("worth.world.checkpoint");
pub const WORLD_CHECKPOINT_VERSION: worth_foundational::facade::BoundaryProtocolVersion =
    worth_foundational::facade::BoundaryProtocolVersion::new(1);
mod acknowledgement;
mod coordinates;
mod coverage;
mod decoding;
mod encoding;
mod folding;
mod framing;
mod graph;
mod image;
mod origin;
pub(crate) use origin::{
    PreparedCheckpointOccurrence, WorldCheckpointOrigin, WorldCheckpointOriginState,
};
pub use origin::{WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial};
mod port;
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
