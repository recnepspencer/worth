//! Pure authored conditional meaning, independent of installed runtime bindings.
use crate::data::aspect::AspectMask;
mod retained_charge;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SignalConditionalCondition {
    Always,
    AspectFilter(AspectMask),
    DeltaThreshold(SignalDeltaThresholdContract),
    OnDemand,
    RuntimePredicate,
    TemporalWake,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalThresholdValueFamily {
    Integer,
    Float32,
    Float64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalThresholdComparisonDomain {
    AbsoluteDifference,
    RelativeRatio,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalThresholdBoundary {
    Inclusive,
    Exclusive,
}

/// Signal-owned, lossless threshold meaning installed by the lowering owner.
/// The runtime predicate identity remains opaque, while this contract retains
/// the typed semantic parameters that identity is authorized to evaluate.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignalDeltaThresholdContract {
    threshold: worth_foundational::facade::AspectValue,
    unit_identity: String,
    value_family: SignalThresholdValueFamily,
    comparison_domain: SignalThresholdComparisonDomain,
    boundary: SignalThresholdBoundary,
}

impl SignalDeltaThresholdContract {
    /// Backing storage retained by this authored threshold, excluding its inline value.
    pub fn owned_allocation_capacity_bytes(&self) -> Option<usize> {
        self.threshold
            .owned_allocation_capacity_bytes()
            .checked_add(self.unit_identity.capacity())
    }
    pub fn new(
        threshold: worth_foundational::facade::AspectValue,
        unit_identity: impl Into<String>,
        value_family: SignalThresholdValueFamily,
        comparison_domain: SignalThresholdComparisonDomain,
        boundary: SignalThresholdBoundary,
    ) -> Self {
        Self {
            threshold,
            unit_identity: unit_identity.into(),
            value_family,
            comparison_domain,
            boundary,
        }
    }

    pub fn threshold(&self) -> &worth_foundational::facade::AspectValue {
        &self.threshold
    }

    pub fn unit_identity(&self) -> &str {
        &self.unit_identity
    }

    pub const fn value_family(&self) -> SignalThresholdValueFamily {
        self.value_family
    }

    pub const fn comparison_domain(&self) -> SignalThresholdComparisonDomain {
        self.comparison_domain
    }

    pub const fn boundary(&self) -> SignalThresholdBoundary {
        self.boundary
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalConditionalVersionComparator {
    Exact,
    Tolerance(u64),
    OutputIdentity,
    RuntimeResolved,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalConditionalArtifactReuse {
    NotReusable,
    DependencyAndOutputEquivalent,
    OutputEquivalent,
    RuntimeResolved,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignalConditionalContractDefinition {
    pub condition: SignalConditionalCondition,
    pub dependency_aspects: AspectMask,
    pub trigger_aspects: AspectMask,
    pub dependency_comparator: SignalConditionalVersionComparator,
    pub output_comparator: SignalConditionalVersionComparator,
    pub artifact_reuse: SignalConditionalArtifactReuse,
}
