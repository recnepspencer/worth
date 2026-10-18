//! Move the funded native position image into its unpublished expected root.
use super::PreparedRelationalBranchRootCapture;
use std::sync::Arc;

impl PreparedRelationalBranchRootCapture {
    pub(crate) fn bind_output_publication_positions(
        mut self,
        positions: crate::history::data::RequiredOutputPublicationPositions,
    ) -> Self {
        // The capture has not escaped its preparation phase or been installed.
        Arc::get_mut(&mut self.root)
            .expect("new root capture is exclusively owned")
            .output_publication_positions = positions;
        self
    }
}
