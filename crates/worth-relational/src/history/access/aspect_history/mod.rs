//! Native finite change pages. Product ancestry remains the composition owner's job.

mod cursor;
mod live_admission;
mod live_cursor;
mod page;
mod request_stage;
mod retained_selection;
mod selection;

pub use live_cursor::RelationalCanonicalRecordHistoryCursor;
pub use page::{
    RelationalPreparedRecordHistoryPage, RelationalRecordHistoryChange,
    RelationalRecordHistoryPage, RelationalRecordHistoryPageLimits,
};
pub use retained_selection::{
    RelationalRecordHistorySelection, RelationalRecordHistorySelectionLimits,
};
pub use selection::RelationalRecordHistorySubject;

mod denial;
pub use denial::RelationalRecordHistoryDenial;
