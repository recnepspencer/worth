use std::collections::BTreeSet;

use super::{
    canonical_identity::canonical_resource_request_identity, validation::validate_resource_request,
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionBoundary,
    WorthQueryExecutionDegradation, WorthQueryExecutionMode, WorthQueryPartialEffectPosture,
    WorthQueryResourceLimitRequest, WorthQueryRetainedProgressPosture,
    WorthQuerySemanticScaleRequest, WorthQueryYieldedStatePosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExecutionResourceRequest {
    boundary: WorthQueryExecutionBoundary,
    scale: WorthQuerySemanticScaleRequest,
    limits: WorthQueryResourceLimitRequest,
    modes: BTreeSet<WorthQueryExecutionMode>,
    degradations: BTreeSet<WorthQueryExecutionDegradation>,
    partial_effect_postures: BTreeSet<WorthQueryPartialEffectPosture>,
    yielded_state_postures: BTreeSet<WorthQueryYieldedStatePosture>,
    retained_progress_postures: BTreeSet<WorthQueryRetainedProgressPosture>,
    cancellation_safe_point: WorthQueryCancellationSafePointFamily,
}

impl WorthQueryExecutionResourceRequest {
    pub fn new(
        scale: WorthQuerySemanticScaleRequest,
        limits: WorthQueryResourceLimitRequest,
        cancellation_safe_point: WorthQueryCancellationSafePointFamily,
    ) -> Result<Self, &'static str> {
        let request = Self {
            boundary: WorthQueryExecutionBoundary::BoundedStep,
            scale,
            limits,
            modes: [WorthQueryExecutionMode::Synchronous].into_iter().collect(),
            degradations: BTreeSet::new(),
            partial_effect_postures: [WorthQueryPartialEffectPosture::EffectFree]
                .into_iter()
                .collect(),
            yielded_state_postures: [WorthQueryYieldedStatePosture::NotYieldable]
                .into_iter()
                .collect(),
            retained_progress_postures: [WorthQueryRetainedProgressPosture::ReleaseAfterAttempt]
                .into_iter()
                .collect(),
            cancellation_safe_point,
        };
        validate_resource_request(&request)?;
        Ok(request)
    }

    /// One native attempt, with only its actual stated demands.
    pub fn atomic(
        scale: WorthQuerySemanticScaleRequest,
        limits: WorthQueryResourceLimitRequest,
        cancellation_safe_point: WorthQueryCancellationSafePointFamily,
    ) -> Result<Self, &'static str> {
        let request = Self {
            boundary: WorthQueryExecutionBoundary::Atomic,
            scale,
            limits,
            modes: [WorthQueryExecutionMode::Synchronous].into_iter().collect(),
            degradations: BTreeSet::new(),
            partial_effect_postures: [WorthQueryPartialEffectPosture::EffectFree]
                .into_iter()
                .collect(),
            yielded_state_postures: [WorthQueryYieldedStatePosture::NotYieldable]
                .into_iter()
                .collect(),
            retained_progress_postures: [WorthQueryRetainedProgressPosture::ReleaseAfterAttempt]
                .into_iter()
                .collect(),
            cancellation_safe_point,
        };
        request.validate()?;
        Ok(request)
    }

    pub const fn boundary(&self) -> WorthQueryExecutionBoundary {
        self.boundary
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_resource_request(self)
    }

    pub fn bounded(
        scale_ceiling: u64,
        resource_ceiling: u64,
        cancellation_safe_point: WorthQueryCancellationSafePointFamily,
    ) -> Self {
        Self::new(
            WorthQuerySemanticScaleRequest::bounded(scale_ceiling),
            WorthQueryResourceLimitRequest::bounded(resource_ceiling),
            cancellation_safe_point,
        )
        .expect("complete bounded resource request is valid")
    }

    pub fn allow_mode(mut self, mode: WorthQueryExecutionMode) -> Self {
        self.modes.insert(mode);
        self
    }

    pub fn allow_degradation(mut self, degradation: WorthQueryExecutionDegradation) -> Self {
        self.degradations.insert(degradation);
        self
    }

    pub fn allow_partial_effect_posture(mut self, posture: WorthQueryPartialEffectPosture) -> Self {
        self.partial_effect_postures.insert(posture);
        self
    }

    pub fn allow_yielded_state_posture(mut self, posture: WorthQueryYieldedStatePosture) -> Self {
        self.yielded_state_postures.insert(posture);
        self
    }

    pub fn allow_retained_progress_posture(
        mut self,
        posture: WorthQueryRetainedProgressPosture,
    ) -> Self {
        self.retained_progress_postures.insert(posture);
        self
    }

    pub fn scale(&self) -> &WorthQuerySemanticScaleRequest {
        &self.scale
    }

    pub fn limits(&self) -> &WorthQueryResourceLimitRequest {
        &self.limits
    }

    pub fn modes(&self) -> &BTreeSet<WorthQueryExecutionMode> {
        &self.modes
    }

    pub fn degradations(&self) -> &BTreeSet<WorthQueryExecutionDegradation> {
        &self.degradations
    }

    pub fn partial_effect_postures(&self) -> &BTreeSet<WorthQueryPartialEffectPosture> {
        &self.partial_effect_postures
    }

    pub fn yielded_state_postures(&self) -> &BTreeSet<WorthQueryYieldedStatePosture> {
        &self.yielded_state_postures
    }

    pub fn retained_progress_postures(&self) -> &BTreeSet<WorthQueryRetainedProgressPosture> {
        &self.retained_progress_postures
    }

    pub fn cancellation_safe_point(&self) -> &WorthQueryCancellationSafePointFamily {
        &self.cancellation_safe_point
    }

    pub fn canonical_identity(&self) -> String {
        canonical_resource_request_identity(self)
    }
}
