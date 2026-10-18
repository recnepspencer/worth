//! Same-bound single-member controls distinguish a cumulative meter from resets.
use super::*;
use crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl;
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn assert_cumulative_currentness_for_test(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
        original: &WorthQueryRetainedQueryDependencies,
    ) {
        assert!(original.sources.len() > 1);
        let execution = phase
            .checked_execution_request_for(&self.application().product_runtime)
            .unwrap();
        let control = RequestAllocationControl::new(request, execution.allocation_policy());
        let installed = NonZeroUsize::new(
            self.application()
                .runtime
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .unwrap();
        let bound =
            self.application()
                .primary_provider
                .graph
                .with_runtime(&execution, |relational| {
                    let mut largest = 0;
                    for source in original.sources.elements() {
                        let mut native = self.request_admission(installed);
                        self.require_current_dependency_in(
                            &execution,
                            relational,
                            source,
                            control.policy(),
                            request,
                            &mut native,
                        )
                        .unwrap();
                        let actual = native.charged_work();
                        assert!(
                            actual > 0,
                            "every genuine source member pays native comparison work"
                        );
                        largest = largest.max(actual);
                    }
                    // This is the actual largest single native charge, not a forecast,
                    // mirrored algorithm, padded allowance or a subset labelled complete.
                    let bound = NonZeroUsize::new(usize::try_from(largest).unwrap()).unwrap();
                    for source in original.sources.elements() {
                        let mut native = self.request_admission(bound);
                        self.require_current_dependency_in(
                            &execution,
                            relational,
                            source,
                            control.policy(),
                            request,
                            &mut native,
                        )
                        .expect("each genuine member fits individually at the SAME finite bound");
                    }
                    bound
                });
        let refused = self
            .require_current_required_set(
                phase,
                &crate::publication_boundary::program_publication_access(),
                request,
                std::iter::empty(),
                original,
                bound,
            )
            .unwrap_err();
        assert!(
            matches!(refused,
            WorthQueryRequiredSetCurrentnessDenial::Source(ref cause) if cause.kind()==WorthQuerySourceExpectationDenialKind::WorkBudgetExceeded)
                || matches!(refused, WorthQueryRequiredSetCurrentnessDenial::Output(ref cause) if cause.kind()==WorthQueryOutputDemandDenialKind::WorkBudgetExceeded),
            "only exhausting the cumulative native meter is the expected refusal: {refused:?}"
        );
        self.require_current_required_set(
            phase,
            &crate::publication_boundary::program_publication_access(),
            request,
            std::iter::empty(),
            original,
            installed,
        )
        .expect("lawful retry retains the whole original proof");
    }
}
