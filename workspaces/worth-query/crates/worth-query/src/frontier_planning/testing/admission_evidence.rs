use super::{
    FrontierAwarePlan, FrontierPostureDigest, FrontierPredictionDriftOutcome, FrontierSurfaceDigest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SerialFallbackReason {
    DeterministicAdmissionDenied,
    PredictionDriftRequiresSerialRoute,
    SerialExecutor,
    BelowMinStageWidth,
    BelowPolicyWorkThreshold,
    ValidationHeavyStage,
    BelowFullParallelThreshold,
    FullParallelUnsupportedByMutableEngine,
}

impl SerialFallbackReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeterministicAdmissionDenied => "deterministic_admission_denied",
            Self::PredictionDriftRequiresSerialRoute => "prediction_drift_requires_serial_route",
            Self::SerialExecutor => "serial_executor",
            Self::BelowMinStageWidth => "below_min_stage_width",
            Self::BelowPolicyWorkThreshold => "below_policy_work_threshold",
            Self::ValidationHeavyStage => "validation_heavy_stage",
            Self::BelowFullParallelThreshold => "below_full_parallel_threshold",
            Self::FullParallelUnsupportedByMutableEngine => {
                "full_parallel_unsupported_by_mutable_engine"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerialFallbackEvidence {
    basis_digest: String,
    surface_digest: FrontierSurfaceDigest,
    drift_outcome: FrontierPredictionDriftOutcome,
    serial_fallback_reason: SerialFallbackReason,
}

impl SerialFallbackEvidence {
    pub(crate) fn from_surface(
        basis_digest: impl Into<String>,
        surface_digest: FrontierSurfaceDigest,
        reason: SerialFallbackReason,
        drift_outcome: FrontierPredictionDriftOutcome,
    ) -> Self {
        Self {
            basis_digest: basis_digest.into(),
            surface_digest,
            drift_outcome,
            serial_fallback_reason: reason,
        }
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn surface_digest(&self) -> &FrontierSurfaceDigest {
        &self.surface_digest
    }

    pub fn drift_outcome(&self) -> &FrontierPredictionDriftOutcome {
        &self.drift_outcome
    }

    pub fn reason(&self) -> &SerialFallbackReason {
        &self.serial_fallback_reason
    }

    pub(in crate::frontier_planning::testing) fn route_posture_digest(
        &self,
        frontier_plan: &FrontierAwarePlan,
    ) -> FrontierPostureDigest {
        FrontierPostureDigest::from_parts(&[
            format!(
                "frontier_plan_posture:{}",
                frontier_plan.report().posture_digest().as_str()
            ),
            format!("evidence_basis:{}", self.basis_digest),
            format!("frontier_surface:{}", self.surface_digest.as_str()),
            format!("drift_outcome:{}", self.drift_outcome.as_str()),
            format!(
                "serial_fallback_reason:{}",
                self.serial_fallback_reason.as_str()
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerialFallbackBundleEvidence {
    basis_digest: String,
    bundle_surface_digest: FrontierSurfaceDigest,
    route_evidences: Vec<SerialFallbackEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SerialFallbackBundleEvidenceError {
    EmptyRouteEvidence,
    MixedBasisDigest {
        expected_basis_digest: String,
        found_basis_digest: String,
    },
}

impl SerialFallbackBundleEvidence {
    pub(crate) fn from_routes(
        bundle_surface_digest: FrontierSurfaceDigest,
        route_evidences: Vec<SerialFallbackEvidence>,
    ) -> Result<Self, SerialFallbackBundleEvidenceError> {
        let first = route_evidences
            .first()
            .ok_or(SerialFallbackBundleEvidenceError::EmptyRouteEvidence)?;
        let expected_basis_digest = first.basis_digest().to_string();
        for route in route_evidences.iter().skip(1) {
            let found_basis_digest = route.basis_digest();
            if found_basis_digest != expected_basis_digest {
                return Err(SerialFallbackBundleEvidenceError::MixedBasisDigest {
                    expected_basis_digest,
                    found_basis_digest: found_basis_digest.to_string(),
                });
            }
        }

        Ok(Self {
            basis_digest: expected_basis_digest,
            bundle_surface_digest,
            route_evidences,
        })
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn bundle_surface_digest(&self) -> &FrontierSurfaceDigest {
        &self.bundle_surface_digest
    }

    pub fn route_evidences(&self) -> &[SerialFallbackEvidence] {
        &self.route_evidences
    }
}
