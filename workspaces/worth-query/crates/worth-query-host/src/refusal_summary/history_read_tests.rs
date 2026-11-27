//! Record history, causal timeline and as-of snapshot refusals: a retired
//! history gap is distinct from every work, capacity, branch and
//! continuation stop of the same World walk.
use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_discovery::WorthQueryWorkflowReadDenial,
    primary_graph::WorthQueryApplicationHistoryDenial as History,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationHistorySelectionDenial, WorthQueryGovernedHistoryDenial,
    WorthQueryWorkflowObservationDenial,
};
use worth_runtime_world::facade::{
    CompositeAncestorSelectionDenial as Ancestor, ProductExactHistoryDenial as Exact,
};

use crate::audit::WorthQueryCausalTimelineDenial;
use crate::session::WorthQuerySessionHistoricalSnapshotDenial;

use super::super::operator_safety::assert_renders;
use super::super::summary::{WorthQueryRefusalCategory as Category, WorthQueryUserFacingRefusal};
use super::super::world_history::RETIRED_HISTORY;

fn world(cause: Exact) -> WorthQueryGovernedHistoryDenial {
    WorthQueryGovernedHistoryDenial::History(History::World(cause))
}

#[test]
fn a_retired_history_gap_classifies_apart_from_every_other_world_stop() {
    let gap = world(Exact::History(Ancestor::RetiredHistoryGap));
    assert!(gap.is_retired_history_gap());
    assert_eq!(gap.user_summary(), RETIRED_HISTORY);
    assert!(!gap.retryable());
    assert_renders(&gap, Category::NotFound);

    let ceiling = WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling);
    let deadline = WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed);
    let others = [
        (world(Exact::Request(ceiling)), Category::Capacity),
        (world(Exact::Request(deadline)), Category::Interrupted),
        (
            world(Exact::History(Ancestor::TargetUnavailable)),
            Category::Unavailable,
        ),
        (
            world(Exact::History(Ancestor::SelectionLimitReached)),
            Category::NotFound,
        ),
        (world(Exact::InvalidContinuation), Category::Internal),
        (
            WorthQueryGovernedHistoryDenial::History(History::MetadataDisclosureRefused),
            Category::PermissionDenied,
        ),
    ];
    for (other, category) in others {
        assert!(!other.is_retired_history_gap(), "{other:?}");
        assert_ne!(other.user_summary(), RETIRED_HISTORY, "{other:?}");
        assert_renders(&other, category);
    }

    let timeline = WorthQueryCausalTimelineDenial::Record(gap);
    assert!(timeline.is_retired_history_gap());
    assert_eq!(timeline.user_summary(), RETIRED_HISTORY);
    let workflow = WorthQueryCausalTimelineDenial::Workflow(
        WorthQueryWorkflowObservationDenial::Read(WorthQueryWorkflowReadDenial::Corrupt),
    );
    assert!(!workflow.is_retired_history_gap());
    assert_renders(&workflow, Category::Storage);
}

#[test]
fn an_as_of_snapshot_names_an_unknown_commit_apart_from_an_interrupted_pin() {
    assert_renders(
        &WorthQuerySessionHistoricalSnapshotDenial::History(
            WorthQueryApplicationHistorySelectionDenial::CommitUnavailable,
        ),
        Category::NotFound,
    );
    assert_renders(
        &WorthQuerySessionHistoricalSnapshotDenial::ExecutionRequest(
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled,
            ),
        ),
        Category::Interrupted,
    );
}
