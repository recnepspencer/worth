use std::sync::Arc;

use crate::facade::{
    BridgeAsyncCompletionRejection, BridgeAsyncCompletionRejectionKind, BridgeMixedCauseOrdering,
    BridgeMixedCauseOrderingInput, BridgeMixedCauseOrderingLaneKind,
    BridgeMixedCauseOrderingRequest, ValidatedBridgeAsyncCompletionEnvelope,
};

use super::super::BridgeOwnedSignalRuntime;
use super::foreign_continuation_denial;

impl BridgeOwnedSignalRuntime {
    pub fn validate_owned_async_completion_envelope(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        raw: worth_signal::facade::RawCompletionEnvelope,
    ) -> Result<ValidatedBridgeAsyncCompletionEnvelope, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        self.bridge
            .validate_async_completion_envelope(request.request(), raw)
    }

    pub fn admit_owned_async_completion(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        validated: &ValidatedBridgeAsyncCompletionEnvelope,
    ) -> Result<super::super::BridgeOwnedAsyncCompletionAdmission, BridgeAsyncCompletionRejection>
    {
        self.require_owned_async_request(request)?;
        if validated.request_identity() != request.request().request_identity().as_str() {
            return Err(BridgeAsyncCompletionRejection::new(
                BridgeAsyncCompletionRejectionKind::EnvelopeHandleMismatch,
                "validated completion belongs to another owned request",
            ));
        }
        let (port, binding) = request.signal_parts();
        let report = port
            .admit_owned_async_completion(binding, validated.raw())
            .map_err(|denial| {
                BridgeAsyncCompletionRejection::new(
                    BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
                    format!("Signal completion denied: {denial:?}"),
                )
            })?;
        Ok(super::super::BridgeOwnedAsyncCompletionAdmission::new(
            &self.async_observation_authority,
            crate::source::map_owned_signal_report(
                request.request().clone(),
                validated.clone(),
                report,
            ),
        ))
    }

    pub fn admit_owned_async_effects_indeterminate(
        &self,
        observation: super::super::BridgeAsyncEffectsIndeterminateObservation,
    ) -> Result<super::super::BridgeOwnedAsyncCompletionAdmission, BridgeAsyncCompletionRejection>
    {
        let (authority, request, port, binding, raw) = observation.into_parts();
        if !Arc::ptr_eq(&authority, &self.async_observation_authority) {
            return Err(BridgeAsyncCompletionRejection::new(
                BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority,
                "effects-indeterminate observation belongs to another owned runtime",
            ));
        }
        let validated = self
            .bridge
            .validate_async_completion_envelope(&request, raw)?;
        let report = port
            .admit_owned_async_completion(&binding, validated.raw())
            .map_err(|denial| {
                BridgeAsyncCompletionRejection::new(
                    BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
                    format!("Signal completion denied: {denial:?}"),
                )
            })?;
        Ok(super::super::BridgeOwnedAsyncCompletionAdmission::new(
            &self.async_observation_authority,
            crate::source::map_owned_signal_report(request, validated, report)
                .from_owner_effects_indeterminate(
                crate::source::BridgeAsyncEffectsIndeterminateCompletion::from_owner_observation(),
            ),
        ))
    }

    pub fn order_owned_async_completion_report(
        &self,
        completion: &super::super::BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<BridgeMixedCauseOrdering, BridgeAsyncCompletionRejection> {
        if !completion.is_owned_by(&self.async_observation_authority) {
            return Err(foreign_continuation_denial());
        }
        let report = completion.report();
        let input = match (report.admitted_completion(), report.denied_completion()) {
            (Some(completion), None) => {
                BridgeMixedCauseOrderingInput::AsyncCompletion(completion.clone())
            }
            (None, Some(completion)) => {
                BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(completion.clone())
            }
            _ => unreachable!("Bridge completion report retains exactly one outcome"),
        };
        Ok(self
            .bridge
            .order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
                BridgeMixedCauseOrderingLaneKind::Authoritative,
                vec![input],
            )))
    }

    pub(super) fn require_owned_async_request(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<(), BridgeAsyncCompletionRejection> {
        if request.is_owned_by(&self.async_observation_authority) {
            Ok(())
        } else {
            Err(foreign_continuation_denial())
        }
    }
}
