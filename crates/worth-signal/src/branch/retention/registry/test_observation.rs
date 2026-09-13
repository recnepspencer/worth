use super::{SignalBranchRetentionRegistry, SignalRetentionLedger};
use crate::state::SignalBranchId;
use worth_foundational::FoundationalBranchTargetEncoding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignalRetentionLedgerObservation {
    pub(crate) maximum_active_leases: usize,
    pub(crate) used_capacity: usize,
    pub(crate) next_lease_id: u64,
    pub(crate) admitted_lease_count: usize,
    pub(crate) external_lease_count: usize,
    pub(crate) reserved_admitted_lease_count: usize,
    pub(crate) admitted_branch_total_count: usize,
    pub(crate) reserved_branch_total_count: usize,
    pub(crate) external_branch_total_count: usize,
    pub(crate) external_target_total_count: usize,
    pub(crate) admitted_lease_identities: Vec<(u64, SignalBranchId)>,
    pub(crate) external_lease_identities:
        Vec<(u64, SignalBranchId, FoundationalBranchTargetEncoding)>,
    pub(crate) admitted_count_by_branch: Vec<(SignalBranchId, u32)>,
    pub(crate) reserved_count_by_branch: Vec<(SignalBranchId, u32)>,
    pub(crate) external_count_by_branch: Vec<(SignalBranchId, u32)>,
    pub(crate) external_count_by_target: Vec<(FoundationalBranchTargetEncoding, u32)>,
}

impl SignalBranchRetentionRegistry {
    pub(crate) fn test_observation(&self) -> SignalRetentionLedgerObservation {
        self.ledger.test_observation()
    }
}

impl SignalRetentionLedger {
    fn test_observation(&self) -> SignalRetentionLedgerObservation {
        let state = self.lock();
        let used_capacity = state.admitted_leases.len()
            + state.external_leases.len()
            + state.reserved_admitted_lease_count;
        let mut admitted_lease_identities = state
            .admitted_leases
            .iter()
            .map(|(lease_id, branch_id)| (*lease_id, *branch_id))
            .collect::<Vec<_>>();
        admitted_lease_identities.sort_unstable();
        let mut external_lease_identities = state
            .external_leases
            .iter()
            .map(|(lease_id, record)| (*lease_id, record.branch_id, record.target.0.clone()))
            .collect::<Vec<_>>();
        external_lease_identities.sort_unstable();
        let mut admitted_count_by_branch = state
            .admitted_count_by_branch
            .iter()
            .map(|(branch_id, count)| (*branch_id, *count))
            .collect::<Vec<_>>();
        admitted_count_by_branch.sort_unstable();
        let mut reserved_count_by_branch = state
            .reserved_admitted_count_by_branch
            .iter()
            .map(|(branch_id, count)| (*branch_id, *count))
            .collect::<Vec<_>>();
        reserved_count_by_branch.sort_unstable();
        let mut external_count_by_branch = state
            .external_count_by_branch
            .iter()
            .map(|(branch_id, count)| (*branch_id, *count))
            .collect::<Vec<_>>();
        external_count_by_branch.sort_unstable();
        let mut external_count_by_target = state
            .external_count_by_target
            .iter()
            .map(|(target, count)| (target.0.clone(), *count))
            .collect::<Vec<_>>();
        external_count_by_target.sort_unstable();
        SignalRetentionLedgerObservation {
            maximum_active_leases: self.maximum_active_leases,
            used_capacity,
            next_lease_id: state.next_lease_id,
            admitted_lease_count: state.admitted_leases.len(),
            external_lease_count: state.external_leases.len(),
            reserved_admitted_lease_count: state.reserved_admitted_lease_count,
            admitted_branch_total_count: state
                .admitted_count_by_branch
                .values()
                .map(|count| *count as usize)
                .sum(),
            reserved_branch_total_count: state
                .reserved_admitted_count_by_branch
                .values()
                .map(|count| *count as usize)
                .sum(),
            external_branch_total_count: state
                .external_count_by_branch
                .values()
                .map(|count| *count as usize)
                .sum(),
            external_target_total_count: state
                .external_count_by_target
                .values()
                .map(|count| *count as usize)
                .sum(),
            admitted_lease_identities,
            external_lease_identities,
            admitted_count_by_branch,
            reserved_count_by_branch,
            external_count_by_branch,
            external_count_by_target,
        }
    }
}
