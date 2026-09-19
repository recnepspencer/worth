use std::sync::Arc;

use super::provider_semantics::{
    BridgeConditionalProviderSemanticContracts, BridgeErasedProviderSemanticContract,
};
use super::BridgeConditionalProviderSemantics;

#[derive(Debug, Clone)]
pub struct BridgeConditionalResolverContext {
    pub dirty_aspects: worth_signal::facade::AspectMask,
    /// Signal-version distance only. This is never a unitful semantic delta;
    /// typed threshold providers must use their installed domain observation.
    pub max_signal_version_delta: u64,
    truth_branch_identity: Option<Arc<str>>,
    truth_snapshot_identity: Arc<str>,
    observations: super::observation_retention::BridgeRetainedObservations,
    _reservation: Arc<super::retention::BridgeRetentionReservation>,
}

impl BridgeConditionalResolverContext {
    pub(super) fn new(
        dirty_aspects: worth_signal::facade::AspectMask,
        max_signal_version_delta: u64,
        truth_branch_identity: Option<&str>,
        truth_snapshot_identity: &str,
        observations: super::observation_retention::BridgeRetainedObservations,
        reservation: Arc<super::retention::BridgeRetentionReservation>,
    ) -> Self {
        Self {
            dirty_aspects,
            max_signal_version_delta,
            truth_branch_identity: truth_branch_identity.map(Arc::from),
            truth_snapshot_identity: Arc::from(truth_snapshot_identity),
            observations,
            _reservation: reservation,
        }
    }

    pub fn truth_branch_identity(&self) -> Option<&str> {
        self.truth_branch_identity.as_deref()
    }

    pub fn truth_snapshot_identity(&self) -> &str {
        &self.truth_snapshot_identity
    }

    pub fn observations(&self) -> &[BridgeConditionalSemanticObservation] {
        &self.observations
    }

    pub fn observation(
        &self,
        dependency_ordinal: usize,
    ) -> Option<&BridgeConditionalSemanticObservation> {
        self.observations
            .iter()
            .find(|observation| observation.dependency_ordinal == dependency_ordinal)
    }
}

#[derive(Debug, Clone)]
pub struct BridgeConditionalSemanticObservation {
    dependency_ordinal: usize,
    previous: Option<worth_foundational::facade::ContractValidatedAspectArtifact>,
    current: Option<worth_foundational::facade::ContractValidatedAspectArtifact>,
    projection_mask:
        worth_foundational::facade::AspectMask<worth_foundational::facade::ProjectionMask>,
}

impl BridgeConditionalSemanticObservation {
    pub(super) fn new(
        dependency_ordinal: usize,
        previous: Option<worth_foundational::facade::ContractValidatedAspectArtifact>,
        current: Option<worth_foundational::facade::ContractValidatedAspectArtifact>,
        projection_mask: worth_foundational::facade::AspectMask<
            worth_foundational::facade::ProjectionMask,
        >,
    ) -> Self {
        Self {
            dependency_ordinal,
            previous,
            current,
            projection_mask,
        }
    }

    pub const fn dependency_ordinal(&self) -> usize {
        self.dependency_ordinal
    }

    pub fn previous(&self) -> Option<&worth_foundational::facade::ContractValidatedAspectArtifact> {
        self.previous.as_ref()
    }

    pub fn current(&self) -> Option<&worth_foundational::facade::ContractValidatedAspectArtifact> {
        self.current.as_ref()
    }

    pub fn projection_mask(
        &self,
    ) -> &worth_foundational::facade::AspectMask<worth_foundational::facade::ProjectionMask> {
        &self.projection_mask
    }
}

pub trait BridgeConditionalConditionProvider: Send + Sync + 'static {
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String>;
}

pub trait BridgeConditionalComparatorProvider: Send + Sync + 'static {
    fn has_meaningful_change(
        &self,
        aspect: worth_signal::facade::Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, String>;
}

pub trait BridgeConditionalTriggerProvider: Send + Sync + 'static {
    fn requested(&self) -> bool;
}

pub trait BridgeConditionalWakeProvider: Send + Sync + 'static {
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String>;
}

pub trait BridgeConditionalComputeProvider: Send + Sync + 'static {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String>;
}

