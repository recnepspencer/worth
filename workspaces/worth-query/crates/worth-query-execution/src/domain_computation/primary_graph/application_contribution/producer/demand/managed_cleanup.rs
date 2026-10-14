//! The admitted demand, never a diagnostic clone, owns failed producer cleanup.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCleanupRequired,
};
impl<S: ApplicationSchema, F: WorthQueryProducerOutputFamily<S>>
    WorthQueryAdmittedOutputDemand<S, F>
{
    /// Borrows the exact pending native producer terminal, without transferring release authority.
    pub fn managed_cleanup_failure(
        &mut self,
    ) -> Option<&crate::domain_computation::WorthQueryMutationCleanupFailure> {
        self.terminal_caller_denial
            .as_mut()
            .and_then(|stop| stop.managed_cleanup_failure())
    }
    /// Transfers the unique native owner while leaving terminal business diagnostics in this demand.
    pub fn take_managed_cleanup(&mut self) -> Option<Box<WorthQueryApplicationCleanupRequired>> {
        self.terminal_caller_denial
            .as_mut()
            .and_then(|stop| stop.take_managed_cleanup())
    }
    /// Discharges only native cleanup under a fresh issuer, never producer completion.
    pub fn release_managed_cleanup(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        Option<crate::domain_computation::WorthQueryDirectRunCleanupReceipt>,
        WorthQueryOutputDemandDenial,
    > {
        match self.terminal_caller_denial.as_mut() {
            Some(stop) => stop.release_managed_cleanup(phase),
            None => Ok(None),
        }
    }
}
