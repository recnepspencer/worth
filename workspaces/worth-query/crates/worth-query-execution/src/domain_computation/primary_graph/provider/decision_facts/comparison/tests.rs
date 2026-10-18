use super::*;
use crate::domain_computation::WorthQueryDecisionReadSetDenialKind;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn assert_comparison_scope_lifecycle(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        request: &WorthQueryRequestScope,
        cancellation: &WorthQueryCancellationSource,
        foreign: &WorthQueryPrimaryGraphProvider,
    ) {
        let control =
            StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, Some(request));
        let (table, _) = self
            .attempts
            .lock()
            .unwrap()
            .shared_observed_facts(authority.session())
            .unwrap();
        let requests: Vec<_> = table.requests().collect();
        funding::assert_full_capture_funding(self, execution, authority, request, &table);
        let count = || {
            self.graph.with_runtime(execution, |runtime| {
                runtime.retention().inspect_plan().active_snapshot_count
            })
        };
        let baseline = count();
        let capture = || {
            authority
                .capture_decision_read_set(requests.clone(), control.policy(), Some(request))
                .unwrap()
        };
        let generic = authority
            .compare_decision_read_set(execution, capture())
            .unwrap();
        let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(generic) = generic else {
            panic!("real unmodified facts must compare fresh");
        };
        let scoped = self
            .compare_application_read_set(
                execution,
                authority,
                self.capture_application_read_set(execution, authority, control)
                    .unwrap(),
                control,
            )
            .unwrap();
        let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(scoped) = scoped else {
            panic!("scoped comparison must agree with generic comparison");
        };
        assert_eq!(generic.counters(), scoped.counters());
        for fact in table.values() {
            assert!(scoped.contains_locator(&fact.locator_identity()));
        }
        assert!(!scoped.contains_locator("unregistered-same-count-substitution"));
        let mut wrong_keys = requests.clone();
        let family = wrong_keys[0].family_identity().to_owned();
        wrong_keys[0] = crate::domain_computation::WorthQueryDecisionFactRequest::new(
            family,
            crate::domain_computation::WorthQueryDecisionFactLocator::structural_proof(
                "unregistered-same-count-substitution",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            authority
                .capture_decision_read_set(wrong_keys, control.policy(), Some(request))
                .err()
                .unwrap()
                .kind(),
            WorthQueryDecisionReadSetDenialKind::ProviderRejected
        );
        let recomparison = self
            .recompare_application_read_set(execution, authority, scoped, control)
            .unwrap();
        assert!(matches!(
            recomparison,
            WorthQueryDecisionReadSetFreshnessOutcome::Fresh(_)
        ));
        drop(recomparison);
        assert_eq!(
            count(),
            baseline,
            "normal comparison releases its sole observer"
        );
        let foreign_count = || {
            foreign.graph.with_runtime(execution, |runtime| {
                runtime.retention().inspect_plan().active_snapshot_count
            })
        };
        let foreign_baseline = foreign_count();
        assert_eq!(
            foreign
                .compare_application_read_set(execution, authority, capture(), control)
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
        let mut comparison = PrimaryDecisionReadSetComparison::acquire(
            execution,
            self,
            authority.session(),
            control,
        )
        .unwrap();
        assert_eq!(count(), baseline + 1);
        cancellation.cancel();
        let denied = authority
            .compare_decision_read_set_with(receipt, |evidence, admission| {
                comparison.compare(execution, authority.session(), evidence, admission, control)
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
            PrimaryDecisionReadSetComparison::acquire(
                execution,
                self,
                authority.session(),
                control
            )
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

mod funding;
mod indexed;

mod principal_staleness;
