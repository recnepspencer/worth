use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};

use super::super::unique_component_pin::{ComponentBasisPinClaim, ExactComponentPinRequest};
use super::super::ComponentBasisDependencyClass;
use super::retained_partial::RetainedPartialRetentionObligation;
use super::{ComponentBasisPinObligation, IssuedComponentPinPair};

/// The sole Runtime World product-head component authority. Both exact
/// successor component claims remain move-only and carry the product-head
/// dependency class.
#[must_use = "the product head must retain both exact component pins"]
pub(crate) struct ProductHeadRetentionObligation {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    relational: ComponentBasisPinObligation,
    signal: ComponentBasisPinObligation,
}

impl std::fmt::Debug for ProductHeadRetentionObligation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductHeadRetentionObligation")
            .field("owner", &self.owner)
            .field("basis", &self.basis)
            .field("relational", &self.relational.key())
            .field("signal", &self.signal.key())
            .finish()
    }
}

impl ProductHeadRetentionObligation {
    pub(crate) fn owner_issued(pair: IssuedComponentPinPair) -> Self {
        let (owner, basis, dependency, relational, signal) = pair.into_parts();
        assert_eq!(dependency, ComponentBasisDependencyClass::ProductBranchHead);
        Self::transferred(owner, basis, relational.into_claim(), signal.into_claim())
    }

    pub(super) fn transferred(
        owner: RuntimeWorldOwnerIdentity,
        basis: CompositeBasisKey,
        relational: ComponentBasisPinClaim,
        signal: ComponentBasisPinClaim,
    ) -> Self {
        assert_eq!(relational.owner_identity(), owner);
        assert_eq!(signal.owner_identity(), owner);
        assert_eq!(
            relational.dependency(),
            ComponentBasisDependencyClass::ProductBranchHead
        );
        assert_eq!(
            signal.dependency(),
            ComponentBasisDependencyClass::ProductBranchHead
        );
        Self {
            owner,
            basis,
            relational: ComponentBasisPinObligation::new(relational),
            signal: ComponentBasisPinObligation::new(signal),
        }
    }

    pub(crate) const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub(crate) fn basis(&self) -> &CompositeBasisKey {
        &self.basis
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
                    ComponentBasisDependencyClass::ProductBranchHead,
                )
                .key()
            && self.signal.key()
                == &ExactComponentPinRequest::signal(
                    basis,
                    ComponentBasisDependencyClass::ProductBranchHead,
                )
                .key()
            && self.relational.dependency() == ComponentBasisDependencyClass::ProductBranchHead
            && self.signal.dependency() == ComponentBasisDependencyClass::ProductBranchHead
    }

    /// The only product-head-to-recovery transition. Both live claims already
    /// passed owner, basis, and dependency admission, so a denial here would be
    /// registry corruption rather than a recoverable publication outcome.
    #[cfg(test)]
    pub(crate) fn transition_to_retained_partial(mut self) -> RetainedPartialRetentionObligation {
        self.try_transfer_retained()
            .expect("live product-head claims transition atomically into recovery custody")
    }

    pub(crate) fn try_transfer_retained(
        &mut self,
    ) -> Result<RetainedPartialRetentionObligation, crate::retention::RetentionTransferDenial> {
        let basis = self.basis.clone();
        self.relational.transfer_pair_to(
            &mut self.signal,
            ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects,
        )?;
        Ok(RetainedPartialRetentionObligation::transferred(
            self.owner,
            basis,
            self.relational.take_transferred_claim(),
            self.signal.take_transferred_claim(),
        ))
    }
}
