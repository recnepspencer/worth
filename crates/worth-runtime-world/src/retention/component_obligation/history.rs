use super::super::RetentionTransferDenial;
use super::{
    ComponentBasisPinObligation, ProductHeadRetentionObligation, PublicationRetentionObligation,
};
use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};

/// Exact history-owned dependencies. One pair lives in each installed history
/// occurrence; it shares the already acquired component-owner leases.
#[derive(Debug)]
pub(crate) struct HistoryRetentionObligation {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    _relational: ComponentBasisPinObligation,
    _signal: ComponentBasisPinObligation,
}
impl HistoryRetentionObligation {
    fn fork(
        basis: &AdmittedCompositeRuntimeWorldBasis,
        relational: &ComponentBasisPinObligation,
        signal: &ComponentBasisPinObligation,
    ) -> Result<Self, RetentionTransferDenial> {
        let (relational, signal) = relational.fork_history_pair(signal)?;
        Ok(Self {
            owner: basis.owner_identity(),
            basis: basis.identity().clone(),
            _relational: relational,
            _signal: signal,
        })
    }
    pub(crate) fn matches_basis(&self, basis: &AdmittedCompositeRuntimeWorldBasis) -> bool {
        self.owner == basis.owner_identity() && self.basis == *basis.identity()
    }
}
impl PublicationRetentionObligation {
    pub(crate) fn fork_history(
        &self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<HistoryRetentionObligation, RetentionTransferDenial> {
        if !self.matches_basis(basis) {
            return Err(RetentionTransferDenial::BasisMismatch);
        }
        HistoryRetentionObligation::fork(basis, self.relational(), self.signal())
    }
}
impl ProductHeadRetentionObligation {
    pub(crate) fn fork_history(
        &self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<HistoryRetentionObligation, RetentionTransferDenial> {
        if !self.matches_basis(basis) {
            return Err(RetentionTransferDenial::BasisMismatch);
        }
        HistoryRetentionObligation::fork(basis, self.relational(), self.signal())
    }
}
