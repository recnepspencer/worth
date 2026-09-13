#[cfg(test)]
use super::unique_component_pin::ComponentBasisLeaseIdentity;
use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use worth_relational::facade::branch::RelationalBranchBasisDenial;
#[cfg(test)]
use worth_signal::facade::branch::SignalBranchRetentionReleaseDenial;

use crate::identity::RuntimeWorldOwnerIdentity;

#[cfg(test)]
use super::obligation_transfer::ComponentBasisObligationTransferDestination;
use super::obligation_transfer::RetentionTransferDenial;
use super::unique_component_pin::{ComponentBasisPinClaim, ExactComponentBasisKey};
use super::ComponentBasisDependencyClass;

mod composite;
mod history;
pub(crate) use history::HistoryRetentionObligation;
mod product_head;
mod retained_partial;

pub(crate) use composite::{
    IssuedComponentPinPair, ObservationRetentionObligation, PublicationRetentionObligation,
};
pub(crate) use product_head::ProductHeadRetentionObligation;
pub(crate) use retained_partial::RetainedPartialRetentionObligation;

/// Typed refusal from the Runtime World retention owner when a live claim
/// cannot be terminated. The component owner lease remains bound to the
/// registry in every refusal path.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum RetentionReleaseDenial {
    UnknownPin,
    ForeignOwner {
        expected: RuntimeWorldOwnerIdentity,
        actual: RuntimeWorldOwnerIdentity,
    },
    Relational(RelationalBranchBasisDenial),
    Signal(SignalBranchRetentionReleaseDenial),
}

/// Why an explicit claim release reached its terminal outcome or refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum ComponentBasisReleaseOutcome {
    OwnerReleased,
    OwnerOperationPanicked,
    SharedOwnerLease,
}

/// Exact evidence for one dependency-count release. It carries no capability
/// and is not used to authorize another release.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct ComponentBasisReleaseReceipt {
    key: ExactComponentBasisKey,
    lease_identity: ComponentBasisLeaseIdentity,
    outcome: ComponentBasisReleaseOutcome,
}

#[cfg(test)]
impl ComponentBasisReleaseReceipt {
    #[cfg(test)]
    pub(crate) fn owner_issued(
        key: ExactComponentBasisKey,
        lease_identity: ComponentBasisLeaseIdentity,
        outcome: ComponentBasisReleaseOutcome,
    ) -> Self {
        Self {
            key,
            lease_identity,
            outcome,
        }
    }

    #[cfg(test)]
    pub(crate) const fn outcome(&self) -> ComponentBasisReleaseOutcome {
        self.outcome
    }
}

/// Error carrier that returns the still-live move-only claim to an explicit
/// caller. A denied foreign release therefore cannot destroy retention.
#[derive(Debug)]
#[cfg(test)]
pub(crate) struct RetentionReleaseFailure {
    pub(super) claim: ComponentBasisPinClaim,
    pub(super) denial: RetentionReleaseDenial,
}

/// Internal authority surface implemented only by the concrete retention
/// owner. Claims carry this narrow object so the shared Phase 1 handoffs need
/// not become generic over component runtime types.
pub(super) trait RetentionControlSurface: Send + Sync {
    fn fork_history_pair(
        &self,
        relational: &ComponentBasisPinClaim,
        signal: &ComponentBasisPinClaim,
    ) -> Result<(ComponentBasisPinClaim, ComponentBasisPinClaim), RetentionTransferDenial>;

    #[cfg(test)]
    fn transfer_claim(
        &self,
        claim: ComponentBasisPinClaim,
        target: ComponentBasisDependencyClass,
    ) -> Result<ComponentBasisPinClaim, (ComponentBasisPinClaim, RetentionTransferDenial)>;

    fn transfer_pair(
        &self,
        relational: &mut ComponentBasisPinClaim,
        signal: &mut ComponentBasisPinClaim,
        target: ComponentBasisDependencyClass,
    ) -> Result<(), RetentionTransferDenial>;

    #[cfg(test)]
    fn release_claim(
        &self,
        claim: ComponentBasisPinClaim,
    ) -> Result<ComponentBasisReleaseReceipt, RetentionReleaseFailure>;

    fn abandon_claim(&self, claim: ComponentBasisPinClaim);
}

