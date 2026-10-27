use super::super::BridgeOwnedSignalRuntime;
use super::admission::{request_denial, BridgeBoundOwnedAsyncBasis};
use super::{
    BridgeOwnedAsyncManagedDenial as Denial, BridgeOwnedAsyncManagedRequestAdmission as Custodian,
};
use crate::facade::{BridgeAsyncRequestTruthViewBasis, LoweredBridgeAsyncSourceDeclaration};
use std::sync::Arc;

impl BridgeOwnedSignalRuntime {
    pub fn observe_managed_owned_async_safe_point(
        &self,
        request: &Custodian,
    ) -> Result<worth_signal::facade::ResourceSafePointObservationReport, Denial> {
        request.require_owner(&self.async_observation_authority)?;
        let queue = request.native.queue().ok_or(Denial::QueueUnavailable)?;
        request
            .port
            .observe_owned_async_safe_point(queue)
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
        Ok(self.assemble_managed_owned_async_request(bound, admitted))
    }

    pub(super) fn assemble_managed_owned_async_request(
        &self,
        bound: BridgeBoundOwnedAsyncBasis,
        native: worth_signal::facade::branch::SignalOwnedAsyncManagedAdmission,
    ) -> Custodian {
        let mut custodian = Custodian {
            authority: Arc::clone(&self.async_observation_authority),
            port: bound.port,
            native,
            business: bound.request,
            request: None,
            refusal: None,
        };
        self.refresh_managed_owned_async_receipt(&mut custodian);
        custodian
    }

    pub(super) fn refresh_managed_owned_async_receipt(&self, owner: &mut Custodian) {
        owner.request = None;
        owner.refusal = None;
        if let Some(in_flight) = owner.native.in_flight().cloned() {
            match crate::source::admit_from_owned_signal_request(
                self.bridge.signal_runtime_key,
                owner.business.clone(),
                owner.native.admitted_request(),
                in_flight,
            ) {
                Ok(request) => {
                    owner.request = Some(super::super::BridgeOwnedAsyncRequestAdmission::new(
                        &self.async_observation_authority,
                        request,
                        owner.port.clone(),
                        owner.native.source_binding().clone(),
                    ))
                }
                Err(cause) => owner.refusal = Some(Denial::Request(cause)),
            }
        } else {
            owner.refusal = Some(Denial::RequestUnavailable);
        }
    }

    pub fn enqueue_managed_owned_async_queue(
        &self,
        request: &Custodian,
        width: u64,
    ) -> Result<worth_signal::facade::ResourceManagedQueueMutationReport, Denial> {
        request.require_owner(&self.async_observation_authority)?;
        let queue = request.native.queue().ok_or(Denial::QueueUnavailable)?;
        request
            .port
            .enqueue_owned_async_queue(queue, width)
            .map_err(Denial::Signal)
    }

    pub fn dequeue_managed_owned_async_queue(
        &self,
        request: &Custodian,
        width: u64,
    ) -> Result<worth_signal::facade::ResourceManagedQueueMutationReport, Denial> {
        request.require_owner(&self.async_observation_authority)?;
        let queue = request.native.queue().ok_or(Denial::QueueUnavailable)?;
        request
            .port
            .dequeue_owned_async_queue(queue, width)
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
            .cancel_owned_async_request(request.native.source_binding(), request.request_handle())
            .map_err(Denial::Service)
    }
}
