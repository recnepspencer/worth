//! The one operation closing a read's native cleanup needs from the runtime that owns the
//! product publication recovery, so the refusal owner never names the application runtime.
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

/// Closes one original product publication recovery inside the caller's advancement,
/// without new business admission.
pub trait WorthQueryProductRecoveryClose {
    fn close_recovery_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryProductUnpublishedRecovery,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductUnpublishedRecoveryReleaseFailure,
    >;
}
