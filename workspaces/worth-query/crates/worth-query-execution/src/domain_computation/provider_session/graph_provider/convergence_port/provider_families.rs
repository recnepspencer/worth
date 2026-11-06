use std::sync::Arc;

use worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract;

use super::identity_validation::portable_family;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCandidateSemanticFamilies {
    universe: Arc<str>,
    termination: Arc<str>,
    feasibility: Arc<str>,
    comparison: Arc<str>,
    incumbent: Arc<str>,
}

impl WorthQueryCandidateSemanticFamilies {
    pub fn new(
        universe: impl Into<Arc<str>>,
        termination: impl Into<Arc<str>>,
        feasibility: impl Into<Arc<str>>,
        comparison: impl Into<Arc<str>>,
        incumbent: impl Into<Arc<str>>,
    ) -> Result<Self, &'static str> {
        let families = Self {
            universe: universe.into(),
            termination: termination.into(),
            feasibility: feasibility.into(),
            comparison: comparison.into(),
            incumbent: incumbent.into(),
        };
        if ![
            families.universe.as_ref(),
            families.termination.as_ref(),
            families.feasibility.as_ref(),
            families.comparison.as_ref(),
            families.incumbent.as_ref(),
        ]
        .into_iter()
        .all(portable_family)
        {
            return Err("invalid-candidate-semantic-family");
        }
        Ok(families)
    }

    fn matches(&self, contract: &WorthQueryAdmittedConvergenceContract) -> bool {
        self.universe.as_ref() == contract.universe_family()
            && self.termination.as_ref() == contract.termination_family()
            && self.feasibility.as_ref() == contract.feasibility_family()
            && self.comparison.as_ref() == contract.comparison_family()
            && self.incumbent.as_ref() == contract.incumbent_family()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIterationSemanticFamilies {
    progress: Arc<str>,
    comparator: Arc<str>,
    repeated_state: Arc<str>,
}

impl WorthQueryIterationSemanticFamilies {
    pub fn new(
        progress: impl Into<Arc<str>>,
        comparator: impl Into<Arc<str>>,
        repeated_state: impl Into<Arc<str>>,
    ) -> Result<Self, &'static str> {
        let families = Self {
            progress: progress.into(),
            comparator: comparator.into(),
            repeated_state: repeated_state.into(),
        };
        if ![
            families.progress.as_ref(),
            families.comparator.as_ref(),
            families.repeated_state.as_ref(),
        ]
        .into_iter()
        .all(portable_family)
        {
            return Err("invalid-iteration-semantic-family");
        }
        Ok(families)
    }

    fn matches(&self, contract: &WorthQueryAdmittedConvergenceContract) -> bool {
        self.progress.as_ref() == contract.progress_measure_family()
            && self.comparator.as_ref() == contract.comparator_family()
            && self.repeated_state.as_ref() == contract.repeated_state_family()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConvergenceProviderFamilies {
    candidate: WorthQueryCandidateSemanticFamilies,
    iteration: WorthQueryIterationSemanticFamilies,
}

impl WorthQueryConvergenceProviderFamilies {
    pub const fn new(
        candidate: WorthQueryCandidateSemanticFamilies,
        iteration: WorthQueryIterationSemanticFamilies,
    ) -> Self {
        Self {
            candidate,
            iteration,
        }
    }

    pub(crate) fn matches(&self, contract: &WorthQueryAdmittedConvergenceContract) -> bool {
        self.candidate.matches(contract) && self.iteration.matches(contract)
    }
}
