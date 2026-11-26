mod allocation_generation;
mod chunk_frame;
mod geometry;
mod range;

pub use allocation_generation::{ArenaExtentFrameIdentity, ExtentAllocationGeneration};
pub use chunk_frame::ExtentChunkFrame;
pub use geometry::ExtentArenaFrameLayout;
pub use range::{ExtentArenaId, ExtentArenaRange};

pub const EXTENT_ARENA_MANIFEST_FRAME_BYTES: usize = 104;

#[cfg(test)]
mod tests;
