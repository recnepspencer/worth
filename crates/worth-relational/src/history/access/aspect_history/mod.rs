//! Native finite change pages. Product ancestry remains the composition owner's job.

pub(crate) use worth_relational_branch::internal_port::history::access::aspect_history::cursor;
mod live_admission;
mod live_cursor;
pub(crate) use worth_relational_branch::internal_port::history::access::aspect_history::page;
pub(crate) use worth_relational_branch::internal_port::history::access::aspect_history::request_stage;
pub(crate) use worth_relational_branch::internal_port::history::access::aspect_history::retained_selection;
pub(crate) use worth_relational_branch::internal_port::history::access::aspect_history::selection;

pub use live_cursor::RelationalCanonicalRecordHistoryCursor;
pub use page::{
    RelationalPreparedRecordHistoryPage, RelationalRecordHistoryChange,
    RelationalRecordHistoryPage, RelationalRecordHistoryPageLimits,
};
pub use retained_selection::{
    RelationalRecordHistorySelection, RelationalRecordHistorySelectionLimits,
};
pub use selection::RelationalRecordHistorySubject;

pub use denial::RelationalRecordHistoryDenial;
pub(crate) use worth_relational_branch::internal_port::history::access::aspect_history::denial;
