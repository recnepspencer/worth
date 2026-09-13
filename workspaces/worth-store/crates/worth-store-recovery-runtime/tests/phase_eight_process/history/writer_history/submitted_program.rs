use std::path::PathBuf;

use super::super::schedule::ParentWriterDurabilityProfileSelection;
use super::expected_history::ExpectedWriterHistory;

#[derive(Debug, Clone)]
pub(crate) struct SubmittedOperationProgram {
    pub(crate) path: PathBuf,
    pub(crate) expected: ExpectedWriterHistory,
    pub(crate) writer_profile_selection: ParentWriterDurabilityProfileSelection,
}
