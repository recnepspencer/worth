#[cfg(test)]
use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{CompositeBasisKey, RuntimeWorldOwnerIdentity};

use super::super::unique_component_pin::ComponentBasisPinClaim;
#[cfg(test)]
use super::super::unique_component_pin::ExactComponentPinRequest;
use super::super::ComponentBasisDependencyClass;
use super::ComponentBasisPinObligation;
#[cfg(test)]
use super::IssuedComponentPinPair;

/// Two exact component claims retained with product-unpublished owner effects.
pub(crate) struct RetainedPartialRetentionObligation {
    owner: RuntimeWorldOwnerIdentity,
    basis: CompositeBasisKey,
    relational: ComponentBasisPinObligation,
    signal: ComponentBasisPinObligation,
}

impl RetainedPartialRetentionObligation {
    #[cfg(test)]
    pub(crate) fn owner_issued(pair: IssuedComponentPinPair) -> Self {
        let (owner, basis, dependency, relational, signal) = pair.into_parts();
        assert_eq!(
            dependency,
            ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects
        );
        Self {
            owner,
            basis,
            relational,
            signal,
        }
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
            ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects
        );
        assert_eq!(
            signal.dependency(),
            ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects
        );
        Self {
            owner,
            basis,
            relational: ComponentBasisPinObligation::new(relational),
            signal: ComponentBasisPinObligation::new(signal),
        }
    }

    #[cfg(test)]
    pub(crate) fn signal(&self) -> &ComponentBasisPinObligation {
        &self.signal
    }

    #[cfg(test)]
    pub(crate) fn matches_basis(&self, basis: &AdmittedCompositeRuntimeWorldBasis) -> bool {
        self.owner == basis.owner_identity()
            && self.basis == *basis.identity()
            && self.relational.key()
                == &ExactComponentPinRequest::relational(
                    basis,
                    ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects,
                )
                .key()
            && self.signal.key()
                == &ExactComponentPinRequest::signal(
                    basis,
                    ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects,
                )
                .key()
            && self.relational.dependency()
                == ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects
            && self.signal.dependency()
                == ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects
    }
}

impl std::fmt::Debug for RetainedPartialRetentionObligation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedPartialRetentionObligation")
            .field("owner", &self.owner)
            .field("basis", &self.basis)
            .field("relational", &self.relational)
            .field("signal", &self.signal)
            .finish()
    }
}
