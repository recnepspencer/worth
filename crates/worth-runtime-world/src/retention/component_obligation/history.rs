use super::super::RetentionTransferDenial;
use super::{
    ComponentBasisPinObligation, ObservationRetentionObligation, ProductHeadRetentionObligation,
    PublicationRetentionObligation,
};
use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};
use crate::inspection::RuntimeWorldRetentionKey;

/// Exact history-owned dependencies. One pair lives in each installed history
/// occurrence or durable origin recipe; it shares the already acquired leases.
#[derive(Debug)]
pub(crate) struct HistoryRetentionObligation {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    relational: ComponentBasisPinObligation,
    signal: ComponentBasisPinObligation,
}
impl HistoryRetentionObligation {
    pub(crate) fn try_begin_close_retirement(
        &mut self,
    ) -> Result<super::RetiringHistoryComponents, ()> {
        let relational = self.relational.claim.as_ref().ok_or(())?;
        let signal = self.signal.claim.as_ref().ok_or(())?;
        let retired = relational
            .control
            .try_take_history_pair_for_close(relational, signal)?;
        self.relational.claim = None;
        self.signal.claim = None;
        Ok(retired)
    }

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

impl ObservationRetentionObligation {
    pub(crate) fn fork_history(
        &self,
        commit: &crate::history::CompositeRuntimeWorldCommit,
    ) -> Result<HistoryRetentionObligation, RetentionTransferDenial> {
        if !self.matches_captured_head(commit) {
            return Err(RetentionTransferDenial::BasisMismatch);
        }
        HistoryRetentionObligation::fork(commit.basis(), self.relational(), self.signal())
    }
}
