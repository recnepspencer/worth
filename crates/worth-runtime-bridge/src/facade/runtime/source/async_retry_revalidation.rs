use super::*;

impl RuntimeBridge {
    /// Revalidates one admitted request-response source against a newer truth
    /// basis and returns the Bridge-issued forward-causality lineage.
    pub fn revalidate_async_request(
        &self,
        prior: &AdmittedBridgeAsyncRequestIdentity,
        current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<BridgeAsyncRevalidationLineage, BridgeAsyncForwardCausalityRejection> {
        crate::source::validate_lineage_runtime(self.signal_runtime_key, prior)?;
        let report = crate::source::with_async_request_signal_runtime(
            self.signal_runtime_key,
            |signal_runtime| {
                signal_runtime.revalidate_resource_node(
                    worth_signal::facade::ResourceRevalidationIntent::with_expected_active(
                        worth_signal::facade::ResourceNodeId::from_node(
                            prior.in_flight_identity().in_flight().node().node(),
                        ),
                        prior.request_handle(),
                    ),
                )
            },
        )
        .map_err(|error| {
            BridgeAsyncForwardCausalityRejection::new(
                BridgeAsyncForwardCausalityRejectionKind::SignalRuntimeThreadAffinityViolation,
                format!(
                    "bridge async revalidation runtime {} is already bound to thread {:?} and cannot admit from thread {:?}",
                    error.runtime_key(),
                    error.owner(),
                    error.current()
                ),
            )
        })?
        .map_err(|error| {
            BridgeAsyncForwardCausalityRejection::new(
                BridgeAsyncForwardCausalityRejectionKind::RevalidationAdmissionMissing,
                format!("Signal rejected async request revalidation: {error:?}"),
            )
        })?;
        self.admit_async_revalidation_lineage(
            BridgeAsyncRevalidationLineageRequest::request_response(
                prior,
                current_truth_view_basis,
                &report,
            ),
        )
    }

    /// Classifies one newer request as a retry lineage after timeout by
    /// consuming the authoritative Signal timeout and retry admission evidence.
    pub fn admit_async_retry_lineage_after_timeout(
        &self,
        request: BridgeAsyncRetryLineageRequest,
    ) -> Result<BridgeAsyncRetryLineage, BridgeAsyncForwardCausalityRejection> {
        request.validate_runtime(self.signal_runtime_key)?;
        crate::source::with_async_request_signal_runtime(self.signal_runtime_key, |signal_runtime| {
            crate::source::admit_retry_lineage(signal_runtime, request)
        })
        .map_err(|error| {
            BridgeAsyncForwardCausalityRejection::new(
                BridgeAsyncForwardCausalityRejectionKind::SignalRuntimeThreadAffinityViolation,
                format!(
                    "bridge async forward causality runtime {} is already bound to thread {:?} and cannot admit from thread {:?}",
                    error.runtime_key(),
                    error.owner(),
                    error.current()
                ),
            )
        })?
    }

    /// Classifies one newer request as a retry lineage after cancellation by
    /// consuming the authoritative Signal cancellation and retry admission evidence.
    pub fn admit_async_retry_lineage_after_cancellation(
        &self,
        request: BridgeAsyncRetryLineageRequest,
    ) -> Result<BridgeAsyncRetryLineage, BridgeAsyncForwardCausalityRejection> {
        request.validate_runtime(self.signal_runtime_key)?;
        crate::source::with_async_request_signal_runtime(self.signal_runtime_key, |signal_runtime| {
            crate::source::admit_retry_lineage(signal_runtime, request)
        })
        .map_err(|error| {
            BridgeAsyncForwardCausalityRejection::new(
                BridgeAsyncForwardCausalityRejectionKind::SignalRuntimeThreadAffinityViolation,
                format!(
                    "bridge async forward causality runtime {} is already bound to thread {:?} and cannot admit from thread {:?}",
                    error.runtime_key(),
                    error.owner(),
                    error.current()
                ),
            )
        })?
    }

    /// Classifies one newer request as a revalidation lineage by consuming the
    /// authoritative Signal revalidation admission evidence.
    pub fn admit_async_revalidation_lineage(
        &self,
        request: BridgeAsyncRevalidationLineageRequest,
    ) -> Result<BridgeAsyncRevalidationLineage, BridgeAsyncForwardCausalityRejection> {
        crate::source::validate_lineage_runtime(self.signal_runtime_key, &request.prior)?;
        crate::source::with_async_request_signal_runtime(self.signal_runtime_key, |signal_runtime| {
            crate::source::admit_revalidation_lineage(signal_runtime, request)
        })
        .map_err(|error| {
            BridgeAsyncForwardCausalityRejection::new(
                BridgeAsyncForwardCausalityRejectionKind::SignalRuntimeThreadAffinityViolation,
                format!(
                    "bridge async forward causality runtime {} is already bound to thread {:?} and cannot admit from thread {:?}",
                    error.runtime_key(),
                    error.owner(),
                    error.current()
                ),
            )
        })?
    }
}
