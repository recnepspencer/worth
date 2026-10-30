use worth_foundational::facade::{prepare_aspect_value_identity_basis, AspectValue};

use super::{
    WorthQueryDomainConditionFamily, WorthQueryPortableConditionParameter,
    WorthQueryQuantityValueFamily, WorthQuerySemanticTruthDependency, WorthQueryTemporalCondition,
    WorthQueryTypedFamilyIdentity,
};

mod portable_parts;
pub use portable_parts::{
    WorthQueryPortableConditionalConditionParts, WorthQueryPortableDeltaThresholdDenial,
    WorthQueryPortableDeltaThresholdParts,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryDeltaComparisonDomain {
    AbsoluteDifference,
    RelativeRatio,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryThresholdBoundary {
    Inclusive,
    Exclusive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeltaThreshold {
    value: AspectValue,
    unit: WorthQueryTypedFamilyIdentity,
    value_family: WorthQueryQuantityValueFamily,
    comparison_domain: WorthQueryDeltaComparisonDomain,
    boundary: WorthQueryThresholdBoundary,
}

impl WorthQueryDeltaThreshold {
    pub fn new<Unit: super::WorthQueryQuantityUnit>(
        value: AspectValue,
        comparison_domain: WorthQueryDeltaComparisonDomain,
        boundary: WorthQueryThresholdBoundary,
    ) -> Result<Self, &'static str> {
        validate_nonnegative_finite_numeric(&value)?;
        if numeric_family(&value) != Some(Unit::VALUE_FAMILY) {
            return Err("delta-threshold-value-family-mismatch");
        }
        let unit = WorthQueryTypedFamilyIdentity::declared(Unit::PORTABLE_IDENTITY);
        if !unit.is_portable() {
            return Err("invalid-portable-quantity-unit-identity");
        }
        Ok(Self {
            value,
            unit,
            value_family: Unit::VALUE_FAMILY,
            comparison_domain,
            boundary,
        })
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }

    pub fn unit(&self) -> &WorthQueryTypedFamilyIdentity {
        &self.unit
    }

    pub const fn comparison_domain(&self) -> WorthQueryDeltaComparisonDomain {
        self.comparison_domain
    }

    pub const fn boundary(&self) -> WorthQueryThresholdBoundary {
        self.boundary
    }

    pub fn value_family(&self) -> WorthQueryQuantityValueFamily {
        self.value_family
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ConditionalEvaluationConditionKind {
    AlwaysEligible,
    AspectFiltered(Vec<WorthQuerySemanticTruthDependency>),
    DeltaThreshold {
        dependency: Box<WorthQuerySemanticTruthDependency>,
        threshold: WorthQueryDeltaThreshold,
    },
    OnDemand,
    Temporal(WorthQueryTemporalCondition),
    DomainSpecific {
        family: WorthQueryTypedFamilyIdentity,
        parameters: Vec<WorthQueryPortableConditionParameter>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConditionalEvaluationCondition(ConditionalEvaluationConditionKind);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalConditionClass {
    AlwaysEligible,
    AspectFiltered,
    DeltaThreshold,
    OnDemand,
    Temporal,
    DomainSpecific,
}

impl WorthQueryConditionalEvaluationCondition {
    pub const fn class(&self) -> WorthQueryConditionalConditionClass {
        match self.0 {
            ConditionalEvaluationConditionKind::AlwaysEligible => {
                WorthQueryConditionalConditionClass::AlwaysEligible
            }
            ConditionalEvaluationConditionKind::AspectFiltered(_) => {
                WorthQueryConditionalConditionClass::AspectFiltered
            }
            ConditionalEvaluationConditionKind::DeltaThreshold { .. } => {
                WorthQueryConditionalConditionClass::DeltaThreshold
            }
            ConditionalEvaluationConditionKind::OnDemand => {
                WorthQueryConditionalConditionClass::OnDemand
            }
            ConditionalEvaluationConditionKind::Temporal(_) => {
                WorthQueryConditionalConditionClass::Temporal
            }
            ConditionalEvaluationConditionKind::DomainSpecific { .. } => {
                WorthQueryConditionalConditionClass::DomainSpecific
            }
        }
    }
    pub fn always_eligible() -> Self {
        Self(ConditionalEvaluationConditionKind::AlwaysEligible)
    }

    pub fn aspect_filtered(
        dependencies: impl IntoIterator<Item = WorthQuerySemanticTruthDependency>,
    ) -> Result<Self, &'static str> {
        let mut dependencies = dependencies.into_iter().collect::<Vec<_>>();
        canonicalize_dependencies(&mut dependencies);
        if dependencies.is_empty() {
            return Err("empty-aspect-filter");
        }
        Ok(Self(ConditionalEvaluationConditionKind::AspectFiltered(
            dependencies,
        )))
    }

    pub fn delta_threshold(
        dependency: WorthQuerySemanticTruthDependency,
        threshold: WorthQueryDeltaThreshold,
    ) -> Self {
        Self(ConditionalEvaluationConditionKind::DeltaThreshold {
            dependency: Box::new(dependency),
            threshold,
        })
    }

    pub fn on_demand() -> Self {
        Self(ConditionalEvaluationConditionKind::OnDemand)
    }

    pub fn temporal(condition: WorthQueryTemporalCondition) -> Self {
        Self(ConditionalEvaluationConditionKind::Temporal(condition))
    }

    pub fn domain_specific<Family: WorthQueryDomainConditionFamily>(
        parameters: impl IntoIterator<Item = WorthQueryPortableConditionParameter>,
    ) -> Result<Self, &'static str> {
        let family = WorthQueryTypedFamilyIdentity::declared(Family::PORTABLE_IDENTITY);
        if !family.is_portable() {
            return Err("invalid-portable-conditional-family-identity");
        }
        let mut parameters = parameters.into_iter().collect::<Vec<_>>();
        parameters.sort_by(|left, right| left.name().cmp(right.name()));
        if parameters
            .windows(2)
            .any(|pair| pair[0].name() == pair[1].name())
        {
            return Err("duplicate-portable-condition-parameter-name");
        }
        Ok(Self(ConditionalEvaluationConditionKind::DomainSpecific {
            family,
            parameters,
        }))
    }

    pub(crate) fn canonicalize(&mut self) {
        match &mut self.0 {
            ConditionalEvaluationConditionKind::AspectFiltered(dependencies) => {
                canonicalize_dependencies(dependencies)
            }
            ConditionalEvaluationConditionKind::DomainSpecific { parameters, .. } => {
                parameters.sort_by(|left, right| left.name().cmp(right.name()));
            }
            ConditionalEvaluationConditionKind::AlwaysEligible
            | ConditionalEvaluationConditionKind::DeltaThreshold { .. }
            | ConditionalEvaluationConditionKind::OnDemand
            | ConditionalEvaluationConditionKind::Temporal(_) => {}
        }
    }

    pub fn dependencies(&self) -> &[WorthQuerySemanticTruthDependency] {
        match &self.0 {
            ConditionalEvaluationConditionKind::AspectFiltered(dependencies) => dependencies,
            ConditionalEvaluationConditionKind::DeltaThreshold { dependency, .. } => {
                std::slice::from_ref(dependency.as_ref())
            }
            _ => &[],
        }
    }

    pub(crate) fn trigger_class(&self) -> ConditionalTriggerClass {
        match self.0 {
            ConditionalEvaluationConditionKind::OnDemand => ConditionalTriggerClass::OnDemand,
            ConditionalEvaluationConditionKind::Temporal(_) => ConditionalTriggerClass::Temporal,
            ConditionalEvaluationConditionKind::AlwaysEligible
            | ConditionalEvaluationConditionKind::AspectFiltered(_)
            | ConditionalEvaluationConditionKind::DeltaThreshold { .. }
            | ConditionalEvaluationConditionKind::DomainSpecific { .. } => {
                ConditionalTriggerClass::DependencyOrExternal
            }
        }
    }

    pub(crate) fn canonical_token(&self) -> String {
        match &self.0 {
            ConditionalEvaluationConditionKind::AlwaysEligible => "always".to_string(),
            ConditionalEvaluationConditionKind::AspectFiltered(dependencies) => {
                let mut material = "aspect-filtered;".to_string();
                for dependency in dependencies {
                    super::push_token(
                        &mut material,
                        "dependency",
                        &super::dependency_token(dependency),
                    );
                }
                material
            }
            ConditionalEvaluationConditionKind::DeltaThreshold {
                dependency,
                threshold,
            } => {
                let mut material = "delta-threshold;".to_string();
                super::push_token(
                    &mut material,
                    "dependency",
                    &super::dependency_token(dependency),
                );
                super::push_token(
                    &mut material,
                    "value",
                    prepare_aspect_value_identity_basis(&threshold.value).as_str(),
                );
                super::push_token(
                    &mut material,
                    "comparison-domain",
                    match threshold.comparison_domain {
                        WorthQueryDeltaComparisonDomain::AbsoluteDifference => {
                            "absolute-difference"
                        }
                        WorthQueryDeltaComparisonDomain::RelativeRatio => "relative-ratio",
                    },
                );
                super::push_token(
                    &mut material,
                    "boundary",
                    match threshold.boundary {
                        WorthQueryThresholdBoundary::Inclusive => "inclusive",
                        WorthQueryThresholdBoundary::Exclusive => "exclusive",
                    },
                );
                super::push_token(&mut material, "unit", threshold.unit.as_str());
                material
            }
            ConditionalEvaluationConditionKind::OnDemand => "on-demand".to_string(),
            ConditionalEvaluationConditionKind::Temporal(condition) => {
                format!(
                    "temporal:{}",
                    super::temporal::temporal_condition_token(*condition)
                )
            }
            ConditionalEvaluationConditionKind::DomainSpecific { family, parameters } => {
                let mut material = "domain-specific;".to_string();
                super::push_token(&mut material, "family", family.as_str());
                for parameter in parameters {
                    super::push_token(
                        &mut material,
                        "parameter",
                        &super::condition_parameter::parameter_token(parameter),
                    );
                }
                material
            }
        }
    }

    pub fn portable_family_identity(&self) -> Option<&WorthQueryTypedFamilyIdentity> {
        match &self.0 {
            ConditionalEvaluationConditionKind::DomainSpecific { family, .. } => Some(family),
            _ => None,
        }
    }

    pub fn delta_threshold_contract(
        &self,
    ) -> Option<(
        &WorthQuerySemanticTruthDependency,
        &WorthQueryDeltaThreshold,
    )> {
        match &self.0 {
            ConditionalEvaluationConditionKind::DeltaThreshold {
                dependency,
                threshold,
            } => Some((dependency.as_ref(), threshold)),
            _ => None,
        }
    }

    pub fn domain_specific_parameters(&self) -> &[WorthQueryPortableConditionParameter] {
        match &self.0 {
            ConditionalEvaluationConditionKind::DomainSpecific { parameters, .. } => parameters,
            _ => &[],
        }
    }

    pub fn temporal_condition(&self) -> Option<WorthQueryTemporalCondition> {
        match self.0 {
            ConditionalEvaluationConditionKind::Temporal(condition) => Some(condition),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConditionalTriggerClass {
    DependencyOrExternal,
    OnDemand,
    Temporal,
}

fn canonicalize_dependencies(dependencies: &mut Vec<WorthQuerySemanticTruthDependency>) {
    dependencies.sort_by_key(super::dependency_token);
    dependencies.dedup();
}

fn validate_nonnegative_finite_numeric(value: &AspectValue) -> Result<(), &'static str> {
    let valid = match value {
        AspectValue::UInt8(_)
        | AspectValue::UInt16(_)
        | AspectValue::UInt32(_)
        | AspectValue::UInt64(_) => true,
        AspectValue::Int8(value) => *value >= 0,
        AspectValue::Int16(value) => *value >= 0,
        AspectValue::Int32(value) => *value >= 0,
        AspectValue::Int64(value) => *value >= 0,
        AspectValue::Float32(value) => value.as_f32().is_finite() && value.as_f32() >= 0.0,
        AspectValue::Float64(value) => value.as_f64().is_finite() && value.as_f64() >= 0.0,
        _ => false,
    };
    valid.then_some(()).ok_or("invalid-delta-threshold")
}

fn numeric_family(value: &AspectValue) -> Option<super::WorthQueryQuantityValueFamily> {
    use super::WorthQueryQuantityValueFamily as Family;
    match value {
        AspectValue::UInt8(_)
        | AspectValue::UInt16(_)
        | AspectValue::UInt32(_)
        | AspectValue::UInt64(_)
        | AspectValue::Int8(_)
        | AspectValue::Int16(_)
        | AspectValue::Int32(_)
        | AspectValue::Int64(_) => Some(Family::Integer),
        AspectValue::Float32(_) => Some(Family::Float32),
        AspectValue::Float64(_) => Some(Family::Float64),
        _ => None,
    }
}
