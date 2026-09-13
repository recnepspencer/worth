use std::sync::Arc;

use crate::branch::{AdmittedSignalBranchBasis, SignalBranchObservation};
use crate::state::SignalBranchId;

use super::{RegistryEntry, SignalBranchBasisRegistry};

impl SignalBranchBasisRegistry {
    /// Prove that a basis is the canonical admission for the currently
    /// observed cell incarnation. Descriptors never participate in this
    /// authority check; the registry's live entry and its owner-issued Arc do.
    pub(crate) fn is_current_canonical_basis(
        &self,
        runtime_instance_id: u64,
        definition_basis: u64,
        branch_id: SignalBranchId,
        cell_incarnation: u64,
        observation: &SignalBranchObservation,
        basis: &AdmittedSignalBranchBasis,
    ) -> bool {
        let key = super::key::SignalBranchBasisRegistryKey::new(
            runtime_instance_id,
            definition_basis,
            branch_id,
            cell_incarnation,
            observation,
        );
        let state = self.lock_state();
        let Some(RegistryEntry::Ready {
            basis: canonical, ..
        }) = state.entries.get(&key)
        else {
            return false;
        };
        canonical
            .upgrade()
            .is_some_and(|canonical| Arc::ptr_eq(&canonical, &basis.0))
    }
}
