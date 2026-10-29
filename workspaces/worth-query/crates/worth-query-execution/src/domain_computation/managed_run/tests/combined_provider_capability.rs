use std::sync::OnceLock;

use super::provisional_attempt_fixture::{ProvisionalProvider, ProvisionalProviderState};
use crate::domain_computation::{
    WorthQueryCandidateSemanticFamilies, WorthQueryConvergenceAssessment,
    WorthQueryConvergenceComparison, WorthQueryConvergenceDomainFailure,
    WorthQueryConvergenceDomainProvider, WorthQueryConvergenceProgress,
    WorthQueryConvergenceProviderFamilies, WorthQueryConvergenceRepeatedState,
    WorthQueryIterationSemanticFamilies,
};

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

#[test]
fn one_installed_anchor_retains_convergence() {
    let provider = ProvisionalProvider {
        state: std::sync::Arc::new(std::sync::Mutex::new(ProvisionalProviderState::default())),
    };
    let anchor = crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_convergent_invariant_capable::<
        super::ManagedGraph,
        _,
    >(provider);
    assert!(anchor.retains_convergence_and_phase_seven_through_ten());
}

fn unused() -> WorthQueryConvergenceDomainFailure {
    WorthQueryConvergenceDomainFailure::new("combined capability fixture is installation-only")
}
