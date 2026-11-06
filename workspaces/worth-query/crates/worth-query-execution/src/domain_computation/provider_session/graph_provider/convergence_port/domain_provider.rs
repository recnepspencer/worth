use {
    super::comparison::WorthQueryConvergenceComparison,
    super::domain_assessment::WorthQueryConvergenceAssessment,
    super::domain_decision::WorthQueryConvergenceProgress,
    super::domain_decision::WorthQueryConvergenceRepeatedState,
    super::domain_work::WorthQueryConvergenceDomainFailure,
    super::provider_families::WorthQueryConvergenceProviderFamilies,
};

pub trait WorthQueryConvergenceDomainProvider: Send + Sync + 'static {
    fn convergence_families(&self) -> &WorthQueryConvergenceProviderFamilies;

    fn compare(
        &self,
        assessment: &WorthQueryConvergenceAssessment<'_>,
    ) -> Result<WorthQueryConvergenceComparison, WorthQueryConvergenceDomainFailure>;

    fn measure_progress(
        &self,
        assessment: &WorthQueryConvergenceAssessment<'_>,
        comparison: &WorthQueryConvergenceComparison,
    ) -> Result<WorthQueryConvergenceProgress, WorthQueryConvergenceDomainFailure>;

    fn detect_repeated_state(
        &self,
        assessment: &WorthQueryConvergenceAssessment<'_>,
        comparison: &WorthQueryConvergenceComparison,
        progress: WorthQueryConvergenceProgress,
    ) -> Result<WorthQueryConvergenceRepeatedState, WorthQueryConvergenceDomainFailure>;
}
