use crate::state::SignalBranchId;

use super::super::accounting::{decrement_obligation_count, increment_obligation_count};
use super::super::outcome::SignalBranchRetentionAcquisitionDenial;
use super::{SignalBranchRetentionBinding, SignalRetentionLedger};

impl SignalBranchRetentionBinding {
    pub(crate) fn release_admitted(&self, lease_id: u64, branch_id: SignalBranchId) {
        self.ledger.release_admitted(lease_id, branch_id);
    }

    pub(crate) fn rebind_admitted(
        &self,
        lease_id: u64,
        previous_branch_id: SignalBranchId,
        branch_id: SignalBranchId,
    ) {
        self.ledger
            .rebind_admitted(lease_id, previous_branch_id, branch_id);
    }

    pub(crate) fn activate_reserved_admitted(&self, lease_id: u64, branch_id: SignalBranchId) {
        self.ledger.activate_reserved_admitted(lease_id, branch_id);
    }

    pub(crate) fn cancel_reserved_admitted_for_branch(
        &self,
        reservation_count: usize,
        branch_id: SignalBranchId,
    ) {
        self.ledger
            .cancel_reserved_admitted(reservation_count, branch_id);
    }

    pub(crate) fn rebind_reserved_admitted(
        &self,
        reservation_count: usize,
        previous_branch_id: SignalBranchId,
        branch_id: SignalBranchId,
    ) {
        self.ledger
            .rebind_reserved_admitted(reservation_count, previous_branch_id, branch_id);
    }
}

impl SignalRetentionLedger {
    pub(super) fn reserve_admitted_lease_identities(
        &self,
        branch_id: SignalBranchId,
        lease_count: usize,
    ) -> Result<Vec<u64>, SignalBranchRetentionAcquisitionDenial> {
        let mut state = self.lock();
        if state.admitted_leases.len()
            + state.external_leases.len()
            + state.reserved_admitted_lease_count
            + lease_count
            > self.maximum_active_leases
        {
            return Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases: self.maximum_active_leases,
            });
        }
        let lease_count = u64::try_from(lease_count)
            .map_err(|_| SignalBranchRetentionAcquisitionDenial::IdentityExhausted)?;
        let final_lease_id = state
            .next_lease_id
            .checked_add(lease_count)
            .ok_or(SignalBranchRetentionAcquisitionDenial::IdentityExhausted)?;
        let lease_ids = if lease_count == 0 {
            Vec::new()
        } else {
            let first_lease_id = state.next_lease_id + 1;
            (first_lease_id..=final_lease_id).collect::<Vec<_>>()
        };
        state.reserved_admitted_lease_count += lease_ids.len();
        for _ in 0..lease_ids.len() {
            increment_obligation_count(&mut state.reserved_admitted_count_by_branch, branch_id);
        }
        state.next_lease_id = final_lease_id;
        Ok(lease_ids)
    }

    fn activate_reserved_admitted(&self, lease_id: u64, branch_id: SignalBranchId) {
        let mut state = self.lock();
        state.reserved_admitted_lease_count = state
            .reserved_admitted_lease_count
            .checked_sub(1)
            .expect("an admitted-output slot converts exactly once");
        decrement_obligation_count(&mut state.reserved_admitted_count_by_branch, &branch_id);
        let prior = state.admitted_leases.insert(lease_id, branch_id);
        debug_assert!(prior.is_none());
        increment_obligation_count(&mut state.admitted_count_by_branch, branch_id);
    }

    fn cancel_reserved_admitted(&self, reservation_count: usize, branch_id: SignalBranchId) {
        let mut state = self.lock();
        state.reserved_admitted_lease_count = state
            .reserved_admitted_lease_count
            .checked_sub(reservation_count)
            .expect("unused admitted-output reservations return capacity exactly once");
        for _ in 0..reservation_count {
            decrement_obligation_count(&mut state.reserved_admitted_count_by_branch, &branch_id);
        }
    }

    fn rebind_reserved_admitted(
        &self,
        reservation_count: usize,
        previous_branch_id: SignalBranchId,
        branch_id: SignalBranchId,
    ) {
        if previous_branch_id == branch_id {
            return;
        }
        let mut state = self.lock();
        for _ in 0..reservation_count {
            decrement_obligation_count(
                &mut state.reserved_admitted_count_by_branch,
                &previous_branch_id,
            );
            increment_obligation_count(&mut state.reserved_admitted_count_by_branch, branch_id);
        }
    }

    fn release_admitted(&self, lease_id: u64, branch_id: SignalBranchId) {
        let mut state = self.lock();
        if state.admitted_leases.remove(&lease_id) == Some(branch_id) {
            decrement_obligation_count(&mut state.admitted_count_by_branch, &branch_id);
        }
    }

    fn rebind_admitted(
        &self,
        lease_id: u64,
        previous_branch_id: SignalBranchId,
        branch_id: SignalBranchId,
    ) {
        let mut state = self.lock();
        if state.admitted_leases.get(&lease_id) != Some(&previous_branch_id) {
            return;
        }
        state.admitted_leases.insert(lease_id, branch_id);
        decrement_obligation_count(&mut state.admitted_count_by_branch, &previous_branch_id);
        increment_obligation_count(&mut state.admitted_count_by_branch, branch_id);
    }
}
