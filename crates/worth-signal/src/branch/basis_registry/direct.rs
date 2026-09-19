use std::sync::Arc;

use crate::state::SignalBranchId;

use super::super::{SignalBranchAdmissionLease, SignalBranchObservation};
use super::{live_ready_basis, new_basis, RegistryEntry, SignalBranchBasisRegistry};

impl SignalBranchBasisRegistry {
    /// Admit a basis when the caller already holds the component owner's
    /// lease. A live canonical basis consumes the extra lease; a stale entry
    /// is replaced only after the owner supplied a new lease.
    pub(crate) fn admit(
        &self,
        runtime_instance_id: u64,
        definition_basis: u64,
        branch_id: SignalBranchId,
        cell_incarnation: u64,
        observation: SignalBranchObservation,
        retention: SignalBranchAdmissionLease,
    ) -> super::super::AdmittedSignalBranchBasis {
        let key = super::key::SignalBranchBasisRegistryKey::new(
            runtime_instance_id,
            definition_basis,
            branch_id,
            cell_incarnation,
            &observation,
        );
        loop {
            let completion = {
                let mut state = self.lock_state();
                match live_ready_basis(&mut state, &key) {
                    Some(existing) => {
                        // Release duplicate custody outside the registry lock:
                        // lease cleanup contacts the independent retention owner.
                        drop(state);
                        drop(retention);
                        return super::super::AdmittedSignalBranchBasis::from_inner(existing);
                    }
                    None => match state.entries.get(&key) {
                        Some(RegistryEntry::Acquiring(acquiring)) => {
                            assert_ne!(
                                acquiring.initiating_thread,
                                std::thread::current().id(),
                                "pre-reserved Signal admission cannot reenter the same key"
                            );
                            acquiring.completion.record_joiner();
                            Some(Arc::clone(&acquiring.completion))
                        }
                        None => {
                            return new_basis(
                                &mut state,
                                &self.state,
                                key.clone(),
                                observation,
                                branch_id,
                                retention,
                            )
                            .expect("owner-issued Signal admission identity cannot be exhausted");
                        }
                        Some(RegistryEntry::Ready { .. }) => unreachable!(
                            "stale Signal basis registry entries are removed before admission"
                        ),
                    },
                }
            };
            match completion
                .expect("a direct Signal admission either joins or installs")
                .wait()
            {
                Ok(existing) => {
                    drop(retention);
                    return existing;
                }
                Err(_) => continue,
            }
        }
    }

    /// Transfer pre-seal canonical entries onto the first sealed owner cell.
    pub(crate) fn rebind_cell_incarnation(
        &self,
        runtime_instance_id: u64,
        definition_basis: u64,
        branch_id: SignalBranchId,
        previous_cell_incarnation: u64,
        cell_incarnation: u64,
    ) {
        if previous_cell_incarnation == cell_incarnation {
            return;
        }
        let mut state = self.lock_state();
        let keys = state
            .entries
            .keys()
            .filter(|key| {
                key.runtime_instance_id == runtime_instance_id
                    && key.definition_basis == definition_basis
                    && key.branch_id == branch_id
                    && key.cell_incarnation == previous_cell_incarnation
            })
            .cloned()
            .collect::<Vec<_>>();
        for mut key in keys {
            let entry = state
                .entries
                .remove(&key)
                .expect("a selected Signal basis registry key remains installed");
            key.cell_incarnation = cell_incarnation;
            let replaced = state.entries.insert(key, entry);
            assert!(
                replaced.is_none(),
                "sealing cannot collide with a pre-existing owner cell incarnation"
            );
        }
    }
}
