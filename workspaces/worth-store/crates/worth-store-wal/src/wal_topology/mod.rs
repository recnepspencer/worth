mod denial;
mod lsn;
mod lsn_range;
mod ordering_proof;
mod replay_cursor;
mod replay_cursor_admission;
mod scan;
mod segment_identity;
#[cfg(test)]
mod tests;

pub use denial::{WalTopologyDenial, WalTopologyDenialKind};
pub use lsn::LogSequenceNumber;
pub use lsn_range::WalLsnRange;
pub use ordering_proof::WalFrameOrderingProof;
pub use replay_cursor::{ReplayCursor, ReplayCursorSegment};
pub(crate) use replay_cursor_admission::admit_replay_cursor_segments;
pub use scan::{WalSegmentScanRecord, WalTopologyScan};
pub use segment_identity::{WalSegmentGeneration, WalSegmentId};
