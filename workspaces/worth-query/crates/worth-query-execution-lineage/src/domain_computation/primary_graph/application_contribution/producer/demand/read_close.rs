//! Explicit disposal of caller read custody, never completion of a source mutation.
use super::denial;
use super::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::ports::WorthQueryReadCleanupRuntime;

/// Why a read request's native cleanup or recovery custody remains undisposed.
#[derive(Debug)]
pub enum WorthQueryOutputReadCloseDenial {
    Execution(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    ForeignDemand,
    NativePublicationRetained,
    ProducerRecoveryRetained,
    Recovery(crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseDenial),
    Owner(
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupDenial,
    ),
    Cleanup(WorthQueryOutputDemandDenial),
}

/// The original native cleanup owner transferred by a program producer refusal.
/// Physical cleanup and its original business outcome remain paired on every retry.
#[derive(Debug)]
#[must_use = "native producer cleanup and recovery must remain owned until discharged"]
pub struct WorthQueryOutputReadCleanup {
    pub custody: denial::cleanup_custody::DemandCleanupCustody,
}

impl WorthQueryOutputReadCleanup {
    pub fn from_custody(custody: denial::cleanup_custody::DemandCleanupCustody) -> Self {
        Self { custody }
    }

    pub fn producer_outcome(
        &mut self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome> {
        self.custody.outcome()
    }

    pub fn close_in_advancement<R: WorthQueryReadCleanupRuntime>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &R,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        self.custody.close_read(phase, &mut |recovery| {
            runtime.close_product_publication_recovery_in_advancement(phase, recovery)
        })
    }

    pub fn retired_component_count(&mut self) -> usize {
        self.custody.retired_components()
    }

    pub fn receipt_mut(
        &mut self,
    ) -> Option<
        &mut crate::domain_computation::primary_graph::WorthQueryApplicationManagedCleanupReceipt,
    > {
        self.custody.receipt_mut()
    }

    pub fn receipt(
        &mut self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryApplicationManagedCleanupReceipt>
    {
        self.custody.receipt()
    }
}
