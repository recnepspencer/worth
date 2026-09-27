//! An offset Scroll has staged past the frame the host shows.
//!
//! Direct input awaiting its frame and a layout awaiting presentation both
//! hold offsets the screen has not accepted. They pose mounted geometry for
//! the next frame and nothing else, so they carry their own type: the
//! accepted offset reads through `UiScrollRuntimeState::offset`, and a staged
//! one can neither stand in for it nor be compared with it. Only Scroll state
//! mints one, and only mounted pose staging reads the offset inside.

use super::UiScrollOffset;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiStagedScrollOffset(UiScrollOffset);

impl UiStagedScrollOffset {
    pub(super) const fn staged(offset: UiScrollOffset) -> Self {
        Self(offset)
    }

    /// The offset the next frame poses mounted geometry at.
    pub(crate) const fn staged_pose_offset(self) -> UiScrollOffset {
        self.0
    }
}
