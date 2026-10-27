//! Initial unqueued custody keeps shared-winner provenance and own cleanup apart.
use super::super::BridgeOwnedSignalRuntime;
use super::admission::request_denial;
use super::{
    BridgeOwnedAsyncManagedDenial as Denial, BridgeOwnedAsyncManagedRequestAdmission as Custodian,
};
use crate::facade::{BridgeAsyncRequestTruthViewBasis, LoweredBridgeAsyncSourceDeclaration};
use std::sync::Arc;

impl BridgeOwnedSignalRuntime {
    pub fn admit_unqueued_owned_async_request_identity(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<Custodian, crate::facade::BridgeAsyncRequestIdentityRejection> {
        if self.owned_async_queue_capacity(lowered).is_some() {
            return Err(request_denial(
                "managed declaration requires managed queue admission",
            ));
        }
        let bound = self.bind_owned_async_basis(lowered, signal_basis, truth_basis)?;
        let native = bound
            .port
            .admit_unqueued_owned_async_request(&bound.source)
            .map_err(|cause| request_denial(format!("Signal request denied: {cause:?}")))?;
        let (business, refusal) = if native.admitted_current_occurrence() {
            (Some(bound.request), None)
        } else {
            (None, Some(Denial::ProvenanceUnavailable))
        };
        let mut owner = Custodian {
            authority: Arc::clone(&self.async_observation_authority),
            port: bound.port,
            native,
            business,
            request: None,
            refusal: None,
        };
        self.refresh_managed_owned_async_receipt(&mut owner);
        if refusal.is_some() {
            owner.refusal = refusal;
        }
        Ok(owner)
    }

    /// Retires only work created by this unqueued entry. Joining a winner does
    /// not implicitly cancel it. This says nothing about an external provider.
    pub fn finish_unqueued_owned_async_entry_cleanup(
        &self,
        owner: &mut Custodian,
    ) -> Result<(), Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        if owner.native.requires_managed_queue() {
            return Err(Denial::QueueUnavailable);
        }
        if owner.native.admitted_current_occurrence() {
            self.cancel_managed_owned_async_request(owner)?;
        }
        owner
            .port
            .finish_managed_owned_async_wake_cleanup(&mut owner.native)
            .map_err(Denial::Service)
    }
}
