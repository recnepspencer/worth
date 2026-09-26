//! Rows a settle moved without lowering them anew.
//!
//! A committed settle moves mounted geometry to the sample the host already
//! shows, and the host keeps drawing each row it moved through that sample,
//! from the command lowered where the row stood before. A frame that places
//! the row's group anew shows each command where it was lowered, so the next
//! change a frame lowers rows for lowers every row a settle carried as well,
//! where the row stands now.

use super::*;

impl UiMountedOccurrenceGeometryState {
    /// Record the rows `pose` moves as carried by a settle's sample.
    pub(crate) fn carry_by_sample(&mut self, pose: &UiPreparedMountedScrollPose) {
        let moved = pose.changed_instances();
        if moved.is_empty() {
            return;
        }
        self.sample_carried
            .entry(pose.surface())
            .or_default()
            .extend(moved.iter().copied());
    }

    /// The rows on `surface` a settle carried that the surface still holds.
    pub(crate) fn sample_carried(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Vec<UiMountedInstanceIdentity> {
        let (Some(carried), Some(geometry)) = (
            self.sample_carried.get(&surface),
            self.surfaces.get(&surface),
        ) else {
            return Vec::new();
        };
        carried
            .iter()
            .filter(|instance| geometry.occurrences.contains_key(instance))
            .copied()
            .collect()
    }

    /// Forget the rows carried on `surface`, once a frame lowers them anew.
    pub(crate) fn release_sample_carried(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.sample_carried.remove(&surface);
    }
}
