use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::ProvisionalProvider;
use std::sync::OnceLock;
use worth_query_execution::facade::convergence_epoch::WorthQueryCandidateSemanticFamilies;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceAssessment;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceComparison;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceDomainFailure;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceDomainProvider;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceProgress;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceProviderFamilies;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceRepeatedState;
use worth_query_execution::facade::convergence_epoch::WorthQueryIterationSemanticFamilies;
impl WorthQueryConvergenceDomainProvider for ProvisionalProvider {
    fn convergence_families(&self) -> &WorthQueryConvergenceProviderFamilies {
        static FAMILIES: OnceLock<WorthQueryConvergenceProviderFamilies> = OnceLock::new();
        FAMILIES.get_or_init(|| {
            WorthQueryConvergenceProviderFamilies::new(
                WorthQueryCandidateSemanticFamilies::new(
                    "universe",
                    "termination",
                    "feasibility",
                    "comparison",
                    "incumbent",
                )
                .unwrap(),
                WorthQueryIterationSemanticFamilies::new(
                    "progress",
                    "comparator",
                    "repeated-state",
                )
                .unwrap(),
            )
        })
    }

    fn compare(
        &self,
        _assessment: &WorthQueryConvergenceAssessment<'_>,
    ) -> Result<WorthQueryConvergenceComparison, WorthQueryConvergenceDomainFailure> {
        Err(unused())
    }

    fn measure_progress(
        &self,
        _assessment: &WorthQueryConvergenceAssessment<'_>,
        _comparison: &WorthQueryConvergenceComparison,
    ) -> Result<WorthQueryConvergenceProgress, WorthQueryConvergenceDomainFailure> {
        Err(unused())
    }

    fn detect_repeated_state(
        &self,
        _assessment: &WorthQueryConvergenceAssessment<'_>,
        _comparison: &WorthQueryConvergenceComparison,
        _progress: WorthQueryConvergenceProgress,
    ) -> Result<WorthQueryConvergenceRepeatedState, WorthQueryConvergenceDomainFailure> {
        Err(unused())
    }
}
pub fn unused() -> WorthQueryConvergenceDomainFailure {
    WorthQueryConvergenceDomainFailure::new("combined capability fixture is installation-only")
}
