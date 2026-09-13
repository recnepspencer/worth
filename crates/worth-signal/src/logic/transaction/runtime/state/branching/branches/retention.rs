use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdmissionLease, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionBinding, SignalBranchRetentionLease, SignalBranchRetentionTerminalCounts,
};
use crate::state::SignalBranchId;

use super::catalog::BranchManager;

impl<D, I, T> BranchManager<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn acquire_admitted_retention(
        &self,
        branch_id: SignalBranchId,
    ) -> Result<SignalBranchAdmissionLease, SignalBranchRetentionAcquisitionDenial> {
        self.retention().acquire_admitted(branch_id)
    }

    /// Open one external obligation over the exact target the descriptor names.
    pub fn acquire_retention(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchRetentionLease, SignalBranchRetentionAcquisitionDenial> {
        self.retention().acquire_external(basis)
    }

    /// A narrow, cloneable binding used to decide how a presented obligation
    /// relates to this owner. It grants no retention capability itself.
    pub fn retention_binding(&self) -> SignalBranchRetentionBinding {
        self.retention().binding()
    }

    pub fn retention_terminal_counts(&self) -> SignalBranchRetentionTerminalCounts {
        self.retention().terminal_counts()
    }

    pub fn branch_admitted_retention_count(&self, branch_id: SignalBranchId) -> u32 {
        self.retention().admitted_count(branch_id)
    }

    pub fn branch_retention_count(&self, branch_id: SignalBranchId) -> u32 {
        self.retention().external_count(branch_id)
    }

    fn retention(&self) -> &crate::branch::SignalBranchRetentionRegistry {
        self.retention
            .as_ref()
            .expect("legacy branch operations are unavailable after owner-service sealing")
    }
}
