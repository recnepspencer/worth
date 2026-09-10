use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::history::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use crate::identity::ProductBranchReferenceGeneration;
use crate::retention::RetentionTransferReceipt;
use crate::retention::{RetentionObligationDenial, RuntimeWorldRetentionOwner};

use super::observation::{
    ProductBranchObservation, ProductBranchObservationAdmissionFailure,
    ProductBranchObservationMismatch,
};
use super::reference_snapshot::ProductBranchReferenceSnapshot;
pub(crate) use protection::{
    ProductBranchHeadProtection, ProductBranchHeadProtectionAdmissionFailure,
};

mod protection;
mod publication;
#[cfg(test)]
pub(crate) mod publication_unwind;
mod retirement;
mod successor_validation;
pub(crate) use retirement::ProductBranchReferenceRetirement;
use successor_validation::validate_successor;

#[derive(Debug)]
struct ProductBranchReferenceImage {
    snapshot: ProductBranchReferenceSnapshot,
    protection: Option<ProductBranchHeadProtection>,
}

#[derive(Debug, Clone)]
struct ReferenceCellState {
    current: Arc<RwLock<ProductBranchReferenceImage>>,
}

impl ReferenceCellState {
    fn new(initial: ProductBranchReferenceImage) -> Self {
        Self {
            current: Arc::new(RwLock::new(initial)),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, ProductBranchReferenceImage> {
        self.current
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, ProductBranchReferenceImage> {
        self.current
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Independently borrowable product-reference cell; clones share only this
/// branch's synchronization domain.
#[derive(Debug, Clone)]
pub(crate) struct ProductBranchReferenceCell {
    state: ReferenceCellState,
}

/// Why a reference movement could not replace the selected product head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProductBranchReferenceCellDenial {
    Retired,
    ExpectedHeadMismatch(ProductBranchObservationMismatch),
    SuccessorOwnerMismatch,
    SuccessorBranchMismatch,
    SuccessorLifecycleMismatch,
    SuccessorGenerationMismatch {
        expected: ProductBranchReferenceGeneration,
        actual: ProductBranchReferenceGeneration,
    },
    SuccessorProtectionMismatch,
    GenerationExhausted,
    Cutoff(crate::publication::ProductMovementCutoffDenial),
}

#[derive(Debug)]
#[cfg(test)]
pub(crate) struct ProductBranchReferencePublishFailure {
    denial: ProductBranchReferenceCellDenial,
    observed_head: ProductBranchReferenceSnapshot,
    successor_protection: ProductBranchHeadProtection,
}

#[derive(Debug)]
pub(crate) struct ProductBranchReferenceLoss {
    denial: ProductBranchReferenceCellDenial,
    observed_head: ProductBranchReferenceSnapshot,
}

impl ProductBranchReferenceLoss {
    pub(crate) fn cutoff_denial(&self) -> Option<crate::publication::ProductMovementCutoffDenial> {
        match self.denial {
            ProductBranchReferenceCellDenial::Cutoff(denial) => Some(denial),
            _ => None,
        }
    }

    pub(crate) fn observed_head(&self) -> &ProductBranchReferenceSnapshot {
        &self.observed_head
    }

    pub(crate) fn is_retired(&self) -> bool {
        matches!(self.denial, ProductBranchReferenceCellDenial::Retired)
    }
}

#[cfg(test)]
impl ProductBranchReferencePublishFailure {
    #[cfg(test)]
    pub(crate) fn denial(&self) -> &ProductBranchReferenceCellDenial {
        &self.denial
    }

    #[cfg(test)]
    pub(crate) fn observed_head(&self) -> &ProductBranchReferenceSnapshot {
        &self.observed_head
    }

    #[cfg(test)]
    pub(crate) fn into_successor_protection(self) -> ProductBranchHeadProtection {
        self.successor_protection
    }
}

pub(crate) enum ProductBranchReferenceObservationFailure {
    Retired,
    HistoryProtection(CompositeHistoryCatalogDenial),
    Retention(RetentionObligationDenial),
    ObservationBinding(ProductBranchObservationAdmissionFailure),
}

/// Exact old/new images installed by one movement; it mints no owner artifact.
#[derive(Debug, Clone)]
pub(crate) struct ProductBranchReferenceMovement {
    before: ProductBranchReferenceSnapshot,
    after: ProductBranchReferenceSnapshot,
    _retention_transfer: RetentionTransferReceipt,
}

impl ProductBranchReferenceMovement {
    pub(crate) fn before(&self) -> &ProductBranchReferenceSnapshot {
        &self.before
    }

    pub(crate) fn after(&self) -> &ProductBranchReferenceSnapshot {
        &self.after
    }
}

impl ProductBranchReferenceCell {
    pub(crate) fn new(
        protection: ProductBranchHeadProtection,
    ) -> Result<Self, ProductBranchHeadProtectionAdmissionFailure> {
        let initial = protection.snapshot().clone();
        match protection.validate() {
            Ok(()) => Ok(Self {
                state: ReferenceCellState::new(ProductBranchReferenceImage {
                    snapshot: initial,
                    protection: Some(protection),
                }),
            }),
            Err(denial) => Err(protection.into_admission_failure(denial)),
        }
    }

    /// Capture an immutable image whose commit stays alive across movement.
    pub(crate) fn atomic_snapshot(&self) -> ProductBranchReferenceSnapshot {
        self.state.read().snapshot.clone()
    }

    /// Run `f` on `argument` while this cell provably still carries
    /// `expected`. The branch-local read guard is held across `f`, so nothing
    /// can publish past the expected head until `f` returns: what `f`
    /// installs from that head is installed from a current head, not from one
    /// that was current when it was last checked. `f` must not touch this
    /// cell. A displaced head returns the head the cell carries, with the
    /// argument untouched, so the caller keeps whatever custody it holds.
    pub(crate) fn while_current<A, R>(
        &self,
        expected: &ProductBranchObservation,
        argument: A,
        f: impl FnOnce(A) -> R,
    ) -> Result<R, (ProductBranchReferenceSnapshot, A)> {
        let current = self.state.read();
        if current.protection.is_none()
            || expected
                .mismatch_against_snapshot(&current.snapshot)
                .is_some()
        {
            return Err((current.snapshot.clone(), argument));
        }
        Ok(f(argument))
    }

    /// Give back the protection of a cell nobody else shares. A cell the
    /// registry never installed has exactly one holder, so for it this is
    /// total; a shared cell keeps its protection and answers `None`.
    pub(crate) fn into_protection(self) -> Option<ProductBranchHeadProtection> {
        Arc::try_unwrap(self.state.current).ok().and_then(|image| {
            image
                .into_inner()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .protection
        })
    }

    /// Protect and recheck one candidate before asking the component owner for
    /// its exact observation claims. The equal recheck is the head
    /// linearization point; all owner calls happen after its read guard drops.
    pub(crate) fn observe<D, I, T>(
        &self,
        history: &CompositeHistoryCatalog,
        retention_owner: &RuntimeWorldRetentionOwner<D, I, T>,
    ) -> Result<ProductBranchObservation, ProductBranchReferenceObservationFailure>
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        loop {
            let candidate = {
                let current = self.state.read();
                if current.protection.is_none() {
                    return Err(ProductBranchReferenceObservationFailure::Retired);
                }
                current.snapshot.clone()
            };
            let history_protection = match history.protect_explicit_commit(candidate.commit()) {
                Ok(protection) => protection,
                Err(denial) => {
                    let current = self.state.read();
                    if current.protection.is_none() {
                        return Err(ProductBranchReferenceObservationFailure::Retired);
                    }
                    if current.snapshot != candidate {
                        drop(current);
                        continue;
                    }
                    return Err(ProductBranchReferenceObservationFailure::HistoryProtection(
                        denial,
                    ));
                }
            };
            let current = self.state.read();
            if current.protection.is_none() {
                return Err(ProductBranchReferenceObservationFailure::Retired);
            }
            let still_selected = current.snapshot == candidate;
            drop(current);
            if !still_selected {
                drop(history_protection);
                continue;
            }

            let components = match retention_owner.issue_observation(candidate.commit()) {
                Ok(components) => components,
                Err(denial) => {
                    drop(history_protection);
                    return Err(ProductBranchReferenceObservationFailure::Retention(denial));
                }
            };
            let current = self.state.read();
            if current.protection.is_none() {
                drop(current);
                drop(components);
                drop(history_protection);
                return Err(ProductBranchReferenceObservationFailure::Retired);
            }
            if current.snapshot != candidate {
                drop(current);
                drop(components);
                drop(history_protection);
                continue;
            }
            let observation =
                ProductBranchObservation::owner_issued(candidate, components, history_protection)
                    .map_err(ProductBranchReferenceObservationFailure::ObservationBinding);
            drop(current);
            return observation;
        }
    }

    /// Replace only if the complete expected observation is still selected.
    /// Expected-currentness, successor validation, and pair replacement use
    /// one short branch-local lock; no owner call occurs while it is held.
    #[cfg(test)]
    pub(crate) fn compare_and_publish(
        &self,
        expected: &ProductBranchObservation,
        successor: ProductBranchHeadProtection,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferencePublishFailure> {
        let mut held = Some(successor);
        self.replace_expected(expected, &mut held, |held| held, None)
            .map_err(|loss| ProductBranchReferencePublishFailure {
                denial: loss.denial,
                observed_head: loss.observed_head,
                successor_protection: held
                    .take()
                    .expect("a losing test CAS retains its successor"),
            })
    }

    #[cfg(test)]
    fn hold_for_test(&self) -> impl Drop + '_ {
        self.state.write()
    }

    /// Whether a writer would block right now: true exactly while some guard
    /// on this cell is held. Non-blocking, so a guarded section may ask it.
    #[cfg(test)]
    fn writers_are_locked_out_for_test(&self) -> bool {
        matches!(
            self.state.current.try_write(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }
}

#[cfg(test)]
#[path = "reference_cell_tests.rs"]
mod tests;

impl std::fmt::Debug for ProductBranchReferenceObservationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retired => f.write_str("Retired"),
            Self::HistoryProtection(value) => {
                f.debug_tuple("HistoryProtection").field(value).finish()
            }
            Self::Retention(value) => f.debug_tuple("Retention").field(value).finish(),
            Self::ObservationBinding(value) => {
                f.debug_tuple("ObservationBinding").field(value).finish()
            }
        }
    }
}
