use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::history::CompositeRuntimeWorldCommit;
use crate::identity::{CompositeBasisKey, CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

use super::super::obligation_transfer::{
    ProductHeadRetentionTransfer, RetentionTransferDenial, RetentionTransferReceipt,
};
use super::super::unique_component_pin::ExactComponentPinRequest;
use super::super::ComponentBasisDependencyClass;
use super::product_head::ProductHeadRetentionObligation;
use super::ComponentBasisPinObligation;

/// The fixed two-component proof issued by the Runtime World retention owner.
/// Its private metadata binds one owner, one exact composite basis, one
/// dependency class, and both exact component claims before any wrapper can be
/// constructed.
#[derive(Debug)]
pub(crate) struct IssuedComponentPinPair {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    dependency: ComponentBasisDependencyClass,
    relational: ComponentBasisPinObligation,
    signal: ComponentBasisPinObligation,
}

impl IssuedComponentPinPair {
    pub(crate) fn owner_issued(
        basis: &AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
        relational: ComponentBasisPinObligation,
        signal: ComponentBasisPinObligation,
    ) -> Self {
        let owner = basis.owner_identity();
        assert_eq!(relational.owner_identity(), owner);
        assert_eq!(signal.owner_identity(), owner);
        assert_eq!(relational.dependency(), dependency);
        assert_eq!(signal.dependency(), dependency);
        assert_ne!(relational.key(), signal.key());
        assert_eq!(
            relational.key(),
            &ExactComponentPinRequest::relational(basis, dependency).key()
        );
        assert_eq!(
            signal.key(),
            &ExactComponentPinRequest::signal(basis, dependency).key()
        );
        Self {
            owner,
            basis: basis.identity().clone(),
            dependency,
            relational,
            signal,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RuntimeWorldOwnerIdentity,
        CompositeBasisKey,
        ComponentBasisDependencyClass,
        ComponentBasisPinObligation,
        ComponentBasisPinObligation,
    ) {
        (
            self.owner,
            self.basis,
            self.dependency,
            self.relational,
            self.signal,
        )
    }
}

/// Two exact component claims carried by one observation and bound to the
/// exact commit occurrence and basis from which the owner issued them.
#[derive(Debug)]
pub(crate) struct ObservationRetentionObligation {
    _relational: ComponentBasisPinObligation,
    _signal: ComponentBasisPinObligation,
    captured_commit: CompositeCommitIdentity,
    captured_basis: CompositeBasisKey,
    _capacity: crate::retention::ReservedObservationCapacity,
}

impl ObservationRetentionObligation {
    pub(crate) fn owner_issued(
        commit: &CompositeRuntimeWorldCommit,
        pair: IssuedComponentPinPair,
        capacity: crate::retention::ReservedObservationCapacity,
    ) -> Self {
        let (owner, basis, dependency, relational, signal) = pair.into_parts();
        assert_eq!(capacity.owner_identity(), owner);
        assert_eq!(commit.identity().owner_identity(), owner);
        assert_eq!(commit.basis().owner_identity(), owner);
        assert_eq!(basis, *commit.basis().identity());
        assert_eq!(
            dependency,
            ComponentBasisDependencyClass::AdmittedObservation
        );
        Self {
            _relational: relational,
            _signal: signal,
            captured_commit: commit.identity().clone(),
            captured_basis: basis,
            _capacity: capacity,
        }
    }

    #[cfg(test)]
    pub(crate) fn relational(&self) -> &ComponentBasisPinObligation {
        &self._relational
    }

    #[cfg(test)]
    pub(crate) fn signal(&self) -> &ComponentBasisPinObligation {
        &self._signal
    }

    pub(crate) fn matches_captured_head(&self, commit: &CompositeRuntimeWorldCommit) -> bool {
        self.captured_commit.eq(commit.identity())
            && self.captured_basis.eq(commit.basis().identity())
    }
}

/// Two exact component claims reserved for one publication attempt. The
/// prospective successor basis is checked before transfer.
#[derive(Debug)]
pub(crate) struct PublicationRetentionObligation {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    relational: ComponentBasisPinObligation,
    signal: ComponentBasisPinObligation,
}

impl PublicationRetentionObligation {
    /// Retention after a denied publication does not need product-head
    /// authority. Keep both original claims guarded through the direct retag.
    pub(crate) fn try_transfer_retained(
        &mut self,
    ) -> Result<super::retained_partial::RetainedPartialRetentionObligation, RetentionTransferDenial>
    {
        let basis = self.basis.clone();
        self.relational.transfer_pair_to(
            &mut self.signal,
            ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects,
        )?;
        Ok(
            super::retained_partial::RetainedPartialRetentionObligation::transferred(
                self.owner,
                basis,
                self.relational.take_transferred_claim(),
                self.signal.take_transferred_claim(),
            ),
        )
    }

    pub(crate) fn owner_issued(pair: IssuedComponentPinPair) -> Self {
        let (owner, basis, dependency, relational, signal) = pair.into_parts();
        assert_eq!(
            dependency,
            ComponentBasisDependencyClass::ActivePublicationAttempt
        );
        Self {
            owner,
            basis,
            relational,
            signal,
        }
    }

    pub(crate) fn relational(&self) -> &ComponentBasisPinObligation {
        &self.relational
    }

    pub(crate) fn signal(&self) -> &ComponentBasisPinObligation {
        &self.signal
    }

    pub(crate) fn matches_basis(&self, basis: &AdmittedCompositeRuntimeWorldBasis) -> bool {
        self.owner == basis.owner_identity()
            && self.basis == *basis.identity()
            && self.relational.key()
                == &ExactComponentPinRequest::relational(
                    basis,
                    ComponentBasisDependencyClass::ActivePublicationAttempt,
                )
                .key()
            && self.signal.key()
                == &ExactComponentPinRequest::signal(
                    basis,
                    ComponentBasisDependencyClass::ActivePublicationAttempt,
                )
                .key()
            && self.relational.dependency()
                == ComponentBasisDependencyClass::ActivePublicationAttempt
            && self.signal.dependency() == ComponentBasisDependencyClass::ActivePublicationAttempt
    }

    /// Transfer publication custody into the sole product-head authority.
    /// The exact successor basis is checked before either claim changes class.
    #[cfg(test)]
    pub(crate) fn into_product_head_transfer(
        mut self,
        successor_basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<ProductHeadRetentionTransfer, (Self, RetentionTransferDenial)> {
        match self.try_transfer_product_head(successor_basis) {
            Ok(transfer) => Ok(transfer),
            Err(denial) => Err((self, denial)),
        }
    }

    /// Keep the original publication obligation in its owner's resource lease
    /// while the pair transfer validates. Denial leaves both claims unchanged.
    pub(crate) fn try_transfer_product_head(
        &mut self,
        successor_basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<ProductHeadRetentionTransfer, RetentionTransferDenial> {
        if !self.matches_basis(successor_basis) {
            return Err(RetentionTransferDenial::BasisMismatch);
        }
        let receipt = RetentionTransferReceipt::product_head(
            self.owner,
            self.basis.clone(),
            self.relational.key().clone(),
            self.signal.key().clone(),
        );
        let successor_identity = self.basis.clone();
        self.relational.transfer_pair_to(
            &mut self.signal,
            ComponentBasisDependencyClass::ProductBranchHead,
        )?;
        let obligation = ProductHeadRetentionObligation::transferred(
            self.owner,
            successor_identity,
            self.relational.take_transferred_claim(),
            self.signal.take_transferred_claim(),
        );
        Ok(ProductHeadRetentionTransfer::new(obligation, receipt))
    }
}
