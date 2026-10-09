//! Conservative retained charge for the selected V3 control observation.

use super::{Controls, SelectedArtifactSlice};

impl Controls {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin::pending_wal_release) fn retained_memory_bytes(
        &self,
    ) -> u64 {
        let payload_bytes = [
            self.descriptor.capacity(),
            self.reservation.capacity(),
            self.manifest.capacity(),
        ]
        .into_iter()
        .fold(0_u64, |sum, capacity| sum.saturating_add(capacity as u64));
        (std::mem::size_of::<Self>() as u64)
            .saturating_add(self.routes.retained_memory_bytes())
            .saturating_add(self.source_routes.retained_memory_bytes())
            .saturating_add(
                self.checkpoint_routes
                    .as_ref()
                    .map_or(0, |routes| routes.retained_memory_bytes()),
            )
            .saturating_add(
                self.selected_base
                    .as_ref()
                    .map_or(0, |base| base.retained_memory_bytes()),
            )
            .saturating_add(
                self.selected_head_v2_controls
                    .as_ref()
                    .map_or(0, |base| base.retained_memory_bytes()),
            )
            .saturating_add(payload_bytes)
            .saturating_add(
                (self.slices.capacity() as u64)
                    .saturating_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64),
            )
            .saturating_add(
                (self.checkpoint_slices.capacity() as u64)
                    .saturating_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64),
            )
    }
}
