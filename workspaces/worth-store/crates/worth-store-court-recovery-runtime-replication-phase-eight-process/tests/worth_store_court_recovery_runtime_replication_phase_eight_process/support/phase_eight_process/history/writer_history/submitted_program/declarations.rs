pub(crate) use std::path::PathBuf;

pub(crate) use crate::history::schedule::parent_writer_durability_profile_selection::ParentWriterDurabilityProfileSelection;
pub(crate) use crate::phase_eight_process::history::writer_history::expected_history::ExpectedWriterHistory;

#[derive(Debug, Clone)]
pub(crate) struct SubmittedOperationProgram {
    pub(crate) path: PathBuf,
    pub(crate) expected: ExpectedWriterHistory,
    pub(crate) writer_profile_selection: ParentWriterDurabilityProfileSelection,
}
