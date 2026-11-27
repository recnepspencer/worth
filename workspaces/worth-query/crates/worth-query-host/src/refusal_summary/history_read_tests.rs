//! Audit and eligibility refusals classify by their own typed cause and
//! render one safe sentence: a retired history gap is distinct from every
//! work, capacity, branch and continuation stop of the same World walk.
use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_discovery::WorthQueryWorkflowReadDenial,
    eligibility::WorthQueryAbilityEligibilityDenial,
    primary_graph::WorthQueryApplicationHistoryDenial as History,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationHistorySelectionDenial, WorthQueryGovernedHistoryDenial,
    WorthQueryWorkflowObservationDenial,
};
use worth_runtime_world::facade::{
    CompositeAncestorSelectionDenial as Ancestor, ProductExactHistoryDenial as Exact,
};

use crate::audit::{
    WorthQueryCausalTimelineDenial, WorthQueryReferencedByDenial, WorthQuerySourceProvenanceDenial,
};
use crate::session::{WorthQueryEligibilityDenial, WorthQuerySessionHistoricalSnapshotDenial};

use super::super::summary::{WorthQueryRefusalCategory as Category, WorthQueryUserFacingRefusal};
use super::super::world_history::RETIRED_HISTORY;

fn world(cause: Exact) -> WorthQueryGovernedHistoryDenial {
    WorthQueryGovernedHistoryDenial::History(History::World(cause))
}

/// The message never echoes the typed refusal's `Debug` form or a type name.
fn assert_operator_safe(refusal: &(impl WorthQueryUserFacingRefusal + std::fmt::Debug)) {
    let message = refusal.user_message();
    let debug = format!("{refusal:?}");
    assert!(!message.is_empty(), "{debug}");
    assert!(
        !message.contains("WorthQuery") && !message.contains("::") && !message.contains('{'),
        "{debug} rendered {message:?}"
    );
    let variant = debug.split(['(', ' ', '{']).next().unwrap_or_default();
    assert!(!message.contains(variant), "{debug} rendered {message:?}");
}

#[test]
fn a_retired_history_gap_classifies_apart_from_every_other_world_stop() {
    let gap = world(Exact::History(Ancestor::RetiredHistoryGap));
    assert!(gap.is_retired_history_gap());
    assert_eq!(gap.user_summary(), RETIRED_HISTORY);
    assert!(!gap.retryable());
    assert_operator_safe(&gap);

    let others = [
        (
            world(Exact::Request(WorkCeilingDenial::Stopped(
                MapKernelStop::WorkCeiling,
            ))),
            Category::Capacity,
        ),
        (
            world(Exact::Request(WorkCeilingDenial::Stopped(
                MapKernelStop::DeadlineElapsed,
            ))),
            Category::Interrupted,
        ),
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
        assert_eq!(other.refusal_category(), category, "{other:?}");
        assert_ne!(other.user_summary(), RETIRED_HISTORY, "{other:?}");
        assert_operator_safe(&other);
    }

    let timeline = WorthQueryCausalTimelineDenial::Record(gap);
    assert!(timeline.is_retired_history_gap());
    assert_eq!(timeline.user_summary(), RETIRED_HISTORY);
    let workflow = WorthQueryCausalTimelineDenial::Workflow(
        WorthQueryWorkflowObservationDenial::Read(WorthQueryWorkflowReadDenial::Corrupt),
    );
    assert!(!workflow.is_retired_history_gap());
    assert_eq!(workflow.refusal_category(), Category::Storage);
}

#[test]
fn every_audit_and_eligibility_refusal_renders_a_typed_operator_safe_summary() {
    let cancelled = || {
        WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        )
    };
    let eligibility = |cause| WorthQueryEligibilityDenial::Observation(cause);
    let cases: [(Box<dyn Case>, Category); 9] = [
        (
            Box::new(WorthQuerySourceProvenanceDenial::ForeignBranch),
            Category::NotFound,
        ),
        (
            Box::new(WorthQueryReferencedByDenial::NotAnIncomingReference {
                query: "ForwardQuery".to_owned(),
            }),
            Category::Internal,
        ),
        (
            Box::new(WorthQuerySessionHistoricalSnapshotDenial::History(
                WorthQueryApplicationHistorySelectionDenial::CommitUnavailable,
            )),
            Category::NotFound,
        ),
        (
            Box::new(WorthQuerySessionHistoricalSnapshotDenial::ExecutionRequest(
                cancelled(),
            )),
            Category::Interrupted,
        ),
        (
            Box::new(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::ContinuationMismatch,
            )),
            Category::Conflict,
        ),
        (
            Box::new(eligibility(
                WorthQueryAbilityEligibilityDenial::RosterNotDisclosed {
                    disclosure: "ViewAuditors",
                },
            )),
            Category::PermissionDenied,
        ),
        (
            Box::new(eligibility(
                WorthQueryAbilityEligibilityDenial::CapabilityNotInstalled {
                    capability: "AuditCapability",
                },
            )),
            Category::Internal,
        ),
        (
            Box::new(eligibility(
                WorthQueryAbilityEligibilityDenial::CapabilityRequiresElevation {
                    capability: "AuditCapability",
                },
            )),
            Category::Internal,
        ),
        (
            Box::new(WorthQueryEligibilityDenial::ExecutionRequest(cancelled())),
            Category::Interrupted,
        ),
    ];
    for (refusal, category) in cases {
        assert_eq!(refusal.category(), category, "{:?}", refusal.debug());
        refusal.check();
    }
}

/// One boxed refusal of any audit or eligibility denial type.
trait Case {
    fn category(&self) -> Category;
    fn debug(&self) -> String;
    fn check(&self);
}

impl<T: WorthQueryUserFacingRefusal + std::fmt::Debug> Case for T {
    fn category(&self) -> Category {
        self.refusal_category()
    }
    fn debug(&self) -> String {
        format!("{self:?}")
    }
    fn check(&self) {
        assert_operator_safe(self);
    }
}
