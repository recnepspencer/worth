//! A read request's transferred native cleanup custody and why it can remain undisposed.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
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
    pub(super) custody: crate::domain_computation::primary_graph::application_output_demand::denial::cleanup_custody::DemandCleanupCustody,
}
impl WorthQueryOutputReadCleanup {
    pub(super) fn from_custody(custody: crate::domain_computation::primary_graph::application_output_demand::denial::cleanup_custody::DemandCleanupCustody) -> Self {
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
            runtime.close_product_publication_recovery_in_advancement(phase, recovery)
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
