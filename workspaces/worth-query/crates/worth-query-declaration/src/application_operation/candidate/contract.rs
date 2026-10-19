use super::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling,
};

/// Installed candidate preparation meaning, independent of a finite attempt footprint.
/// Neither this declaration nor the handler's footprint grants mutation authority.
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_operation::{ApplicationCandidateContract, ApplicationCandidateRequirements};
/// fn incomplete_footprint() -> ApplicationCandidateRequirements {
///     ApplicationCandidateContract::prepared()
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationCandidateContract {
    FixedShape(ApplicationCandidateRequirements),
    FixedCardinality(ApplicationCandidateCardinalityCeiling),
    Prepared,
}

impl ApplicationCandidateContract {
    pub const fn fixed_shape(
        cardinality: ApplicationCandidateCardinalityCeiling,
        resources: ApplicationCandidateResourceCeiling,
    ) -> Self {
        Self::FixedShape(ApplicationCandidateRequirements::fixed_shape(
            cardinality,
            resources,
        ))
    }

    /// Fixed maximum effect counts with input-dependent retained representation.
    pub const fn fixed_cardinality(cardinality: ApplicationCandidateCardinalityCeiling) -> Self {
        Self::FixedCardinality(cardinality)
    }

    /// The installed handler must prepare finite requirements before authoring.
    pub const fn prepared() -> Self {
        Self::Prepared
    }

    pub const fn cardinality(self) -> Option<ApplicationCandidateCardinalityCeiling> {
        match self {
            Self::FixedShape(requirements) => Some(requirements.cardinality()),
            Self::FixedCardinality(cardinality) => Some(cardinality),
            Self::Prepared => None,
        }
    }

    pub const fn fixed_requirements(self) -> Option<ApplicationCandidateRequirements> {
        match self {
            Self::FixedShape(requirements) => Some(requirements),
            Self::FixedCardinality(_) | Self::Prepared => None,
        }
    }
}
