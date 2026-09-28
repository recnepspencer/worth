mod geometry;
mod range;

pub use geometry::ExtentArenaFrameLayout;
pub use range::{ExtentArenaId, ExtentArenaRange};

pub const EXTENT_ARENA_MANIFEST_FRAME_BYTES: usize = 104;

#[cfg(test)]
mod tests;
