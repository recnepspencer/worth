//! World branch-history reads, classified by their own stop or refusal.
use worth_runtime_world::facade::{
    CompositeAncestorSelectionDenial as History, ProductBranchAncestorSelectionDenial as Ancestor,
    ProductExactHistoryDenial as Exact, RuntimeWorldBranchAdmissionDenial as Branch,
    RuntimeWorldServiceDenial as Service,
};

use super::correction::{HISTORY_GONE, OTHER_VERSION};
use super::request::{BUSY, CANCELLED, CAPACITY, INTERNAL, STORAGE, TIMED_OUT};
use super::summary::{WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary};

const VERSION_CLOSED: Summary = Summary::with_message(
    Category::NotFound,
    "This version of your data has been closed.",
);

pub(super) fn exact_history(cause: &Service<Exact>) -> Summary {
    match cause {
        Service::OwnerUnavailable(_) => BUSY,
        Service::Denied(Exact::Branch(cause)) => branch(cause),
        Service::Denied(Exact::History(cause)) => history(cause, HISTORY_GONE),
        Service::Denied(Exact::Request(_) | Exact::Allocation(_)) => CAPACITY,
        Service::Denied(Exact::InvalidContinuation) => INTERNAL,
    }
}

pub(super) fn ancestry(cause: &Service<Ancestor>) -> Summary {
    match cause {
        Service::OwnerUnavailable(_) => BUSY,
        Service::Denied(Ancestor::Branch(cause)) => branch(cause),
        Service::Denied(Ancestor::History(cause)) => history(cause, OTHER_VERSION),
    }
}

fn history(cause: &History, not_here: Summary) -> Summary {
    match cause {
        History::NotAncestor => not_here,
        History::SourceUnavailable
        | History::SelectionLimitReached
        | History::RetiredHistoryGap => HISTORY_GONE,
        History::TargetUnavailable => BUSY,
        History::Allocation(_) | History::Request(_) => CAPACITY,
        History::Catalog(_) => STORAGE,
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
