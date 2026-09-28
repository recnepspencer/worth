//! The samples a frame's work displaces when it lands.
//!
//! Once the work lands the host retires the sample of every command its
//! delta touches and applies each sample it re-issues, sampled as the frame
//! was prepared, over any sample a tick showed beside the frame while it was
//! in flight. A group the frame places lands where the frame publishes it,
//! which its placement reconciles. Any other command the work touches or
//! re-samples would be drawn where the frame was prepared, while its group
//! and hit testing stand where the tick put them.

use std::collections::HashSet;
use std::rc::Rc;

use worth_ui_host_contract::{
    UiMountedPaintCommandChange, UiMountedPaintCommandIdentity, UiMountedPresentationSampleChange,
    UiMountedPresentationWorkView,
};

use super::super::UiMountedPresentationWork;
use super::UiMountedPresentationState;

impl UiMountedPresentationState {
    /// Record what `work`, re-issuing `samples`, displaces once it lands.
    /// A frame that displaces nothing records nothing, so it allocates
    /// nothing.
    pub(in crate::mounting::presentation) fn displace_samples(
        &mut self,
        work: &UiMountedPresentationWork,
        samples: &[UiMountedPresentationSampleChange],
    ) {
        let touched = match work.view() {
            UiMountedPresentationWorkView::Delta(delta) => delta.changes(),
            UiMountedPresentationWorkView::Initial(_)
            | UiMountedPresentationWorkView::Reconstruction(_)
            | UiMountedPresentationWorkView::Sample(_)
            | UiMountedPresentationWorkView::Unchanged(_) => &[],
        };
        let mut placed = None;
        let displaced = touched
            .iter()
            .flat_map(displayed_commands)
            .filter(|command| {
                !placed
                    .get_or_insert_with(|| self.placed_scroll_commands().collect::<HashSet<_>>())
                    .contains(command)
            })
            .chain(samples.iter().map(|sample| sample.command()))
            .collect::<HashSet<_>>();
        self.displaced_samples = (!displaced.is_empty()).then(|| Rc::new(displaced));
    }

    /// Whether the work presenting this frame displaces the sample `command`
    /// is shown through once it lands.
    pub(in crate::mounting::presentation) fn displaces_sample_of(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> bool {
        self.displaced_samples
            .as_ref()
            .is_some_and(|displaced| displaced.contains(&command))
    }

    /// Whether the work presenting this frame recorded nothing it displaces.
    #[cfg(test)]
    pub(in crate::mounting::presentation) fn records_no_displacement(&self) -> bool {
        self.displaced_samples.is_none()
    }
}

/// The commands a delta change touches that the displayed frame draws: the
/// host retires the sample of each. An insert names a command the displayed
/// frame does not draw, so no tick shows a sample of it.
fn displayed_commands(
    change: &UiMountedPaintCommandChange,
) -> Option<UiMountedPaintCommandIdentity> {
    match change {
        UiMountedPaintCommandChange::Insert(_) => None,
        UiMountedPaintCommandChange::Replace { predecessor, .. } => Some(*predecessor),
        UiMountedPaintCommandChange::Remove(identity) => Some(*identity),
    }
}
