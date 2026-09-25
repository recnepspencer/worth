/// The newest positive client extent the window reported that no presentation
/// target has been prepared for yet.
///
/// A border drag reports far more extents than frames can be prepared for.
/// Each observation replaces the one before it, so preparing a target costs
/// one allocation per dispatch turn, whatever the raw event count. A zero
/// extent is never held here: suspension is a lifecycle transition, applied
/// when it is observed, and it discards the extent it supersedes.
#[derive(Debug, Default)]
pub(in crate::native::event_loop) struct UiNativePendingResize {
    latest: Option<[u32; 2]>,
}

/// How an observed client extent is handled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::native::event_loop) enum UiNativeResizeAdmission {
    /// The extent waits for the next dispatch turn to prepare its target.
    Pending,
    /// The extent suspends presentation and is applied now.
    Suspend([u32; 2]),
}

impl UiNativePendingResize {
    pub(in crate::native::event_loop) fn observe(
        &mut self,
        extent: [u32; 2],
    ) -> UiNativeResizeAdmission {
        if extent.contains(&0) {
            self.latest = None;
            return UiNativeResizeAdmission::Suspend(extent);
        }
        self.latest = Some(extent);
        UiNativeResizeAdmission::Pending
    }

    /// Takes the extent to prepare a target for, leaving nothing pending.
    pub(in crate::native::event_loop) fn take(&mut self) -> Option<[u32; 2]> {
        self.latest.take()
    }

    /// Drops the pending extent because a transition that reads the window's
    /// current extent itself, such as a scale change, supersedes it.
    pub(in crate::native::event_loop) fn supersede(&mut self) {
        self.latest = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{UiNativePendingResize, UiNativeResizeAdmission};

    #[test]
    fn a_drag_prepares_only_the_newest_extent_once() {
        let mut pending = UiNativePendingResize::default();
        for width in 900..1_000 {
            assert_eq!(
                pending.observe([width, 700]),
                UiNativeResizeAdmission::Pending
            );
        }
        assert_eq!(pending.take(), Some([999, 700]));
        assert_eq!(pending.take(), None);
    }

    #[test]
    fn a_zero_extent_suspends_now_and_discards_the_extent_it_supersedes() {
        let mut pending = UiNativePendingResize::default();
        pending.observe([1_200, 800]);
        assert_eq!(
            pending.observe([0, 0]),
            UiNativeResizeAdmission::Suspend([0, 0])
        );
        assert_eq!(pending.take(), None);
        pending.observe([1_200, 800]);
        pending.supersede();
        assert_eq!(pending.take(), None);
    }
}
