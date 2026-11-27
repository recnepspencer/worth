//! Same-Native-position actor edits may change equality meaning during a probe.

use super::*;

impl FullVerificationImage<'_> {
    /// The oracle's fence: the rows its walk read must mean the same in the
    /// re-read image. Only this consistency check is shared with the
    /// equality walk (`semantic_fence`); the verdict stays the oracle's own.
    pub fn fence_semantic_image(
        &self,
        observed: &OrdSet<Arc<RecordedSettlementIdentity>>,
    ) -> Result<(), FullVerificationStop> {
        let cell = self
            .owner
            .cell_for_read(self.selected)?
            .ok_or(FullVerificationStop::ActorImageChanged)?;

        let image = cell.read_image();

        let current = SnapshotAlignedMarkState::select_image(image, self.selected)
            .map_err(FullVerificationStop::Alignment)?
            .into_full_verification_state();

        if !super::super::semantic_fence::same_observed_meaning(
            &self.state,
            &current,
            observed,
            self.selected,
        ) {
            return Err(FullVerificationStop::ActorImageChanged);
        }
        Ok(())
    }
}

mod court_doors_for_test;