#[derive(Clone, Default)]
pub struct BridgeConditionalProviderSet {
    pub(crate) condition: Option<Arc<dyn BridgeConditionalConditionProvider>>,
    pub(crate) dependency_comparator: Option<Arc<dyn BridgeConditionalComparatorProvider>>,
    pub(crate) output_comparator: Option<Arc<dyn BridgeConditionalComparatorProvider>>,
    pub(crate) reuse_comparator: Option<Arc<dyn BridgeConditionalComparatorProvider>>,
    pub(crate) trigger: Option<Arc<dyn BridgeConditionalTriggerProvider>>,
    pub(crate) wake: Option<Arc<dyn BridgeConditionalWakeProvider>>,
    pub(crate) compute: Option<Arc<dyn BridgeConditionalComputeProvider>>,
    pub(super) semantic_contracts: BridgeConditionalProviderSemanticContracts,
}

impl BridgeConditionalProviderSet {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn condition<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalConditionProvider + BridgeConditionalProviderSemantics,
    {
        self.semantic_contracts.condition =
            Some(BridgeErasedProviderSemanticContract::capture(&provider));
        self.condition = Some(Arc::new(provider));
        self
    }
    pub fn dependency_comparator<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComparatorProvider + BridgeConditionalProviderSemantics,
    {
        self.semantic_contracts.dependency_comparator =
            Some(BridgeErasedProviderSemanticContract::capture(&provider));
        self.dependency_comparator = Some(Arc::new(provider));
        self
    }
    pub fn output_comparator<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComparatorProvider + BridgeConditionalProviderSemantics,
    {
        self.semantic_contracts.output_comparator =
            Some(BridgeErasedProviderSemanticContract::capture(&provider));
        self.output_comparator = Some(Arc::new(provider));
        self
    }
    pub fn reuse_comparator<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComparatorProvider + BridgeConditionalProviderSemantics,
    {
        self.semantic_contracts.reuse_comparator =
            Some(BridgeErasedProviderSemanticContract::capture(&provider));
        self.reuse_comparator = Some(Arc::new(provider));
        self
    }
    pub fn trigger<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalTriggerProvider + BridgeConditionalProviderSemantics,
    {
        self.semantic_contracts.trigger =
            Some(BridgeErasedProviderSemanticContract::capture(&provider));
        self.trigger = Some(Arc::new(provider));
        self
    }
    pub fn wake<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalWakeProvider + BridgeConditionalProviderSemantics,
    {
        self.semantic_contracts.wake =
            Some(BridgeErasedProviderSemanticContract::capture(&provider));
        self.wake = Some(Arc::new(provider));
        self
    }
    pub fn compute<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComputeProvider + BridgeConditionalProviderSemantics,
    {
        self.semantic_contracts.compute =
            Some(BridgeErasedProviderSemanticContract::capture(&provider));
        self.compute = Some(Arc::new(provider));
        self
    }

    pub fn has_compute_provider(&self) -> bool {
        self.compute.is_some()
    }

    pub(super) fn retained_arc_bytes(
        &self,
    ) -> Result<u64, super::retention::BridgeRetentionDenial> {
        let provider_bytes = [
            self.condition
                .as_deref()
                .map(super::retention::arc_value_charge),
            self.dependency_comparator
                .as_deref()
                .map(super::retention::arc_value_charge),
            self.output_comparator
                .as_deref()
                .map(super::retention::arc_value_charge),
            self.reuse_comparator
                .as_deref()
                .map(super::retention::arc_value_charge),
            self.trigger
                .as_deref()
                .map(super::retention::arc_value_charge),
            self.wake.as_deref().map(super::retention::arc_value_charge),
            self.compute
                .as_deref()
                .map(super::retention::arc_value_charge),
        ]
        .into_iter()
        .flatten()
        .try_fold(0u64, |total, charge| {
            total
                .checked_add(charge?)
                .ok_or(super::retention::BridgeRetentionDenial::BytesExhausted)
        })?;
        super::retention::sum(&[
            provider_bytes,
            self.semantic_contracts.retained_arc_bytes()?,
        ])
    }
}
