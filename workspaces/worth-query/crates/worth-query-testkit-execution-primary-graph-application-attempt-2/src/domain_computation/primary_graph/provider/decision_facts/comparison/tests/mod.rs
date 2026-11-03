use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::decision_facts::comparison::PrimaryDecisionReadSetComparison;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use worth_query_execution::facade::domain_computation::WorthQuerySessionReadAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetDenialKind;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

/// Test doors on a library type, owned by the moved courts.
pub trait TestsWorthQueryPrimaryGraphProviderForTest: Sized {
    fn assert_comparison_scope_lifecycle(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        request: &WorthQueryRequestScope,
        cancellation: &WorthQueryCancellationSource,
        foreign: &WorthQueryPrimaryGraphProvider,
    );
}

impl TestsWorthQueryPrimaryGraphProviderForTest for WorthQueryPrimaryGraphProvider {
    fn assert_comparison_scope_lifecycle(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        request: &WorthQueryRequestScope,
        cancellation: &WorthQueryCancellationSource,
        foreign: &WorthQueryPrimaryGraphProvider,
    ) {
        let control =
            StorageControl::new_for_test(ExecutionAllocationPolicy::SystemAllocation, Some(request));
        let (table, _) = (*self
            .attempts_for_test())
            .lock()
            .unwrap()
            .shared_observed_facts_for_test(authority.session_for_test())
            .unwrap();
        let requests: Vec<_> = table.requests().collect();
        crate::domain_computation::primary_graph::provider::decision_facts::comparison::tests::funding::assert_full_capture_funding(self, execution, authority, request, &table);
        let count = || {
            (*self.graph_for_test()).with_runtime(execution, |runtime| {
                runtime.retention().active_snapshot_count()
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
            .compare_application_read_set_for_test(
                execution,
                authority,
                self.capture_application_read_set_for_test(execution, authority, control)
                    .unwrap(),
                control,
            )
            .unwrap();
        let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(scoped) = scoped else {
            panic!("scoped comparison must agree with generic comparison");
        };
        assert_eq!(generic.counters(), scoped.counters());
        for fact in table.values() {
            assert!(scoped.contains_locator_for_test(&fact.locator_identity()));
        }
        assert!(!scoped.contains_locator_for_test("unregistered-same-count-substitution"));
        let mut wrong_keys = requests.clone();
        let family = wrong_keys[0].family_identity().to_owned();
        wrong_keys[0] = worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequest::new(
            family,
            worth_query_execution::facade::domain_computation::WorthQueryDecisionFactLocator::structural_proof(
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
            .recompare_application_read_set_for_test(execution, authority, scoped, control)
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
            (*foreign.graph_for_test()).with_runtime(execution, |runtime| {
                runtime.retention().active_snapshot_count()
            })
        };
        let foreign_baseline = foreign_count();
        assert_eq!(
            foreign
                .compare_application_read_set_for_test(execution, authority, capture(), control)
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
        let mut comparison = PrimaryDecisionReadSetComparison::acquire_for_test(
            execution,
            self,
            authority.session_for_test(),
            control,
        )
        .unwrap();
        assert_eq!(count(), baseline + 1);
        cancellation.cancel();
        let denied = authority
            .compare_decision_read_set_with_for_test(receipt, |evidence, admission| {
                comparison.compare_for_test(execution, authority.session_for_test(), evidence, admission, control)
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
            PrimaryDecisionReadSetComparison::acquire_for_test(
                execution,
                self,
                authority.session_for_test(),
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

#[path = "funding/mod.rs"]
pub mod funding;
#[path = "principal_staleness/mod.rs"]
pub mod principal_staleness;
