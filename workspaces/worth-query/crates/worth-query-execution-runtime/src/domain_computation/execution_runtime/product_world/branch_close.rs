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
    AlreadyClosed,
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

pub enum WorthQueryProductBranchCloseScope {
    Workspace,
    Application,
}

pub struct WorthQueryPendingProductBranchClose {
    runtime: WorthQueryProductRuntime,
    branch: WorthQueryProductBranch,
    occurrence: WorthQueryRetiredProductOccurrence,
    cleanup_identity: u64,
}

impl WorthQueryPendingProductBranchClose {
    pub const fn occurrence(&self) -> &WorthQueryRetiredProductOccurrence {
        &self.occurrence
    }

    pub fn into_cleanup(self) -> (WorthQueryProductBranch, WorthQueryProductBranchOwnerCleanup) {
        (
            self.branch,
            WorthQueryProductBranchOwnerCleanup::new(self.runtime, self.cleanup_identity),
        )
    }

    pub fn finish(
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
        self.begin_product_branch_close_inner(branch, scope, None)
    }

    pub(crate) fn begin_product_branch_close_with_request(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial> {
        self.begin_product_branch_close_inner(branch, scope, Some(execution))
    }

    fn begin_product_branch_close_inner(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
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
        let report = match execution {
            Some(execution) => self.retire_product_branch_with_request(&observed, execution),
            None => self.retire_product_branch(&observed),
        }
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
        RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchRetirementDenial::StaleIncarnation) => {
            WorthQueryProductBranchCloseDenial::Selection(
                crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(
                    crate::basis::WorthQueryProductBranchAdmissionRefusalKind::StaleIncarnation,
                ),
            )
        }
        RuntimeWorldServiceDenial::OwnerUnavailable(_)
        | RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchRetirementDenial::OwnerUnavailable) => {
            WorthQueryProductBranchCloseDenial::OwnerUnavailable
        }
        RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchRetirementDenial::AlreadyRetired) => {
            WorthQueryProductBranchCloseDenial::AlreadyClosed
        }
        RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchRetirementDenial::CapacityExhausted,
        ) => WorthQueryProductBranchCloseDenial::CapacityExhausted,
    }
}

mod court_doors_for_test;
