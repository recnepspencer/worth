//! Accept a reading only at this actual installed process-local source's cell.
use super::super::BridgeOwnedSignalRuntime;
use super::admission::request_denial;
use crate::facade::{
    BridgeAsyncRequestIdentityRejection, BridgeAsyncRequestTruthViewBasis,
    LoweredBridgeAsyncSourceDeclaration,
};

impl BridgeOwnedSignalRuntime {
    pub fn advance_owned_async_source_clock(
        &self,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth: BridgeAsyncRequestTruthViewBasis,
        coordinate: u64,
    ) -> Result<
        worth_signal::facade::TemporalClockAdvanceSummary,
        BridgeAsyncRequestIdentityRejection,
    > {
        if !self.owned_async_uses_process_local_retention(declaration) {
            return Err(request_denial(
                "named resource clock requires installed process-local custody",
            ));
        }
        let bound = self.bind_owned_async_basis(declaration, basis, truth)?;
        bound
            .port
            .advance_owned_async_clock(&bound.source, coordinate)
            .map_err(|cause| request_denial(format!("resource cell clock refused: {cause:?}")))
    }
}
