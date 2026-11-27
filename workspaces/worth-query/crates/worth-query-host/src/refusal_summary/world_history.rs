//! World branch-history reads, classified by their own stop or refusal.
use worth_runtime_world::facade::{
    CompositeAncestorSelectionDenial as History, ProductBranchAncestorSelectionDenial as Ancestor,
    ProductExactHistoryDenial as Exact, RuntimeWorldBranchAdmissionDenial as Branch,
    RuntimeWorldServiceDenial as Service,
};

use super::correction::{HISTORY_GONE, OTHER_VERSION};
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

/// Undo and correction selection: any history no longer kept reads as the
/// history needed to undo.
pub(super) fn exact_history(cause: &Service<Exact>) -> Summary {
    match cause {
        Service::OwnerUnavailable(_) => BUSY,
        Service::Denied(cause) => exact(cause, HISTORY_GONE),
    }
}

/// A record-history read's own walk stop: a retired gap is named apart.
pub(super) fn audit_history(cause: &Exact) -> Summary {
    match cause {
        Exact::History(History::RetiredHistoryGap) => RETIRED_HISTORY,
        cause => exact(cause, HISTORY_UNAVAILABLE),
    }
}

fn exact(cause: &Exact, gone: Summary) -> Summary {
    match cause {
        Exact::Branch(cause) => branch(cause),
        Exact::History(cause) => history(cause, gone.clone(), gone),
        Exact::Request(cause) => work(cause),
        Exact::Allocation(_) => CAPACITY,
        Exact::InvalidContinuation => INTERNAL,
    }
}

pub(super) fn ancestry(cause: &Service<Ancestor>) -> Summary {
    match cause {
        Service::OwnerUnavailable(_) => BUSY,
        Service::Denied(Ancestor::Branch(cause)) => branch(cause),
        Service::Denied(Ancestor::History(cause)) => history(cause, OTHER_VERSION, HISTORY_GONE),
    }
}

fn history(cause: &History, not_here: Summary, gone: Summary) -> Summary {
    match cause {
        History::NotAncestor => not_here,
        History::SourceUnavailable
        | History::SelectionLimitReached
        | History::RetiredHistoryGap => gone,
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
