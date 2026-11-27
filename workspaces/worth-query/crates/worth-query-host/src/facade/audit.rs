//! Audit time-travel, causal timelines, source provenance and impact reads.
//!
//! Start from a session: `session.snapshot_at_commit(..)` (or
//! `snapshot_at_recorded_commit`, `snapshot_at_original_receipt`) pins an
//! earlier commit; `record_history::<History>(..)`, `workflow_audit::<..>(..)`
//! and `causal_timeline::<..>(..)` read a record's governed change log and its
//! workflow transitions at a pin; `session.source_provenance::<Disclosure>(..)`
//! explains what an authored operation wrote. Every read admits the session
//! principal afresh and refuses rather than returning less than it may see.
pub use crate::audit::*;
pub use crate::session::WorthQuerySessionHistoricalSnapshotDenial;
pub use worth_query_publication::facade::application_entry::{
    WorthQueryGovernedHistoryCompleteness, WorthQueryGovernedHistoryDenial,
    WorthQueryGovernedHistoryLimits, WorthQueryGovernedReadAccess,
};
