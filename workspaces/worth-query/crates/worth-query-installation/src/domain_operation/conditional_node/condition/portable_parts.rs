//! Owned reconstruction parts for authority-free conditional conditions.

use worth_foundational::facade::AspectValue;
use worth_query_declaration::archive_decode_authority::ArchiveDecodeAuthority;

use super::{
    numeric_family, validate_nonnegative_finite_numeric, ConditionalEvaluationConditionKind,
    WorthQueryConditionalEvaluationCondition, WorthQueryDeltaComparisonDomain,
    WorthQueryDeltaThreshold, WorthQueryThresholdBoundary,
};
use crate::domain_operation::conditional_node::{
    WorthQueryPortableConditionParameter, WorthQueryQuantityValueFamily,
    WorthQuerySemanticTruthDependency, WorthQueryTemporalCondition, WorthQueryTypedFamilyIdentity,
};

/// Exact owned fields for one decoded delta threshold.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableDeltaThresholdParts {
    pub value: AspectValue,
    pub unit: WorthQueryTypedFamilyIdentity,
    pub value_family: WorthQueryQuantityValueFamily,
    pub comparison_domain: WorthQueryDeltaComparisonDomain,
    pub boundary: WorthQueryThresholdBoundary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableDeltaThresholdDenial {
    InvalidNumericValue,
    ValueFamilyMismatch,
}

impl WorthQueryDeltaThreshold {
    /// Reconstructs an authority-free threshold without a compile-time unit type.
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableDeltaThresholdParts,
    ) -> Result<Self, WorthQueryPortableDeltaThresholdDenial> {
        validate_nonnegative_finite_numeric(&parts.value)
            .map_err(|_| WorthQueryPortableDeltaThresholdDenial::InvalidNumericValue)?;
        if numeric_family(&parts.value) != Some(parts.value_family) {
            return Err(WorthQueryPortableDeltaThresholdDenial::ValueFamilyMismatch);
        }
        Ok(Self {
            value: parts.value,
            unit: parts.unit,
            value_family: parts.value_family,
            comparison_domain: parts.comparison_domain,
            boundary: parts.boundary,
        })
    }

    pub fn into_parts(self) -> WorthQueryPortableDeltaThresholdParts {
        WorthQueryPortableDeltaThresholdParts {
            value: self.value,
            unit: self.unit,
            value_family: self.value_family,
            comparison_domain: self.comparison_domain,
            boundary: self.boundary,
        }
    }
}

/// Exact owned variant data for one decoded conditional condition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableConditionalConditionParts {
    AlwaysEligible,
    AspectFiltered(Vec<WorthQuerySemanticTruthDependency>),
    DeltaThreshold {
        dependency: WorthQuerySemanticTruthDependency,
        threshold: WorthQueryDeltaThreshold,
    },
    OnDemand,
    Temporal(WorthQueryTemporalCondition),
    DomainSpecific {
        family: WorthQueryTypedFamilyIdentity,
        parameters: Vec<WorthQueryPortableConditionParameter>,
    },
}

impl WorthQueryConditionalEvaluationCondition {
    /// Reconstructs an untrusted condition exactly as represented.
    ///
    /// Sequence order and duplicates are deliberately preserved for the fresh
    /// package-readmission boundary to judge.
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableConditionalConditionParts,
    ) -> Self {
        let kind = match parts {
            WorthQueryPortableConditionalConditionParts::AlwaysEligible => {
                ConditionalEvaluationConditionKind::AlwaysEligible
            }
            WorthQueryPortableConditionalConditionParts::AspectFiltered(dependencies) => {
                ConditionalEvaluationConditionKind::AspectFiltered(dependencies)
            }
            WorthQueryPortableConditionalConditionParts::DeltaThreshold {
                dependency,
                threshold,
            } => ConditionalEvaluationConditionKind::DeltaThreshold {
                dependency: Box::new(dependency),
                threshold,
            },
            WorthQueryPortableConditionalConditionParts::OnDemand => {
                ConditionalEvaluationConditionKind::OnDemand
            }
            WorthQueryPortableConditionalConditionParts::Temporal(condition) => {
                ConditionalEvaluationConditionKind::Temporal(condition)
            }
            WorthQueryPortableConditionalConditionParts::DomainSpecific { family, parameters } => {
                ConditionalEvaluationConditionKind::DomainSpecific { family, parameters }
            }
        };
        Self(kind)
    }

    pub fn into_parts(self) -> WorthQueryPortableConditionalConditionParts {
        match self.0 {
            ConditionalEvaluationConditionKind::AlwaysEligible => {
                WorthQueryPortableConditionalConditionParts::AlwaysEligible
            }
            ConditionalEvaluationConditionKind::AspectFiltered(dependencies) => {
                WorthQueryPortableConditionalConditionParts::AspectFiltered(dependencies)
            }
            ConditionalEvaluationConditionKind::DeltaThreshold {
                dependency,
                threshold,
            } => WorthQueryPortableConditionalConditionParts::DeltaThreshold {
                dependency: *dependency,
                threshold,
            },
            ConditionalEvaluationConditionKind::OnDemand => {
                WorthQueryPortableConditionalConditionParts::OnDemand
            }
            ConditionalEvaluationConditionKind::Temporal(condition) => {
                WorthQueryPortableConditionalConditionParts::Temporal(condition)
            }
            ConditionalEvaluationConditionKind::DomainSpecific { family, parameters } => {
                WorthQueryPortableConditionalConditionParts::DomainSpecific { family, parameters }
            }
        }
    }
}
