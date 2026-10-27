use super::super::BridgeOwnedSignalRuntime;
use super::{
    BridgeOwnedAsyncManagedDenial as Denial, BridgeOwnedAsyncManagedRequestAdmission as Custodian,
};
use worth_signal::facade::branch::{
    SignalOwnedAsyncManagedProgress, SignalOwnedAsyncManagedReport,
};

/// Native coalescing retains the original business source; it is not a new read.
pub enum BridgeOwnedAsyncManagedRevalidation {
    Coalesced,
    Denied,
    Replaced(crate::facade::BridgeAsyncRevalidationLineage),
}

impl BridgeOwnedSignalRuntime {
    pub fn revalidate_managed_owned_async_request(
        &self,
        owner: &mut Custodian,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: crate::facade::BridgeAsyncRequestTruthViewBasis,
    ) -> Result<BridgeOwnedAsyncManagedRevalidation, Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        let business = owner
            .business
            .as_ref()
            .ok_or(Denial::ProvenanceUnavailable)?;
        if business.basis_binding().truth_view_basis().digest() == truth_basis.digest() {
            return Err(Denial::ProgressDenied);
        }
        let prior = owner
            .request
            .as_ref()
            .map(|request| request.request().clone());
        let bound = self
            .bind_owned_async_basis(business.lowered(), signal_basis, truth_basis)
            .map_err(Denial::Request)?;
        let intent = worth_signal::facade::ResourceRevalidationIntent::with_expected_active(
            business
                .lowered()
                .resource_descriptor()
                .expect("the installed managed declaration retains its resource descriptor")
                .node(),
            owner.request_handle(),
        );
        let progress = bound
            .port
            .revalidate_managed_owned_async_request(&bound.source, &mut owner.native, intent)
            .map_err(Denial::Signal)?;
        let business =
            (progress == SignalOwnedAsyncManagedProgress::Replaced).then_some(bound.request);
        self.retain_managed_native_progress(owner, bound.port, business)?;
        match progress {
            SignalOwnedAsyncManagedProgress::Denied => {
                Ok(BridgeOwnedAsyncManagedRevalidation::Denied)
            }
            SignalOwnedAsyncManagedProgress::Coalesced => {
                Ok(BridgeOwnedAsyncManagedRevalidation::Coalesced)
            }
            SignalOwnedAsyncManagedProgress::Replaced => {
                let prior = prior.ok_or(Denial::RequestUnavailable)?;
                let current = owner.request.as_ref().ok_or(Denial::RequestUnavailable)?;
                let SignalOwnedAsyncManagedReport::Revalidation(report) = owner.native.report()
                else {
                    unreachable!("native revalidation replacement retains its report")
                };
                crate::source::admit_owned_revalidation_lineage(
                    prior,
                    current.request().clone(),
                    report,
                )
                .map(BridgeOwnedAsyncManagedRevalidation::Replaced)
                .map_err(|cause| {
                    owner.refusal = Some(Denial::Lineage(cause.clone()));
                    Denial::Lineage(cause)
                })
            }
            _ => unreachable!("revalidation returns only its actual resource decision"),
        }
    }
}
