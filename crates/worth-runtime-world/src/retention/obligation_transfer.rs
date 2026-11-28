use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};

use super::component_obligation::ProductHeadRetentionObligation;
use super::unique_component_pin::ExactComponentBasisKey;
use super::ComponentClaimKind;

/// Runtime World destinations used when one exact pin dependency changes
/// meaning. `Release` is explicit so a transfer cannot be mistaken for a
/// fresh acquisition or an owner-lease operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum ComponentBasisObligationTransferDestination {
    ProductBranchHead,
    RetainedCompositeHistory,
    AdmittedObservation,
    ActivePublicationAttempt,
    ProductUnpublishedOwnerEffects,
    Release,
}

#[cfg(test)]
impl ComponentBasisObligationTransferDestination {
    #[cfg(test)]
    pub(crate) const fn dependency_class(self) -> Option<ComponentClaimKind> {
        match self {
            Self::ProductBranchHead => Some(ComponentClaimKind::ProductBranchHead),
            Self::RetainedCompositeHistory => Some(ComponentClaimKind::RetainedCompositeHistory),
            Self::AdmittedObservation => Some(ComponentClaimKind::AdmittedObservation),
            Self::ActivePublicationAttempt => Some(ComponentClaimKind::ActivePublicationAttempt),
            Self::ProductUnpublishedOwnerEffects => {
                Some(ComponentClaimKind::ProductUnpublishedOwnerEffects)
            }
            Self::Release => None,
        }
    }
}

/// Why an owner-issued obligation could not change semantic custody.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionTransferDenial {
    BasisMismatch,
    #[cfg(test)]
    ReleaseDestination,
    UnknownPin,
    ForeignOwner,
    DependencyCountExhausted,
}

/// Evidence for the only publication-to-product-head transfer. This value
/// contains no component claim and therefore cannot release or move anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RetentionTransferReceipt {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    relational: ExactComponentBasisKey,
    signal: ExactComponentBasisKey,
    source: ComponentClaimKind,
    destination: ComponentClaimKind,
}

impl RetentionTransferReceipt {
    pub(super) fn product_head(
        owner: RuntimeWorldOwnerIdentity,
        basis: CompositeBasisKey,
        relational: ExactComponentBasisKey,
        signal: ExactComponentBasisKey,
    ) -> Self {
        Self {
            owner,
            basis,
            relational,
            signal,
            source: ComponentClaimKind::ActivePublicationAttempt,
            destination: ComponentClaimKind::ProductBranchHead,
        }
    }

    pub(crate) const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub(crate) fn basis(&self) -> &CompositeBasisKey {
        &self.basis
    }

    pub(crate) fn relational_key(&self) -> &ExactComponentBasisKey {
        &self.relational
    }

    pub(crate) fn signal_key(&self) -> &ExactComponentBasisKey {
        &self.signal
    }

    pub(crate) const fn source(&self) -> ComponentClaimKind {
        self.source
    }

    pub(crate) const fn destination(&self) -> ComponentClaimKind {
        self.destination
    }
}

/// The product-head transfer carries the new release authority and its
/// separate evidence together. The receipt is deliberately not authoritative.
#[derive(Debug)]
pub(crate) struct ProductHeadRetentionTransfer {
    obligation: ProductHeadRetentionObligation,
    receipt: RetentionTransferReceipt,
}

impl ProductHeadRetentionTransfer {
    pub(crate) fn new(
        obligation: ProductHeadRetentionObligation,
        receipt: RetentionTransferReceipt,
    ) -> Self {
        Self {
            obligation,
            receipt,
        }
    }

    pub(crate) fn into_parts(self) -> (ProductHeadRetentionObligation, RetentionTransferReceipt) {
        (self.obligation, self.receipt)
    }
}
