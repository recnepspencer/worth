//! Query-owned lifecycle entry for branches in one installed Product World.

use std::num::NonZeroUsize;

use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryCursor, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial,
};

use super::{
    WorthQueryProductBranch, WorthQueryProductBranchCreationRecovery,
    WorthQueryProductBranchRecoveryDenial,
};
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCloseDenial, WorthQueryProductBranchCloseReceipt,
    WorthQueryProductBranchCloseScope, WorthQueryProductBranchOwnerCleanup,
    WorthQueryProductRuntime,
};

/// Complete outer lifecycle for branches owned by one installed Query World.
pub struct WorthQueryProductBranches<'runtime> {
    pub(super) runtime: &'runtime WorthQueryProductRuntime,
}

impl WorthQueryProductRuntime {
    /// Query facade construction; component runtimes remain hidden.
    #[doc(hidden)]
    pub fn product_branches(&self) -> WorthQueryProductBranches<'_> {
        WorthQueryProductBranches { runtime: self }
    }
}

impl WorthQueryProductBranches<'_> {
    pub fn close(
        &self,
        branch: WorthQueryProductBranch,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryProductBranchCloseDenial> {
        self.runtime
            .begin_product_branch_close(branch, WorthQueryProductBranchCloseScope::Workspace)?
            .finish()
    }

    /// Enumerates bounded World recovery records. Readmission proves whether
    /// a row belongs to branch creation before granting cleanup authority.
    pub fn recovery_page(
        &self,
        after: Option<&RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>>
    {
        self.runtime
            .owner
            .inspection_port()
            .recovery_page(after, maximum)
    }

    pub fn readmit_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryProductBranchCreationRecovery, WorthQueryProductBranchRecoveryDenial>
    {
        let recovery = self.runtime.owner.recovery_port();
        let effects = recovery
            .inspect_effects(handle)
            .map_err(super::map_recovery_denial)?;
        if effects.destination_branch().is_none() {
            return Err(WorthQueryProductBranchRecoveryDenial::NotBranchCreation);
        }
        Ok(WorthQueryProductBranchCreationRecovery::new(
            effects,
            recovery,
            self.runtime.clone(),
        ))
    }

    /// Rediscovers outer creation and retirement cleanup only. Application
    /// retirement remains behind its primary-provider lifecycle surface.
    pub fn pending_cleanup(&self) -> Vec<WorthQueryProductBranchOwnerCleanup> {
        self.runtime.pending_workspace_owner_cleanup()
    }
}
