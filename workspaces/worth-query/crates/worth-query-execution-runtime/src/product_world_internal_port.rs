//! Crate-boundary ports for internal Product World tokens, clocks, and receipts.
//! Never exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;

use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::execution_runtime::product_world::{
    OwnerCleanupRetirements, WorthQueryPerformedRelationalProductChange,
    WorthQueryProductBranchCloseReceipt, WorthQueryProductBranchOwnerCleanup,
    WorthQueryProductBranchOwnerCleanupDenial, WorthQueryProductBranchOwnerCleanupReceipt,
    WorthQueryProductPublicationReceipt, WorthQueryProductRootIdentity, WorthQueryProductRuntime,
    WorthQueryProductWorldClock,
};

pub trait ProductWorldClockInternalPort {
    fn admission_clock(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;
}

impl ProductWorldClockInternalPort for WorthQueryProductWorldClock {
    fn admission_clock(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        &self.admission_clock
    }
}

pub trait PerformedRelationalProductChangeInternalPort: Sized {
    fn from_publication(publication: WorthQueryProductPublicationReceipt) -> Self;
    fn root_identity(&self) -> &Arc<WorthQueryProductRootIdentity>;
}

impl PerformedRelationalProductChangeInternalPort for WorthQueryPerformedRelationalProductChange {
    fn from_publication(publication: WorthQueryProductPublicationReceipt) -> Self {
        Self::new(publication)
    }

    fn root_identity(&self) -> &Arc<WorthQueryProductRootIdentity> {
        &self.root_identity
    }
}

pub trait ProductBranchInternalPort: Sized {
    fn from_occurrence(occurrence: worth_runtime_world::facade::ProductBranchIncarnation) -> Self;
    fn occurrence(self) -> worth_runtime_world::facade::ProductBranchIncarnation;
}

impl ProductBranchInternalPort for WorthQueryProductBranch {
    fn from_occurrence(occurrence: worth_runtime_world::facade::ProductBranchIncarnation) -> Self {
        Self::from_occurrence(occurrence)
    }

    fn occurrence(self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        Self::occurrence(self)
    }
}

pub trait ProductBranchCloseReceiptInternalPort: Sized {
    fn from_owner_cleanup(
        branch: WorthQueryProductBranch,
        cleanup: WorthQueryProductBranchOwnerCleanupReceipt,
    ) -> Self;
}

impl ProductBranchCloseReceiptInternalPort for WorthQueryProductBranchCloseReceipt {
    fn from_owner_cleanup(
        branch: WorthQueryProductBranch,
        cleanup: WorthQueryProductBranchOwnerCleanupReceipt,
    ) -> Self {
        Self::from_owner_cleanup(branch, cleanup)
    }
}

pub trait ProductBranchOwnerCleanupInternalPort: Sized {
    fn from_runtime_identity(runtime: WorthQueryProductRuntime, identity: u64) -> Self;

    fn release_retired_history(
        &self,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial>;

    fn release_retired_history_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial>;

    fn retired_application_occurrence(
        &self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryRetiredProductOccurrence,
    >;

    fn pending_signal_bases(
        &self,
    ) -> Result<
        Vec<worth_signal::facade::branch::AdmittedSignalBranchBasis>,
        worth_signal::facade::branch::SignalBranchBasisObservationDenial,
    >;
}

impl ProductBranchOwnerCleanupInternalPort for WorthQueryProductBranchOwnerCleanup {
    fn from_runtime_identity(runtime: WorthQueryProductRuntime, identity: u64) -> Self {
        Self::new(runtime, identity)
    }

    fn release_retired_history(
        &self,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        Self::release_retired_history(self, retirements)
    }

    fn release_retired_history_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        Self::release_retired_history_with_request(self, execution, retirements)
    }

    fn retired_application_occurrence(
        &self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryRetiredProductOccurrence,
    >{
        Self::retired_application_occurrence(self)
    }

    fn pending_signal_bases(
        &self,
    ) -> Result<
        Vec<worth_signal::facade::branch::AdmittedSignalBranchBasis>,
        worth_signal::facade::branch::SignalBranchBasisObservationDenial,
    > {
        Self::pending_signal_bases(self)
    }
}
