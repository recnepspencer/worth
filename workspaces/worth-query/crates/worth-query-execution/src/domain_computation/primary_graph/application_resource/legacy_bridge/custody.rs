use super::WorthQueryOwnedAsyncRuntimeCause as Cause;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use worth_runtime_bridge::facade::{
    BridgeOwnedAsyncEffectsIndeterminateIssuer, BridgeOwnedAsyncManagedDenial,
    BridgeOwnedAsyncManagedRequestAdmission, BridgeOwnedAsyncRequestAdmission,
};

/// One caller-owned Native carrier, distinct from immutable occurrence witnesses.
/// Dropping it neither cancels a winner nor attests to provider cleanup.
pub struct WorthQueryOwnedAsyncRequestCustody {
    pub(super) runtime: WorthQueryRuntimeAuthorityIdentity,
    pub(super) graph: u64,
    pub(super) native: BridgeOwnedAsyncManagedRequestAdmission,
}

impl WorthQueryOwnedAsyncRequestCustody {
    pub fn retained_witness(
        &self,
    ) -> Result<std::sync::Arc<BridgeOwnedAsyncRequestAdmission>, Cause> {
        self.witness()?;
        Ok(self
            .native
            .retained_request()
            .expect("the checked native witness remains present"))
    }

    pub fn witness(&self) -> Result<&BridgeOwnedAsyncRequestAdmission, Cause> {
        self.native.request().ok_or_else(|| {
            Cause::Custody(
                self.native
                    .refusal()
                    .unwrap_or(BridgeOwnedAsyncManagedDenial::RequestUnavailable),
            )
        })
    }

    pub fn effects_indeterminate_issuer(
        &self,
    ) -> Result<BridgeOwnedAsyncEffectsIndeterminateIssuer, Cause> {
        self.witness()
            .map(|request| request.effects_indeterminate_issuer())
    }

    pub fn request_handle(&self) -> worth_signal::facade::ResourceRequestHandle {
        self.native.request_handle()
    }

    pub fn native_report(&self) -> &worth_signal::facade::branch::SignalOwnedAsyncManagedReport {
        self.native.native_report()
    }
}

impl std::fmt::Debug for WorthQueryOwnedAsyncRequestCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryOwnedAsyncRequestCustody")
            .field("runtime", &self.runtime)
            .field("graph", &self.graph)
            .field("report", self.native.native_report())
            .finish_non_exhaustive()
    }
}
