//! Exact Product World branch retirement with bounded Query cleanup custody.

use worth_runtime_world::facade::{RuntimeWorldBranchRetirementDenial, RuntimeWorldServiceDenial};

use super::{
    WorthQueryProductBranchOwnerCleanup, WorthQueryProductBranchOwnerCleanupFailure,
    WorthQueryProductRuntime, WorthQueryRetiredProductOccurrence,
};
use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial};

#[derive(Debug)]
pub enum WorthQueryProductBranchCloseDenial {
    Selection(WorthQueryProductBranchAdmissionDenial),
    OwnerUnavailable,
    UnknownBranch,
    AlreadyClosed,
    RetentionStillRequired,
    CapacityExhausted,
    CleanupCapacityExhausted,
    OwnerCleanupPending(WorthQueryProductBranchOwnerCleanupFailure),
}

#[derive(Debug)]
pub struct WorthQueryProductBranchCloseReceipt {
    branch: WorthQueryProductBranch,
    retired_component_count: usize,
}

impl WorthQueryProductBranchCloseReceipt {
    pub(crate) fn from_owner_cleanup(
        branch: WorthQueryProductBranch,
        cleanup: super::WorthQueryProductBranchOwnerCleanupReceipt,
    ) -> Self {
        Self {
            branch,
            retired_component_count: cleanup.retired_component_count(),
        }
    }

    pub const fn product_branch(&self) -> WorthQueryProductBranch {
        self.branch
    }

    pub const fn retired_component_count(&self) -> usize {
        self.retired_component_count
    }

    pub const fn is_complete(&self) -> bool {
        true
    }
}

pub(crate) enum WorthQueryProductBranchCloseScope {
    Workspace,
    Application,
}

pub(crate) struct WorthQueryPendingProductBranchClose {
    runtime: WorthQueryProductRuntime,
    branch: WorthQueryProductBranch,
    occurrence: WorthQueryRetiredProductOccurrence,
    cleanup_identity: u64,
}

impl WorthQueryPendingProductBranchClose {
    pub(crate) const fn occurrence(&self) -> &WorthQueryRetiredProductOccurrence {
        &self.occurrence
    }

    pub(crate) fn into_cleanup(
        self,
    ) -> (WorthQueryProductBranch, WorthQueryProductBranchOwnerCleanup) {
        (
            self.branch,
            WorthQueryProductBranchOwnerCleanup::new(self.runtime, self.cleanup_identity),
        )
    }

    pub(crate) fn finish(
        self,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryProductBranchCloseDenial> {
        let (branch, cleanup) = self.into_cleanup();
        cleanup
            .retry()
            .map(|receipt| WorthQueryProductBranchCloseReceipt::from_owner_cleanup(branch, receipt))
            .map_err(WorthQueryProductBranchCloseDenial::OwnerCleanupPending)
    }
}

impl WorthQueryProductRuntime {
    pub(crate) fn begin_product_branch_close(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
    ) -> Result<WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial> {
        let observed = self
            .admit_product_occurrence(branch.occurrence())
            .map_err(WorthQueryProductBranchCloseDenial::Selection)?;
        let occurrence = WorthQueryRetiredProductOccurrence::new(
            observed.branch_identity().clone(),
            observed.observation().lifecycle_incarnation(),
        );
        let cleanup = match scope {
            WorthQueryProductBranchCloseScope::Workspace => {
                self.reserve_owner_cleanup_for_workspace_retirement()
            }
            WorthQueryProductBranchCloseScope::Application => {
                self.reserve_owner_cleanup_for_application_retirement(occurrence.clone())
            }
        }
        .map_err(|_| WorthQueryProductBranchCloseDenial::CleanupCapacityExhausted)?;
        let report = self
            .retire_product_branch(&observed)
            .map_err(map_retirement_denial)?;
        drop(observed);
        Ok(WorthQueryPendingProductBranchClose {
            runtime: self.clone(),
            branch,
            occurrence,
            cleanup_identity: cleanup.install_retirement(report),
        })
    }
}

fn map_retirement_denial(
    denial: RuntimeWorldServiceDenial<RuntimeWorldBranchRetirementDenial>,
) -> WorthQueryProductBranchCloseDenial {
    match denial {
        RuntimeWorldServiceDenial::OwnerUnavailable(_)
        | RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchRetirementDenial::OwnerUnavailable) => {
            WorthQueryProductBranchCloseDenial::OwnerUnavailable
        }
        RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchRetirementDenial::UnknownBranch) => {
            WorthQueryProductBranchCloseDenial::UnknownBranch
        }
        RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchRetirementDenial::AlreadyRetired) => {
            WorthQueryProductBranchCloseDenial::AlreadyClosed
        }
        RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchRetirementDenial::RetentionStillRequired,
        ) => WorthQueryProductBranchCloseDenial::RetentionStillRequired,
        RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchRetirementDenial::CapacityExhausted,
        ) => WorthQueryProductBranchCloseDenial::CapacityExhausted,
    }
}
