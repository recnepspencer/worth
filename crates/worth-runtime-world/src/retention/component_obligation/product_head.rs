use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};

use super::super::unique_component_pin::{ComponentBasisPinClaim, ExactComponentPinRequest};
use super::super::ComponentClaimKind;
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
            .field("relational", &self.relational)
            .field("signal", &self.signal)
            .finish()
    }
}

impl ProductHeadRetentionObligation {
    pub(crate) fn try_retire_shared_for_close(&mut self) -> Result<(), ()> {
        let (Some(relational), Some(signal)) = (&self.relational.claim, &self.signal.claim) else {
            assert!(self.relational.claim.is_none() && self.signal.claim.is_none());
            return Ok(());
        };
        relational.control.try_retire_shared_pair(
            ComponentClaimKind::ProductBranchHead,
            relational,
            signal,
        )?;
        // Claims contain only metadata and a control handle; the actual owner
        // leases remain in the same registry and the populated history slot.
        self.relational.claim = None;
        self.signal.claim = None;
        Ok(())
    }

    pub(crate) fn owner_issued(pair: IssuedComponentPinPair) -> Self {
        let (owner, basis, dependency, relational, signal) = pair.into_parts();
        assert_eq!(dependency, ComponentClaimKind::ProductBranchHead);
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
            ComponentClaimKind::ProductBranchHead
        );
        assert_eq!(signal.dependency(), ComponentClaimKind::ProductBranchHead);
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
                    ComponentClaimKind::ProductBranchHead,
                )
                .key()
            && self.signal.key()
                == &ExactComponentPinRequest::signal(basis, ComponentClaimKind::ProductBranchHead)
                    .key()
            && self.relational.dependency() == ComponentClaimKind::ProductBranchHead
            && self.signal.dependency() == ComponentClaimKind::ProductBranchHead
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
            ComponentClaimKind::ProductUnpublishedOwnerEffects,
        )?;
        Ok(RetainedPartialRetentionObligation::transferred(
            self.owner,
            basis,
            self.relational.take_transferred_claim(),
            self.signal.take_transferred_claim(),
        ))
    }
}
