use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceDisposition {
    Continue,
    Converged,
    StableWithoutProof,
    FeasibleIncumbent,
    Oscillating,
    Indeterminate,
}

impl WorthQueryConvergenceDisposition {
    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Converged => "converged",
            Self::StableWithoutProof => "stable-without-proof",
            Self::FeasibleIncumbent => "feasible-incumbent",
            Self::Oscillating => "oscillating",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceFeasibility {
    Feasible,
    Infeasible,
    Indeterminate,
}

impl WorthQueryConvergenceFeasibility {
    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Feasible => "feasible",
            Self::Infeasible => "infeasible",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceProgress {
    Advanced,
    Stable,
    Stalled,
    Indeterminate,
}

impl WorthQueryConvergenceProgress {
    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Advanced => "advanced",
            Self::Stable => "stable",
            Self::Stalled => "stalled",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceRepeatedState {
    Novel,
    Repeated,
    Indeterminate,
}

impl WorthQueryConvergenceRepeatedState {
    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Novel => "novel",
            Self::Repeated => "repeated",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceIncumbentUpdate {
    Retain,
    ReplaceWithCandidate,
    AddCandidate,
    RemoveCandidatesAndAdd {
        removed_occurrence_identities: Vec<Arc<str>>,
    },
    Clear,
}

impl WorthQueryConvergenceIncumbentUpdate {
    pub(crate) fn canonical_identity(&self) -> String {
        match self {
            Self::Retain => "retain".to_owned(),
            Self::ReplaceWithCandidate => "replace-with-candidate".to_owned(),
            Self::AddCandidate => "add-candidate".to_owned(),
            Self::RemoveCandidatesAndAdd {
                removed_occurrence_identities,
            } => {
                let mut identities = removed_occurrence_identities
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<_>>();
                identities.sort_unstable();
                format!("remove-and-add:{}", identities.join(","))
            }
            Self::Clear => "clear".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConvergenceDomainDecision {
    candidate_selection_key: Arc<str>,
    state_identity: Arc<str>,
    disposition: WorthQueryConvergenceDisposition,
    feasibility: WorthQueryConvergenceFeasibility,
    progress: WorthQueryConvergenceProgress,
    repeated_state: WorthQueryConvergenceRepeatedState,
    incumbent_update: WorthQueryConvergenceIncumbentUpdate,
}

impl WorthQueryConvergenceDomainDecision {
    pub(super) fn from_governed_assessment(
        comparison: super::comparison::WorthQueryConvergenceComparison,
        progress: WorthQueryConvergenceProgress,
        repeated_state: WorthQueryConvergenceRepeatedState,
    ) -> Self {
        let (candidate_selection_key, state_identity, disposition, feasibility, incumbent_update) =
            comparison.into_parts();
        Self {
            candidate_selection_key,
            state_identity,
            disposition,
            feasibility,
            progress,
            repeated_state,
            incumbent_update,
        }
    }

    pub fn candidate_selection_key(&self) -> &str {
        &self.candidate_selection_key
    }

    pub fn state_identity(&self) -> &str {
        &self.state_identity
    }

    pub const fn disposition(&self) -> WorthQueryConvergenceDisposition {
        self.disposition
    }

    pub const fn feasibility(&self) -> WorthQueryConvergenceFeasibility {
        self.feasibility
    }

    pub const fn progress(&self) -> WorthQueryConvergenceProgress {
        self.progress
    }

    pub const fn repeated_state(&self) -> WorthQueryConvergenceRepeatedState {
        self.repeated_state
    }

    pub fn incumbent_update(&self) -> &WorthQueryConvergenceIncumbentUpdate {
        &self.incumbent_update
    }
}
