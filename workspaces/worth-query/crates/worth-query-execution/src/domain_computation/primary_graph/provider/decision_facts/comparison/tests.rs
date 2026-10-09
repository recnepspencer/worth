use super::*;
use crate::domain_computation::{
    WorthQueryDecisionFactRequest, WorthQueryDecisionReadSetDenialKind,
};
use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn assert_comparison_scope_lifecycle(
        &self,
        authority: &WorthQuerySessionReadAuthority<'_>,
        requests: Vec<WorthQueryDecisionFactRequest>,
        request: &WorthQueryRequestScope,
        cancellation: &WorthQueryCancellationSource,
        foreign: &WorthQueryPrimaryGraphProvider,
    ) {
        let control =
            StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, Some(request));
        let count = || {
            self.graph
                .with_runtime(|runtime| runtime.retention().inspect_plan().active_snapshot_count)
        };
        let baseline = count();
        let capture = || {
            authority
                .capture_decision_read_set(requests.clone(), control.policy(), Some(request))
                .unwrap()
        };
        let generic = authority.compare_decision_read_set(capture()).unwrap();
        let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(generic) = generic else {
            panic!("real unmodified facts must compare fresh");
        };
        let scoped = self
            .compare_application_read_set(authority, capture(), control)
            .unwrap();
        let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(scoped) = scoped else {
            panic!("scoped comparison must agree with generic comparison");
        };
        assert_eq!(generic.counters(), scoped.counters());
        assert_eq!(
            count(),
            baseline,
            "normal comparison releases its sole observer"
        );
        let foreign_count = || {
            foreign
                .graph
                .with_runtime(|runtime| runtime.retention().inspect_plan().active_snapshot_count)
        };
        let foreign_baseline = foreign_count();
        assert_eq!(
            foreign
                .compare_application_read_set(authority, capture(), control)
                .err()
                .unwrap()
                .kind(),
            WorthQueryDecisionReadSetDenialKind::ProviderRejected
        );
        assert_eq!(
            foreign_count(),
            foreign_baseline,
            "foreign registration must be rejected before observer admission"
        );
        let receipt = capture();
        let mut comparison =
            PrimaryDecisionReadSetComparison::acquire(self, authority.session(), control).unwrap();
        assert_eq!(count(), baseline + 1);
        cancellation.cancel();
        let denied = authority
            .compare_decision_read_set_with(receipt, |evidence, admission| {
                comparison.compare(authority.session(), evidence, admission, control)
            })
            .err()
            .unwrap();
        assert_eq!(
            denied.kind(),
            WorthQueryDecisionReadSetDenialKind::RequestInterrupted(
                WorthQueryRequestInterruption::Cancelled
            )
        );
        drop(comparison);
        assert_eq!(
            count(),
            baseline,
            "interruption releases the exact comparison observer"
        );
        assert_eq!(
            PrimaryDecisionReadSetComparison::acquire(self, authority.session(), control)
                .err()
                .unwrap()
                .kind(),
            denied.kind()
        );
        assert_eq!(
            count(),
            baseline,
            "stopped admission must not create another observer"
        );
    }
}

mod indexed;
