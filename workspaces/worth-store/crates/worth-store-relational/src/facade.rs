//! Native-section and journal-adapter audience. Home lifecycle belongs to Application.
pub use crate::cold_source::RelationalStoreColdReadCounters;
pub use crate::recovered_history::RelationalStoreJournalRefusal;
pub use crate::recovered_transitions::RelationalStoreTransitionJournalRefusal;
pub use crate::resolution::RelationalStoreCommitFate;
pub use crate::section::{
    ClosingRelationalSection, PreparedRelationalSection, RelationalCheckpointSelectionProtection,
    RelationalCheckpointSelectionSource, RelationalSectionCheckpointDenial,
    RelationalSectionReadmissionDenial, RelationalStoreSection,
};
pub use crate::transition_resolution::RelationalStoreTransitionFate;
