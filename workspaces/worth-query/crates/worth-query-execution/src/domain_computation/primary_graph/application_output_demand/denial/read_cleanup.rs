//! A read request's transferred native cleanup custody and why it can remain undisposed.
use super::recovery_release::WorthQueryProductRecoveryClose;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;

/// Why a read request's native cleanup or recovery custody remains undisposed.
#[derive(Debug)]
pub enum WorthQueryOutputReadCloseDenial {
    Execution(crate::domain_computation::request_execution::WorthQueryAdvancementDenial),
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
    pub(in crate::domain_computation::primary_graph) custody: crate::domain_computation::primary_graph::application_output_demand::denial::cleanup_custody::DemandCleanupCustody,
}
impl WorthQueryOutputReadCleanup {
    pub(in crate::domain_computation::primary_graph) fn from_custody(
        custody: crate::domain_computation::primary_graph::application_output_demand::denial::cleanup_custody::DemandCleanupCustody,
    ) -> Self {
        Self { custody }
    }
    pub fn producer_outcome(
        &mut self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted> {
        self.custody.outcome()
    }
    pub fn close_in_advancement<R: WorthQueryProductRecoveryClose + ?Sized>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &R,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        self.custody.close_read(phase, &mut |recovery| {
            runtime.close_recovery_in_advancement(phase, recovery)
        })
    }
    pub fn retired_component_count(&mut self) -> usize {
        self.custody.retired_components()
    }
    pub fn receipt(
        &mut self,
    ) -> Option<&crate::domain_computation::vocabulary::managed_run::inspection::WorthQueryDirectRunCleanupReceipt> {
        self.custody.receipt()
    }
}
