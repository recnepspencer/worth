use std::sync::Arc;

use worth_signal::facade::{
    SignalConditionalArtifactReuse, SignalConditionalVersionComparator,
    SignalDeltaThresholdContract,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BridgeConditionalLocation {
    stage_identity: Option<Arc<str>>,
    node_identity: Arc<str>,
}

impl BridgeConditionalLocation {
    pub fn operation(node_identity: impl Into<Arc<str>>) -> Self {
        Self {
            stage_identity: None,
            node_identity: node_identity.into(),
        }
    }

    pub fn workflow_stage(
        stage_identity: impl Into<Arc<str>>,
        node_identity: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            stage_identity: Some(stage_identity.into()),
            node_identity: node_identity.into(),
        }
    }

    pub fn stage_identity(&self) -> Option<&str> {
        self.stage_identity.as_deref()
    }

    pub fn node_identity(&self) -> &str {
        &self.node_identity
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.node_identity.trim().is_empty()
            && self
                .stage_identity
                .as_deref()
                .is_none_or(|identity| !identity.trim().is_empty())
    }

    pub(in crate::conditional_execution) fn retained_heap_bytes(
        &self,
    ) -> Result<u64, super::retention::BridgeRetentionDenial> {
        super::retention::sum(&[
            super::retention::arc_slice_charge::<u8>(self.node_identity.len())?,
            self.stage_identity
                .as_deref()
                .map(|identity| super::retention::arc_slice_charge::<u8>(identity.len()))
                .transpose()?
                .unwrap_or(0),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeConditionalCondition {
    Always,
    AspectFiltered,
    DeltaThreshold(SignalDeltaThresholdContract),
    OnDemand,
    RuntimePredicate,
    TemporalWake,
}

#[derive(Clone, Debug)]
pub struct BridgeConditionalContractParts {
    pub identity: Arc<str>,
    pub dependency_count: usize,
    pub condition_dependency_ordinals: Vec<usize>,
    pub condition: BridgeConditionalCondition,
    pub dependency_comparator: SignalConditionalVersionComparator,
    pub output_comparator: SignalConditionalVersionComparator,
    pub artifact_reuse: SignalConditionalArtifactReuse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeConditionalContract {
    identity: Arc<str>,
    dependency_count: usize,
    condition_dependency_ordinals: Vec<usize>,
    condition: BridgeConditionalCondition,
    dependency_comparator: SignalConditionalVersionComparator,
    output_comparator: SignalConditionalVersionComparator,
    artifact_reuse: SignalConditionalArtifactReuse,
}

impl BridgeConditionalContract {
    pub fn new(mut parts: BridgeConditionalContractParts) -> Self {
        parts.condition_dependency_ordinals.sort_unstable();
        parts.condition_dependency_ordinals.dedup();
        Self {
            identity: parts.identity,
            dependency_count: parts.dependency_count,
            condition_dependency_ordinals: parts.condition_dependency_ordinals,
            condition: parts.condition,
            dependency_comparator: parts.dependency_comparator,
            output_comparator: parts.output_comparator,
            artifact_reuse: parts.artifact_reuse,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn dependency_count(&self) -> usize {
        self.dependency_count
    }

    pub fn condition_dependency_ordinals(&self) -> &[usize] {
        &self.condition_dependency_ordinals
    }

    pub fn condition(&self) -> &BridgeConditionalCondition {
        &self.condition
    }

    pub const fn dependency_comparator(&self) -> SignalConditionalVersionComparator {
        self.dependency_comparator
    }

    pub const fn output_comparator(&self) -> SignalConditionalVersionComparator {
        self.output_comparator
    }

    pub const fn artifact_reuse(&self) -> SignalConditionalArtifactReuse {
        self.artifact_reuse
    }

    pub(in crate::conditional_execution) fn retained_heap_bytes(
        &self,
    ) -> Result<u64, super::retention::BridgeRetentionDenial> {
        super::retention::sum(&[
            super::retention::arc_slice_charge::<u8>(self.identity.len())?,
            super::retention::array_charge::<usize>(self.condition_dependency_ordinals.capacity())?,
        ])
    }

    pub(crate) fn requires_condition_provider(&self) -> bool {
        matches!(self.condition, BridgeConditionalCondition::RuntimePredicate)
    }

    pub(crate) fn requires_dependency_comparator(&self) -> bool {
        matches!(
            self.dependency_comparator,
            SignalConditionalVersionComparator::RuntimeResolved
        )
    }

    pub(crate) fn requires_output_comparator(&self) -> bool {
        matches!(
            self.output_comparator,
            SignalConditionalVersionComparator::RuntimeResolved
        )
    }

    pub(crate) fn requires_reuse_comparator(&self) -> bool {
        matches!(
            self.artifact_reuse,
            SignalConditionalArtifactReuse::RuntimeResolved
        )
    }

    pub(crate) fn requires_trigger_provider(&self) -> bool {
        matches!(self.condition, BridgeConditionalCondition::OnDemand)
    }

    pub(crate) fn requires_wake_provider(&self) -> bool {
        matches!(self.condition, BridgeConditionalCondition::TemporalWake)
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.identity.trim().is_empty()
            && (self.dependency_count > 0
                || matches!(
                    self.condition,
                    BridgeConditionalCondition::Always
                        | BridgeConditionalCondition::OnDemand
                        | BridgeConditionalCondition::RuntimePredicate
                        | BridgeConditionalCondition::TemporalWake
                ))
            && self
                .condition_dependency_ordinals
                .iter()
                .all(|ordinal| *ordinal < self.dependency_count)
    }
}
