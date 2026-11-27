//! Audit history reads: a record's governed change log, the causal timeline
//! pairing it with a workflow audit trail, and as-of snapshot pinning.
//!
//! A retired history gap renders [`super::world_history::RETIRED_HISTORY`],
//! never the generic "history gone" or a capacity or work stop.
use worth_query_execution::facade::primary_graph::WorthQueryApplicationHistoryDenial as History;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationHistorySelectionDenial as Selection, WorthQueryGovernedHistoryDenial,
};
use worth_relational::facade::history::RelationalRecordHistoryDenial as Native;

use crate::audit::WorthQueryCausalTimelineDenial;
use crate::session::WorthQuerySessionHistoricalSnapshotDenial;

use super::request::{BUSY, CAPACITY, INTERNAL, PERMISSION};
use super::stops::{authorization, kernel, native, principal_resolution, work};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};
use super::world_history::{audit_history, HISTORY_UNAVAILABLE};

const HISTORY_TOO_LARGE: Summary = Summary::with_message(
    Category::Capacity,
    "This history is too long to show at once. Try a narrower view.",
);
const COMMIT_NOT_IN_HISTORY: Summary = Summary::with_message(
    Category::NotFound,
    "That earlier version isn't part of this history, or is no longer kept.",
);

impl WorthQueryUserFacingRefusal for WorthQueryGovernedHistoryDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Installation(_) => INTERNAL,
            Self::Request(cause) => cause.user_summary(),
            Self::History(cause) => record_history(cause),
        }
    }
}

/// The history walk's own stop: capture, metadata disclosure, open or page.
fn record_history(cause: &History) -> Summary {
    match cause {
        History::Query(cause) => cause.user_summary(),
        History::Read(cause) => cause.user_summary(),
        History::Authorization(denial) => authorization(&denial.kind()),
        History::Principal(denial) => principal_resolution(denial.kind()),
        History::Scope(cause) => cause.user_summary(),
        History::MetadataDisclosureRefused => PERMISSION,
        History::World(cause) => audit_history(cause),
        History::Native(cause) => native_history(cause),
        History::CaptureLimitExceeded | History::StorageLimitExceeded | History::SizeOverflow => {
            HISTORY_TOO_LARGE
        }
        History::Request(cause) => work(cause),
        History::Allocation(_) => CAPACITY,
        History::Selection(_) | History::BasisReleaseFailed => BUSY,
        History::SubjectMismatch | History::InvalidContinuation => INTERNAL,
    }
}

fn native_history(cause: &Native) -> Summary {
    match cause {
        Native::Storage(cause) => native(cause),
        Native::SubjectUnavailable | Native::CommitUnavailable => HISTORY_UNAVAILABLE,
        Native::Retention(_) => BUSY,
        Native::Request(cause) => work(cause),
        Native::Interrupted(stop) => kernel(stop),
        Native::Allocation(_)
        | Native::PageStorageTooSmall
        | Native::SizeOverflow
        | Native::SelectionLimitExceeded => HISTORY_TOO_LARGE,
        Native::ForeignOwner
        | Native::EmptySubjectSelection
        | Native::EmptyAspectSelection
        | Native::NotExactSelectedCommit
        | Native::InvalidContinuation => INTERNAL,
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryCausalTimelineDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Snapshot(cause) => cause.user_summary(),
            Self::Record(cause) => cause.user_summary(),
            Self::Workflow(cause) => cause.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQuerySessionHistoricalSnapshotDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::History(Selection::ProductSelection(_)) => BUSY,
            Self::History(Selection::CommitUnavailable) => COMMIT_NOT_IN_HISTORY,
        }
    }
}

#[cfg(test)]
#[path = "history_read_tests.rs"]
mod tests;
