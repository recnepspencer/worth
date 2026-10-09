//! Move-only C8 WAL storage and its Store-issued native allocation lifetime.

mod candidate_storage;
mod resident_allocation;
mod source_selection;
mod tail_selection;

pub(crate) use candidate_storage::{
    CandidateClassificationDenial, RecordedWalDisposition, ResidentWalCandidates,
};
pub(crate) use source_selection::ResidentSourceSelection;
pub(crate) use tail_selection::{ResidentWalTail, WalTailSelectionDenial};
