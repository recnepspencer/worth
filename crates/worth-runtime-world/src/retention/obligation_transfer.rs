use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};

use super::component_obligation::ProductHeadRetentionObligation;
use super::unique_component_pin::ExactComponentBasisKey;
use super::ComponentBasisDependencyClass;

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
    HistoricalInspection,
    Release,
}

#[cfg(test)]
impl ComponentBasisObligationTransferDestination {
    #[cfg(test)]
    pub(crate) const fn dependency_class(self) -> Option<ComponentBasisDependencyClass> {
        match self {
            Self::ProductBranchHead => Some(ComponentBasisDependencyClass::ProductBranchHead),
            Self::RetainedCompositeHistory => {
                Some(ComponentBasisDependencyClass::RetainedCompositeHistory)
            }
            Self::AdmittedObservation => Some(ComponentBasisDependencyClass::AdmittedObservation),
            Self::ActivePublicationAttempt => {
                Some(ComponentBasisDependencyClass::ActivePublicationAttempt)
            }
            Self::ProductUnpublishedOwnerEffects => {
                Some(ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects)
            }
            Self::HistoricalInspection => Some(ComponentBasisDependencyClass::HistoricalInspection),
            Self::Release => None,
        }
    }
}

/// Why an owner-issued obligation could not change semantic custody.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetentionTransferDenial {
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
    source: ComponentBasisDependencyClass,
    destination: ComponentBasisDependencyClass,
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
            source: ComponentBasisDependencyClass::ActivePublicationAttempt,
            destination: ComponentBasisDependencyClass::ProductBranchHead,
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

    pub(crate) const fn source(&self) -> ComponentBasisDependencyClass {
        self.source
    }

    pub(crate) const fn destination(&self) -> ComponentBasisDependencyClass {
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
