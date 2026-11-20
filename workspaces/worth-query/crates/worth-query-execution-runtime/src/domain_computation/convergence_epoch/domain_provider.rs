use super::{
    WorthQueryConvergenceAssessment, WorthQueryConvergenceComparison,
    WorthQueryConvergenceDomainFailure, WorthQueryConvergenceProgress,
    WorthQueryConvergenceProviderFamilies, WorthQueryConvergenceRepeatedState,
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
