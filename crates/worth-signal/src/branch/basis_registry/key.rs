use crate::state::SignalBranchId;

use super::super::SignalBranchObservation;

/// Exact live owner basis key. The encoding is a lookup key only; it is never
/// exposed as admission or currentness evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct SignalBranchBasisRegistryKey {
    pub(super) runtime_instance_id: u64,
    pub(super) definition_basis: u64,
    pub(super) branch_id: SignalBranchId,
    pub(super) cell_incarnation: u64,
    observation_encoding: Vec<u8>,
}

impl SignalBranchBasisRegistryKey {
    pub(super) fn new(
        runtime_instance_id: u64,
        definition_basis: u64,
        branch_id: SignalBranchId,
        cell_incarnation: u64,
        observation: &SignalBranchObservation,
    ) -> Self {
        Self {
            runtime_instance_id,
            definition_basis,
            branch_id,
            cell_incarnation,
            observation_encoding: observation.canonical_encoding(),
        }
    }
}
