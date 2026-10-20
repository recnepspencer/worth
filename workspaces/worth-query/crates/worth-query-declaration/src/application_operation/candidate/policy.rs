use super::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling,
};

/// Declaration policy for a candidate, independent of its actual reservation.
/// Absence imposes no intrinsic ceiling; the handler must still supply finite
/// requirements before candidate construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCandidatePolicy {
    cardinality: Option<ApplicationCandidateCardinalityCeiling>,
    resources: Option<ApplicationCandidateResourceCeiling>,
}

impl ApplicationCandidatePolicy {
    pub const fn new(
        cardinality: Option<ApplicationCandidateCardinalityCeiling>,
        resources: Option<ApplicationCandidateResourceCeiling>,
    ) -> Self {
        Self {
            cardinality,
            resources,
        }
    }

    /// A genuine finite policy, often also used by a fixed-shape handler.
    pub const fn finite(requirements: ApplicationCandidateRequirements) -> Self {
        Self::new(
            Some(requirements.cardinality()),
            Some(requirements.resources()),
        )
    }

    pub const fn cardinality(self) -> Option<ApplicationCandidateCardinalityCeiling> {
        self.cardinality
    }

    pub const fn resources(self) -> Option<ApplicationCandidateResourceCeiling> {
        self.resources
    }

    /// Fixed-shape handlers may explicitly request their complete finite policy.
    /// A measured-shape handler derives its requirements from its decision.
    pub const fn finite_requirements(self) -> Option<ApplicationCandidateRequirements> {
        match (self.cardinality, self.resources) {
            (Some(cardinality), Some(resources)) => Some(
                ApplicationCandidateRequirements::fixed_shape(cardinality, resources),
            ),
            _ => None,
        }
    }
}
