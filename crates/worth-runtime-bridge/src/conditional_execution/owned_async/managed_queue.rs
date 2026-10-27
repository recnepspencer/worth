use super::super::BridgeOwnedSignalRuntime;
use super::admission::{request_denial, BridgeBoundOwnedAsyncBasis};
use super::{
    BridgeOwnedAsyncManagedDenial as Denial, BridgeOwnedAsyncManagedRequestAdmission as Custodian,
};
use crate::facade::{BridgeAsyncRequestTruthViewBasis, LoweredBridgeAsyncSourceDeclaration};
use std::sync::Arc;
use worth_signal::facade::branch::SignalOwnedAsyncManagedAdmission;

impl BridgeOwnedSignalRuntime {
    pub fn observe_managed_owned_async_safe_point(
        &self,
        request: &Custodian,
    ) -> Result<worth_signal::facade::ResourceSafePointObservationReport, Denial> {
        request.require_owner(&self.async_observation_authority)?;
        let queue = request.queue.as_ref().ok_or(Denial::QueueUnavailable)?;
        request
            .port
            .observe_owned_async_safe_point(&request.source, queue)
            .map_err(Denial::Signal)
    }

    pub fn admit_managed_owned_async_request_identity(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<Custodian, crate::facade::BridgeAsyncRequestIdentityRejection> {
        let capacity = self
            .owned_async_queue_capacity(lowered)
            .ok_or_else(|| request_denial("declaration has no installed managed queue capacity"))?;
        let bound = self.bind_owned_async_basis(lowered, signal_basis, truth_basis)?;
        let admitted = bound
            .port
            .admit_managed_owned_async_request(&bound.source, capacity.get())
            .map_err(|denial| {
                request_denial(format!("Signal managed request denied: {denial:?}"))
            })?;
        Ok(match admitted {
            SignalOwnedAsyncManagedAdmission::Ready { request, queue } => self
                .assemble_managed_owned_async_request(
                    bound,
                    request.report().admitted_request(),
                    Some(request.in_flight().clone()),
                    Some(queue),
                    None,
                ),
            SignalOwnedAsyncManagedAdmission::CleanupRequired {
                report,
                queue,
                cause,
            } => self.assemble_managed_owned_async_request(
                bound,
                report.admitted_request(),
                None,
                queue,
                Some(Denial::Signal(cause)),
            ),
        })
    }

    pub(super) fn assemble_managed_owned_async_request(
        &self,
        bound: BridgeBoundOwnedAsyncBasis,
        admitted: worth_signal::facade::AdmittedResourceRequest,
        in_flight: Option<worth_signal::facade::InFlightResourceRequest>,
        queue: Option<worth_signal::facade::ResourceManagedQueueBinding>,
        mut refusal: Option<Denial>,
    ) -> Custodian {
        let request = if let Some(in_flight) = in_flight {
            match crate::source::admit_from_owned_signal_request(
                self.bridge.signal_runtime_key,
                bound.request,
                admitted,
                in_flight,
            ) {
                Ok(request) => Some(super::super::BridgeOwnedAsyncRequestAdmission::new(
                    &self.async_observation_authority,
                    request,
                    bound.port.clone(),
                    bound.source.clone(),
                )),
                Err(cause) => {
                    refusal = Some(Denial::Request(cause));
                    None
                }
            }
        } else {
            None
        };
        Custodian {
            authority: Arc::clone(&self.async_observation_authority),
            port: bound.port,
            source: bound.source,
            admitted,
            queue,
            request,
            refusal,
        }
    }

    pub fn enqueue_managed_owned_async_queue(
        &self,
        request: &Custodian,
        width: u64,
    ) -> Result<worth_signal::facade::ResourceManagedQueueMutationReport, Denial> {
        request.require_owner(&self.async_observation_authority)?;
        let queue = request.queue.as_ref().ok_or(Denial::QueueUnavailable)?;
        request
            .port
            .enqueue_owned_async_queue(&request.source, queue, width)
            .map_err(Denial::Signal)
    }

    pub fn dequeue_managed_owned_async_queue(
        &self,
        request: &Custodian,
        width: u64,
    ) -> Result<worth_signal::facade::ResourceManagedQueueMutationReport, Denial> {
        request.require_owner(&self.async_observation_authority)?;
        let queue = request.queue.as_ref().ok_or(Denial::QueueUnavailable)?;
        request
            .port
            .dequeue_owned_async_queue(&request.source, queue, width)
            .map_err(Denial::Signal)
    }

    /// Cancels only the real native request; it does not attest to queue/provider cleanup.
    pub fn cancel_managed_owned_async_request(
        &self,
        request: &Custodian,
    ) -> Result<worth_signal::facade::ResourceCancellationReport, Denial> {
        request.require_owner(&self.async_observation_authority)?;
        request
            .port
            .cancel_owned_async_request(&request.source, request.admitted.handle())
            .map_err(Denial::Service)
    }
}
