use super::coverage_basis::AdmittedCoverageBasis;
use super::denial::{CoverageGapWitness, MaterializationDenial};
use super::state::{LayoutMaterializationState, MaterializationStateClass};
use super::watermark::LayoutWatermark;
use super::LayoutMaterializationSourceIdentity;
use crate::catalog::PhysicalArtifactFamily;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutCoverageWitness {
    family: PhysicalArtifactFamily,
    state: LayoutMaterializationState,
    lower_bound: LayoutWatermark,
    upper_bound: LayoutWatermark,
    gap: Option<CoverageGapWitness>,
    source: LayoutMaterializationSourceIdentity,
}

impl LayoutCoverageWitness {
    fn new(
        state: LayoutMaterializationState,
        lower_bound: LayoutWatermark,
        upper_bound: LayoutWatermark,
        gap: Option<CoverageGapWitness>,
        source: LayoutMaterializationSourceIdentity,
    ) -> Result<Self, MaterializationDenial> {
        let family = state.family();
        let ordered_basis = matches!(
            state.class(),
            MaterializationStateClass::Exact
                | MaterializationStateClass::ExactThroughPhysicalBasis
                | MaterializationStateClass::EmptyInitialized
                | MaterializationStateClass::Lagged
                | MaterializationStateClass::Stale
                | MaterializationStateClass::PartiallyCovered
                | MaterializationStateClass::Quarantined
        );

        if ordered_basis && lower_bound.basis_kind() != upper_bound.basis_kind() {
            return Err(
                MaterializationDenial::CoverageBasisDoesNotMatchMaterializationState {
                    family,
                    state: state.class(),
                    basis_kind: upper_bound.basis_kind(),
                },
            );
        }

        if lower_bound.value() > upper_bound.value() {
            return Err(MaterializationDenial::CoverageIntervalIsReversed {
                family,
                basis_kind: upper_bound.basis_kind(),
                lower_bound: lower_bound.value(),
                upper_bound: upper_bound.value(),
            });
        }

        Ok(Self {
            family,
            state,
            lower_bound,
            upper_bound,
            gap,
            source,
        })
    }

    pub(super) fn from_admitted_bases(
        state: LayoutMaterializationState,
        lower: AdmittedCoverageBasis,
        upper: AdmittedCoverageBasis,
        gap: Option<CoverageGapWitness>,
    ) -> Result<Self, MaterializationDenial> {
        if lower.source() != upper.source() {
            return Err(MaterializationDenial::CoverageSourceMismatch);
        }
        Self::new(
            state,
            lower.watermark(),
            upper.watermark(),
            gap,
            lower.source().clone(),
        )
    }

    #[cfg(test)]
    pub(super) fn observed_exact_through(
        state: LayoutMaterializationState,
        watermark: LayoutWatermark,
        source: LayoutMaterializationSourceIdentity,
    ) -> Result<Self, MaterializationDenial> {
        Self::new(state, watermark, watermark, None, source)
    }

    #[cfg(test)]
    pub(super) fn observed_lagged(
        state: LayoutMaterializationState,
        lower_bound: LayoutWatermark,
        upper_bound: LayoutWatermark,
        source: LayoutMaterializationSourceIdentity,
    ) -> Result<Self, MaterializationDenial> {
        Self::new(state, lower_bound, upper_bound, None, source)
    }

    #[cfg(test)]
    pub(super) fn observed_partial(
        state: LayoutMaterializationState,
        lower_bound: LayoutWatermark,
        upper_bound: LayoutWatermark,
        gap: CoverageGapWitness,
        source: LayoutMaterializationSourceIdentity,
    ) -> Result<Self, MaterializationDenial> {
        Self::new(state, lower_bound, upper_bound, Some(gap), source)
    }

    pub const fn family(&self) -> PhysicalArtifactFamily {
        self.family
    }

    pub const fn state(&self) -> LayoutMaterializationState {
        self.state
    }

    pub const fn lower_bound(&self) -> LayoutWatermark {
        self.lower_bound
    }

    pub const fn upper_bound(&self) -> LayoutWatermark {
        self.upper_bound
    }

    pub const fn gap(&self) -> Option<CoverageGapWitness> {
        self.gap
    }

    pub const fn source(&self) -> &LayoutMaterializationSourceIdentity {
        &self.source
    }

    pub const fn is_exact(&self) -> bool {
        self.state.supports_exact_access() && self.gap.is_none()
    }

    pub fn require_exact(&self) -> Result<Self, MaterializationDenial> {
        if let Some(gap) = self.gap
            && self.state.class() != MaterializationStateClass::Quarantined
        {
            return Err(MaterializationDenial::LayoutCoverageIsPartial { gap });
        }

        match self.state.class() {
            MaterializationStateClass::Exact
            | MaterializationStateClass::ExactThroughPhysicalBasis
            | MaterializationStateClass::EmptyInitialized => Ok(self.clone()),
            MaterializationStateClass::Stale => Err(MaterializationDenial::LayoutCoverageIsStale {
                family: self.family,
                basis_kind: self.upper_bound.basis_kind(),
            }),
            MaterializationStateClass::Quarantined => {
                let gap = self.gap.unwrap_or_else(|| {
                    CoverageGapWitness::physical_range(
                        self.family,
                        self.upper_bound.basis_kind(),
                        self.lower_bound.value(),
                        self.upper_bound.value(),
                    )
                });
                Err(MaterializationDenial::LayoutRangeIsQuarantined { gap })
            }
            MaterializationStateClass::RebuildRequired => {
                Err(MaterializationDenial::LayoutRequiresRebuild {
                    family: self.family,
                })
            }
            MaterializationStateClass::Migrating => Err(MaterializationDenial::LayoutIsMigrating {
                family: self.family,
            }),
            MaterializationStateClass::Lagged => {
                Err(MaterializationDenial::LayoutCoverageIsLagged {
                    family: self.family,
                    basis_kind: self.upper_bound.basis_kind(),
                })
            }
            state => Err(
                MaterializationDenial::MaterializationStateDoesNotSupportExactAccess {
                    family: self.family,
                    state,
                },
            ),
        }
    }
}
