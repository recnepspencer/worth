//! World branch-history reads, classified by their own stop or refusal.
use worth_runtime_world::facade::{
    CompositeAncestorSelectionDenial as History, ProductExactHistoryDenial as Exact,
    RuntimeWorldBranchAdmissionDenial as Branch,
};

use super::request::{BUSY, CANCELLED, CAPACITY, INTERNAL, TIMED_OUT};
use super::stops::{catalog, work};
use super::summary::{WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary};

const VERSION_CLOSED: Summary = Summary::with_message(
    Category::NotFound,
    "This version of your data has been closed.",
);

/// Earlier steps were retired: the item existed, its older history is gone.
/// An audit read shows it apart from a missing item and every work stop.
pub(super) const RETIRED_HISTORY: Summary = Summary::with_message(
    Category::NotFound,
    "Older history for this item is no longer kept, so it can't be shown.",
);
/// An audit read's history source or selection is no longer available.
pub(super) const HISTORY_UNAVAILABLE: Summary = Summary::with_message(
    Category::NotFound,
    "The history of this item is no longer available.",
);

/// A record-history read's own walk stop: a retired gap is named apart.
pub(super) fn audit_history(cause: &Exact) -> Summary {
    match cause {
        Exact::History(History::RetiredHistoryGap) => RETIRED_HISTORY,
        cause => exact(cause),
    }
}

fn exact(cause: &Exact) -> Summary {
    match cause {
        Exact::Branch(cause) => branch(cause),
        Exact::History(cause) => history(cause),
        Exact::Request(cause) => work(cause),
        Exact::Allocation(_) => CAPACITY,
        Exact::InvalidContinuation => INTERNAL,
    }
}

fn history(cause: &History) -> Summary {
    match cause {
        History::NotAncestor
        | History::SourceUnavailable
        | History::SelectionLimitReached
        | History::RetiredHistoryGap => HISTORY_UNAVAILABLE,
        History::TargetUnavailable => BUSY,
        History::Allocation(_) => CAPACITY,
        History::Request(cause) => work(cause),
        History::Catalog(cause) => catalog(cause),
    }
}

fn branch(cause: &Branch) -> Summary {
    match cause {
        Branch::CancelledBeforeEffect => CANCELLED,
        Branch::DeadlineBeforeEffect => TIMED_OUT,
        Branch::RetiredBranch => VERSION_CLOSED,
        Branch::OwnerUnavailable | Branch::StaleIncarnation => BUSY,
        Branch::CapacityExhausted | Branch::IdentityExhausted => CAPACITY,
        _ => INTERNAL,
    }
}
