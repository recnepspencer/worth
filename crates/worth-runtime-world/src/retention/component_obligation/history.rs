use super::super::RetentionTransferDenial;
use super::{
    ComponentBasisPinObligation, ProductHeadRetentionObligation, PublicationRetentionObligation,
};
use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};
use crate::inspection::RuntimeWorldRetentionKey;

/// Exact history-owned dependencies. One pair lives in each installed history
/// occurrence; it shares the already acquired component-owner leases.
#[derive(Debug)]
pub(crate) struct HistoryRetentionObligation {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    relational: ComponentBasisPinObligation,
    signal: ComponentBasisPinObligation,
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
            relational,
            signal,
        })
    }
    pub(crate) fn matches_basis(&self, basis: &AdmittedCompositeRuntimeWorldBasis) -> bool {
        self.owner == basis.owner_identity() && self.basis == *basis.identity()
    }

    /// Registry keys whose released entries become reclaimable once this
    /// obligation and every other holder of the same exact pin are gone.
    pub(crate) fn retention_keys(&self) -> [RuntimeWorldRetentionKey; 2] {
        [&self.relational, &self.signal].map(|pin| RuntimeWorldRetentionKey {
            owner: self.owner,
            key: pin.key().clone(),
        })
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
