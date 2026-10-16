//! Explicit disposal of caller read custody, never completion of a source mutation.
use super::*;
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};

/// Why a read request's native cleanup or recovery custody remains undisposed.
#[derive(Debug)]
pub enum WorthQueryOutputReadCloseDenial {
    Execution(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    ForeignDemand,
    NativePublicationRetained,
    ProducerRecoveryRetained,
    Recovery(crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseDenial),
    Owner(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupDenial),
    Cleanup(WorthQueryOutputDemandDenial),
}
/// The original native cleanup owner transferred by a program producer refusal.
/// Physical cleanup and its original business outcome remain paired on every retry.
#[derive(Debug)]
#[must_use = "native producer cleanup and recovery must remain owned until discharged"]
pub struct WorthQueryOutputReadCleanup {
    pub(super) custody: denial::cleanup_custody::DemandCleanupCustody,
}
impl WorthQueryOutputReadCleanup {
    pub(super) fn from_custody(custody: denial::cleanup_custody::DemandCleanupCustody) -> Self {
        Self { custody }
    }
    pub fn producer_outcome(
        &mut self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted> {
        self.custody.outcome()
    }
    pub fn close_in_advancement<S: ApplicationSchema>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        self.custody.close_read(phase, &mut |recovery| {
            runtime.close_read_product_publication_recovery_in_advancement(phase, recovery)
        })
    }
    pub fn retired_component_count(&mut self) -> usize {
        self.custody.retired_components()
    }
    pub fn receipt(
        &mut self,
    ) -> Option<&crate::domain_computation::WorthQueryDirectRunCleanupReceipt> {
        self.custody.receipt()
    }
}
impl<S: ApplicationSchema + 'static, F: WorthQueryProducerOutputFamily<S>>
    WorthQueryAdmittedOutputDemand<S, F>
{
    pub(in crate::domain_computation::primary_graph) fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        phase
            .checked_execution_request_for(&runtime.product_runtime)
            .map_err(WorthQueryOutputReadCloseDenial::Execution)?;
        if self.runtime_authority != runtime.runtime.authority_identity().as_u64()
            || self.schema_binding != runtime.installed_schema.binding_identity()
            || self
                .observed_source
                .selected_product_occurrence()
                .is_some_and(|original| original != branch.occurrence())
        {
            return Err(WorthQueryOutputReadCloseDenial::ForeignDemand);
        }
        if let Some(denial) = self.terminal_caller_denial.as_mut() {
            denial.close_read_cleanup(phase, runtime)?;
        }
        if self.unpublished_selected_checkpoint.is_some() {
            return Err(WorthQueryOutputReadCloseDenial::NativePublicationRetained);
        }
        if let Some(interest) = self.interest.as_ref() {
            interest.require_read_closeable()?;
        }
        self.required_continuations
            .close_read_request(phase, runtime, branch)?;
        // The visitor has discharged every owned successor; close only these interests.
        self.close();
        Ok(())
    }
}
