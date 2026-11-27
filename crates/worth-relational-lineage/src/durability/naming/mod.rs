//! Versioned durable group lineage frames: one canonical frame per commit
//! that names groups, carried in its commit envelope and readmitted on
//! checkpoint restore and Journal replay. Record births and lineage events
//! persist in the commit envelope and checkpoint lineage pages themselves.

pub mod format;
pub mod group_frame_decoding;
pub mod group_frame_encoding;
#[cfg(test)]
mod group_frame_tests;
pub mod refusal;
pub mod wire;

pub use group_frame_decoding::decode_group_lineage_frame;
pub use group_frame_encoding::encode_group_lineage_frame;
pub use refusal::{PersistentNameDurabilityError, PersistentNameWireSection};
