//! Crate-boundary internal port for `WorthQueryProductRuntime` owner-cleanup
//! operations. Never exported on `worth-query-execution::facade` or Host.

use super::{
    WorthQueryProductBranchOwnerCleanup, WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    WorthQueryProductBranchOwnerCleanupReservation, WorthQueryRetiredProductOccurrence,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

pub trait ProductRuntimeOwnerCleanupInternalPort {
    fn reserve_owner_cleanup_for_workspace_retirement(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    >;

    fn reserve_owner_cleanup_for_application_retirement(
        &self,
        occurrence: WorthQueryRetiredProductOccurrence,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    >;

    fn reserve_owner_cleanup_for_creation(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    >;

    fn reserve_owner_cleanup_for_application(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    >;

    fn pending_owner_cleanup(&self) -> Vec<WorthQueryProductBranchOwnerCleanup>;
    fn pending_workspace_owner_cleanup(&self) -> Vec<WorthQueryProductBranchOwnerCleanup>;
    fn pending_application_retired_product_occurrences(
        &self,
    ) -> Vec<WorthQueryRetiredProductOccurrence>;
    fn replace_owner_cleanup_after_install_hook(&self, hook: Option<Box<dyn Fn()>>);
}

impl ProductRuntimeOwnerCleanupInternalPort for WorthQueryProductRuntime {
    fn reserve_owner_cleanup_for_workspace_retirement(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        Self::reserve_owner_cleanup_for_workspace_retirement(self)
    }

    fn reserve_owner_cleanup_for_application_retirement(
        &self,
        occurrence: WorthQueryRetiredProductOccurrence,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        Self::reserve_owner_cleanup_for_application_retirement(self, occurrence)
    }

    fn reserve_owner_cleanup_for_creation(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        Self::reserve_owner_cleanup_for_creation(self)
    }

    fn reserve_owner_cleanup_for_application(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        Self::reserve_owner_cleanup_for_application(self)
    }

    fn pending_owner_cleanup(&self) -> Vec<WorthQueryProductBranchOwnerCleanup> {
        Self::pending_owner_cleanup(self)
    }

    fn pending_workspace_owner_cleanup(&self) -> Vec<WorthQueryProductBranchOwnerCleanup> {
        Self::pending_workspace_owner_cleanup(self)
    }

    fn pending_application_retired_product_occurrences(
        &self,
    ) -> Vec<WorthQueryRetiredProductOccurrence> {
        Self::pending_application_retired_product_occurrences(self)
    }

    fn replace_owner_cleanup_after_install_hook(&self, hook: Option<Box<dyn Fn()>>) {
        Self::replace_owner_cleanup_after_install_hook(self, hook)
    }
}
