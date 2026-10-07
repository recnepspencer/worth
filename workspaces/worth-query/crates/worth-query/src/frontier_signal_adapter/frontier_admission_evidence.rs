#[cfg(test)]
use crate::frontier_planning::{
    FrontierPredictionDriftOutcome, SerialFallbackEvidence, SerialFallbackReason,
};
#[cfg(test)]
use worth_signal::facade::adapters::{
    FrontierRouteEvidenceReceipt, FrontierRouteSerialFallbackReason,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalAdmissionEvidenceError {
    MissingSerialFallbackReason,
    ParallelStageUnsupported,
    RouteCountMismatch {
        surfaces: usize,
        route_receipts: usize,
    },
}

#[cfg(test)]
use super::frontier_surface_model::SignalFrontierSurfaceEvidence;

#[cfg(test)]
impl SignalFrontierSurfaceEvidence {
    #[cfg(test)]
    pub(crate) fn to_serial_fallback_evidence(
        &self,
        basis_digest: &str,
        reason: SerialFallbackReason,
        drift_outcome: FrontierPredictionDriftOutcome,
    ) -> SerialFallbackEvidence {
        SerialFallbackEvidence::from_surface(
            basis_digest,
            self.surface_digest().clone(),
            reason,
            drift_outcome,
        )
    }

    #[cfg(test)]
    pub(crate) fn to_route_evidence_from_stage_record(
        &self,
        basis_digest: &str,
        route_receipt: &FrontierRouteEvidenceReceipt,
    ) -> Result<SerialFallbackEvidence, SignalAdmissionEvidenceError> {
        if route_receipt.is_parallel_admitted() {
            return Err(SignalAdmissionEvidenceError::ParallelStageUnsupported);
        }
        let reason = route_receipt
            .serial_fallback_reason()
            .ok_or(SignalAdmissionEvidenceError::MissingSerialFallbackReason)?;

        Ok(self.to_serial_fallback_evidence(
            basis_digest,
            serial_fallback_reason_from_signal(reason),
            FrontierPredictionDriftOutcome::WithinBudget,
        ))
    }
}

#[cfg(test)]
fn serial_fallback_reason_from_signal(
    reason: FrontierRouteSerialFallbackReason,
) -> SerialFallbackReason {
    match reason {
        FrontierRouteSerialFallbackReason::SerialExecutor => SerialFallbackReason::SerialExecutor,
        FrontierRouteSerialFallbackReason::BelowMinStageWidth => {
            SerialFallbackReason::BelowMinStageWidth
        }
        FrontierRouteSerialFallbackReason::BelowPolicyWorkThreshold => {
            SerialFallbackReason::BelowPolicyWorkThreshold
        }
        FrontierRouteSerialFallbackReason::ValidationHeavyStage => {
            SerialFallbackReason::ValidationHeavyStage
        }
        FrontierRouteSerialFallbackReason::BelowFullParallelThreshold => {
            SerialFallbackReason::BelowFullParallelThreshold
        }
        FrontierRouteSerialFallbackReason::FullParallelUnsupportedByMutableEngine => {
            SerialFallbackReason::FullParallelUnsupportedByMutableEngine
        }
    }
}