/// One non-cloneable, owner-issued exact component dependency claim.
pub(crate) struct ComponentBasisPinObligation {
    claim: Option<ComponentBasisPinClaim>,
}

impl fmt::Debug for ComponentBasisPinObligation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComponentBasisPinObligation")
            .field("claim", &self.claim)
            .finish()
    }
}

impl ComponentBasisPinObligation {
    pub(super) fn new(claim: ComponentBasisPinClaim) -> Self {
        Self { claim: Some(claim) }
    }

    pub(crate) fn key(&self) -> &ExactComponentBasisKey {
        &self
            .claim
            .as_ref()
            .expect("a live component obligation carries its claim")
            .key
    }

    pub(crate) const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        match &self.claim {
            Some(claim) => claim.owner,
            None => panic!("a live component obligation carries its claim"),
        }
    }

    pub(crate) const fn dependency(&self) -> ComponentBasisDependencyClass {
        match &self.claim {
            Some(claim) => claim.dependency,
            None => panic!("a live component obligation carries its claim"),
        }
    }

    #[cfg(test)]
    pub(crate) const fn lease_identity(&self) -> ComponentBasisLeaseIdentity {
        match &self.claim {
            Some(claim) => claim.lease_identity,
            None => panic!("a live component obligation carries its claim"),
        }
    }

    /// Retag the actual held claims under the retention owner's pair lock.
    /// No raw claim leaves either Drop guard while validation can deny/unwind.
    pub(super) fn fork_history_pair(
        &self,
        other: &Self,
    ) -> Result<(Self, Self), RetentionTransferDenial> {
        let relational = self.claim.as_ref().expect("live first claim");
        let signal = other.claim.as_ref().expect("live second claim");
        let (relational, signal) = relational.control.fork_history_pair(relational, signal)?;
        Ok((Self::new(relational), Self::new(signal)))
    }

    pub(super) fn transfer_pair_to(
        &mut self,
        other: &mut Self,
        target: ComponentBasisDependencyClass,
    ) -> Result<(), RetentionTransferDenial> {
        let relational = self
            .claim
            .as_mut()
            .expect("a live pair retains its first claim");
        let signal = other
            .claim
            .as_mut()
            .expect("a live pair retains its second claim");
        let control = Arc::clone(&relational.control);
        control.transfer_pair(relational, signal, target)
    }

    pub(super) fn take_transferred_claim(&mut self) -> ComponentBasisPinClaim {
        self.claim
            .take()
            .expect("a transferred obligation yields its claim once")
    }

    pub(super) fn into_claim(mut self) -> ComponentBasisPinClaim {
        self.claim
            .take()
            .expect("a live component obligation carries its claim")
    }

    /// Transfer this one exact dependency without acquiring another owner
    /// lease. Failure returns the original obligation unchanged.
    #[cfg(test)]
    pub(crate) fn try_transfer_to(
        mut self,
        destination: ComponentBasisObligationTransferDestination,
    ) -> Result<Self, (Self, RetentionTransferDenial)> {
        let Some(target) = destination.dependency_class() else {
            return Err((self, RetentionTransferDenial::ReleaseDestination));
        };
        let claim = self
            .claim
            .take()
            .expect("a live component obligation carries its claim");
        let control = Arc::clone(&claim.control);
        match control.transfer_claim(claim, target) {
            Ok(claim) => Ok(Self::new(claim)),
            Err((claim, denial)) => {
                self.claim = Some(claim);
                Err((self, denial))
            }
        }
    }

    /// Explicitly consume one claim. A denied release returns the original
    /// obligation so its caller can rebind it to the correct owner and retry.
    #[cfg(test)]
    pub(crate) fn try_release(
        mut self,
    ) -> Result<ComponentBasisReleaseReceipt, (Self, RetentionReleaseDenial)> {
        let claim = self
            .claim
            .take()
            .expect("a live component obligation reaches release only once");
        let control = Arc::clone(&claim.control);
        match control.release_claim(claim) {
            Ok(receipt) => Ok(receipt),
            Err(failure) => {
                self.claim = Some(failure.claim);
                Err((self, failure.denial))
            }
        }
    }
}

impl Drop for ComponentBasisPinObligation {
    fn drop(&mut self) {
        let Some(claim) = self.claim.take() else {
            return;
        };
        let control = Arc::clone(&claim.control);
        control.abandon_claim(claim);
    }
}
