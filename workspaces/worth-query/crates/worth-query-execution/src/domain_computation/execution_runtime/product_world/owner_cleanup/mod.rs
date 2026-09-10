//! Managed custody for branch and unpublished-creation cleanup.

mod component;
mod history;
mod registry;

use worth_runtime_world::facade::{
    CompositeCommitIdentity, OwnerRetirementWork, ProductBranchRetirementReport,
};

use super::WorthQueryProductRuntime;
pub(crate) use registry::WorthQueryProductBranchOwnerCleanupReservation;
pub(super) use registry::{CleanupScope, WorthQueryProductBranchOwnerCleanupRegistry};

#[derive(Clone)]
pub(crate) struct WorthQueryRetiredProductOccurrence {
    branch: worth_runtime_world::facade::ProductBranchIdentity,
    incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
}

impl WorthQueryRetiredProductOccurrence {
    pub(crate) const fn new(
        branch: worth_runtime_world::facade::ProductBranchIdentity,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Self {
        Self {
            branch,
            incarnation,
        }
    }

    pub(crate) const fn branch(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        &self.branch
    }

    pub(crate) const fn incarnation(
        &self,
    ) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.incarnation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchOwnerCleanupWork {
    RelationalBranch { name: String },
    SignalBranch { name: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchOwnerCleanupDenial {
    CleanupUnavailable,
    CleanupBusy,
    WorldHistoryUnavailable,
    WorldHistoryStillRetained,
    RelationalOwnerDenied,
    RelationalOperationsStillActive,
    SignalOwnerDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryProductBranchOwnerCleanupCapacityExhausted;

#[derive(Debug)]
pub struct WorthQueryProductBranchOwnerCleanupReceipt {
    retired_component_count: usize,
}

impl WorthQueryProductBranchOwnerCleanupReceipt {
    pub const fn retired_component_count(&self) -> usize {
        self.retired_component_count
    }

    pub const fn is_complete(&self) -> bool {
        true
    }
}

/// A handle into runtime-owned exact cleanup custody. Dropping the handle does
/// not remove the obligation; `branches().pending_cleanup()` can rediscover it.
#[must_use = "owner cleanup must be completed or retained for retry"]
pub struct WorthQueryProductBranchOwnerCleanup {
    runtime: WorthQueryProductRuntime,
    identity: u64,
}

impl std::fmt::Debug for WorthQueryProductBranchOwnerCleanup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductBranchOwnerCleanup")
            .field("pending", &self.pending_work())
            .finish_non_exhaustive()
    }
}

impl WorthQueryProductBranchOwnerCleanup {
    pub(crate) fn new(runtime: WorthQueryProductRuntime, identity: u64) -> Self {
        Self { runtime, identity }
    }

    pub fn pending_work(&self) -> Vec<WorthQueryProductBranchOwnerCleanupWork> {
        self.runtime.owner_cleanup.pending_work(self.identity)
    }

    pub fn retry(
        self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductBranchOwnerCleanupFailure,
    > {
        let registry = self.runtime.owner_cleanup.clone();
        match registry.retry(self.identity, &self.runtime) {
            Ok(receipt) => Ok(receipt),
            Err(denial) => Err(WorthQueryProductBranchOwnerCleanupFailure {
                denial,
                cleanup: self,
            }),
        }
    }
}

#[derive(Debug)]
pub struct WorthQueryProductBranchOwnerCleanupFailure {
    denial: WorthQueryProductBranchOwnerCleanupDenial,
    cleanup: WorthQueryProductBranchOwnerCleanup,
}

impl WorthQueryProductBranchOwnerCleanupFailure {
    pub const fn denial(&self) -> WorthQueryProductBranchOwnerCleanupDenial {
        self.denial
    }

    pub fn into_cleanup(self) -> WorthQueryProductBranchOwnerCleanup {
        self.cleanup
    }
}

pub(super) struct WorthQueryProductBranchOwnerCleanupRecord {
    history: Option<WorthQueryProductBranchRetiredHistory>,
    pending: Vec<OwnerRetirementWork>,
    retired_component_count: usize,
}

pub(super) struct WorthQueryProductBranchRetiredHistory {
    retired_head: CompositeCommitIdentity,
    retirement_boundary: CompositeCommitIdentity,
    pending: Option<Vec<CompositeCommitIdentity>>,
}

impl WorthQueryProductBranchOwnerCleanupRecord {
    pub(super) fn from_retirement(report: ProductBranchRetirementReport) -> Self {
        let (retired_head, retirement_boundary, pending) = report.into_cleanup_parts();
        Self {
            history: retirement_boundary.map(|retirement_boundary| {
                WorthQueryProductBranchRetiredHistory {
                    retired_head,
                    retirement_boundary,
                    pending: None,
                }
            }),
            pending,
            retired_component_count: 0,
        }
    }

    pub(super) fn from_unpublished(pending: Vec<OwnerRetirementWork>) -> Self {
        Self {
            history: None,
            pending,
            retired_component_count: 0,
        }
    }

    pub(super) fn retry(
        &mut self,
        runtime: &WorthQueryProductRuntime,
        mut after_component_progress: impl FnMut(),
    ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductBranchOwnerCleanupDenial>
    {
        self.release_retired_history(runtime)?;
        while let Some(work) = self.pending.first() {
            runtime.retire_owner_component(work)?;
            self.pending.remove(0);
            self.retired_component_count += 1;
            after_component_progress();
        }
        Ok(WorthQueryProductBranchOwnerCleanupReceipt {
            retired_component_count: self.retired_component_count,
        })
    }

    fn described_work(&self) -> Vec<WorthQueryProductBranchOwnerCleanupWork> {
        self.pending.iter().map(describe_work).collect()
    }
}

impl WorthQueryProductRuntime {
    pub(crate) fn reserve_owner_cleanup_for_workspace_retirement(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        self.owner_cleanup
            .reserve(CleanupScope::WorkspaceRetirement)
    }

    pub(crate) fn reserve_owner_cleanup_for_application_retirement(
        &self,
        occurrence: WorthQueryRetiredProductOccurrence,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        self.owner_cleanup
            .reserve(CleanupScope::ApplicationRetirement(occurrence))
    }

    pub(crate) fn reserve_owner_cleanup_for_creation(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        self.owner_cleanup.reserve(CleanupScope::Creation)
    }

    pub(crate) fn reserve_owner_cleanup_for_application(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        self.owner_cleanup.reserve(CleanupScope::Application)
    }

    pub(crate) fn pending_owner_cleanup(&self) -> Vec<WorthQueryProductBranchOwnerCleanup> {
        self.owner_cleanup
            .pending_identities()
            .into_iter()
            .map(|identity| WorthQueryProductBranchOwnerCleanup::new(self.clone(), identity))
            .collect()
    }

    pub(crate) fn pending_workspace_owner_cleanup(
        &self,
    ) -> Vec<WorthQueryProductBranchOwnerCleanup> {
        self.owner_cleanup
            .pending_workspace_identities()
            .into_iter()
            .map(|identity| WorthQueryProductBranchOwnerCleanup::new(self.clone(), identity))
            .collect()
    }

    pub(crate) fn pending_application_retired_product_occurrences(
        &self,
    ) -> Vec<WorthQueryRetiredProductOccurrence> {
        self.owner_cleanup
            .pending_application_retired_product_occurrences()
    }

    #[cfg(test)]
    pub(crate) fn replace_owner_cleanup_after_install_hook(&self, hook: Option<Box<dyn Fn()>>) {
        self.owner_cleanup.replace_after_install_hook(hook);
    }

    #[cfg(test)]
    pub(crate) fn replace_owner_cleanup_after_component_progress_hook(
        &self,
        hook: Option<Box<dyn Fn()>>,
    ) {
        self.owner_cleanup
            .replace_after_component_progress_hook(hook);
    }
}

fn describe_work(work: &OwnerRetirementWork) -> WorthQueryProductBranchOwnerCleanupWork {
    match work {
        OwnerRetirementWork::RelationalBranchRetirement { target, .. } => {
            WorthQueryProductBranchOwnerCleanupWork::RelationalBranch {
                name: target.0.clone(),
            }
        }
        OwnerRetirementWork::SignalBranchRetirement { target, .. } => {
            WorthQueryProductBranchOwnerCleanupWork::SignalBranch {
                name: target.as_str().to_owned(),
            }
        }
    }
}
