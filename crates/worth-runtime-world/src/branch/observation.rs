use std::sync::Arc;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::history::ExplicitCommitHistoryProtectionObligation;
use crate::identity::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchReferenceGeneration, RuntimeWorldOwnerIdentity,
};
use crate::retention::ObservationRetentionObligation;

use super::reference_snapshot::ProductBranchReferenceSnapshot;

/// Complete product-head observation used by every compare-and-publish
/// operation. A single reference snapshot supplies the commit and basis;
/// its obligation owns both exact component claims and explicit history
/// protection for that same commit occurrence.
#[derive(Debug)]
pub struct ProductBranchObservation {
    snapshot: Arc<ProductBranchReferenceSnapshot>,
    obligation: Arc<ProductBranchObservationObligation>,
}

impl Clone for ProductBranchObservation {
    fn clone(&self) -> Self {
        Self {
            snapshot: Arc::clone(&self.snapshot),
            obligation: Arc::clone(&self.obligation),
        }
    }
}

impl PartialEq for ProductBranchObservation {
    fn eq(&self, other: &Self) -> bool {
        self.owner_identity() == other.owner_identity()
            && self.branch_identity() == other.branch_identity()
            && self.lifecycle_incarnation() == other.lifecycle_incarnation()
            && self.reference_generation() == other.reference_generation()
            && self.selected_commit() == other.selected_commit()
            && self.basis() == other.basis()
    }
}

impl Eq for ProductBranchObservation {}

/// Cloned observations share one coherent pair of exact component and History
/// obligations. Cloning this wrapper acquires neither authority again.
#[derive(Debug)]
pub(crate) struct ProductBranchObservationObligation {
    _components: ObservationRetentionObligation,
    _history: ExplicitCommitHistoryProtectionObligation,
}

impl ProductBranchObservationObligation {
    fn owner_issued(
        components: ObservationRetentionObligation,
        history: ExplicitCommitHistoryProtectionObligation,
    ) -> Self {
        Self {
            _components: components,
            _history: history,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductBranchObservationAuthorityDenial {
    ComponentCommitOrBasisMismatch,
    HistoryCommitOrOwnerMismatch,
}

pub(crate) struct ProductBranchObservationAdmissionFailure {
    denial: ProductBranchObservationAuthorityDenial,
    snapshot: ProductBranchReferenceSnapshot,
    components: ObservationRetentionObligation,
    history: ExplicitCommitHistoryProtectionObligation,
}

impl ProductBranchObservationAdmissionFailure {
    #[cfg(test)]
    pub(crate) const fn denial(&self) -> ProductBranchObservationAuthorityDenial {
        self.denial
    }

    #[cfg(test)]
    pub(crate) fn into_parts(
        self,
    ) -> (
        ProductBranchReferenceSnapshot,
        ObservationRetentionObligation,
        ExplicitCommitHistoryProtectionObligation,
    ) {
        (self.snapshot, self.components, self.history)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductBranchObservationMismatchAxis {
    OwnerIdentity,
    BranchIdentity,
    LifecycleIncarnation,
    ReferenceGeneration,
    SelectedCompositeCommit,
    CompositeBasis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductBranchObservationMismatch {
    axes: Vec<ProductBranchObservationMismatchAxis>,
}

impl ProductBranchObservationMismatch {
    pub fn axes(&self) -> &[ProductBranchObservationMismatchAxis] {
        &self.axes
    }

    fn between(
        expected: &ProductBranchReferenceSnapshot,
        observed: &ProductBranchReferenceSnapshot,
    ) -> Option<Self> {
        let mut axes = Vec::new();
        if expected.owner() != observed.owner() {
            axes.push(ProductBranchObservationMismatchAxis::OwnerIdentity);
        }
        if expected.branch() != observed.branch() {
            axes.push(ProductBranchObservationMismatchAxis::BranchIdentity);
        }
        if expected.lifecycle() != observed.lifecycle() {
            axes.push(ProductBranchObservationMismatchAxis::LifecycleIncarnation);
        }
        if expected.generation() != observed.generation() {
            axes.push(ProductBranchObservationMismatchAxis::ReferenceGeneration);
        }
        if expected.commit().identity() != observed.commit().identity() {
            axes.push(ProductBranchObservationMismatchAxis::SelectedCompositeCommit);
        }
        if crate::basis::compare_exact(expected.commit().basis(), observed.commit().basis())
            .is_err()
        {
            axes.push(ProductBranchObservationMismatchAxis::CompositeBasis);
        }
        (!axes.is_empty()).then_some(Self { axes })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldBranchAdmissionDenial {
    CancelledBeforeEffect,
    DeadlineBeforeEffect,
    OwnerUnavailable,
    ForeignOwner,
    RetiredBranch,
    DuplicateName,
    CapacityExhausted,
    IdentityExhausted,
    /// A creation intent reached branch creation without the two explicit
    /// component postures a creation cell requires.
    PlansOmitted,
    /// The installed owner-created component custody budget has no free slot
    /// for the fork this creation would perform.
    CustodyCapacityExhausted,
    /// The admitted source observation no longer matches the branch's current
    /// head, so the operation it was admitted for is answering about a product
    /// occurrence that has already moved on. Nothing was reserved or moved. A
    /// branch with no head at all is `RetiredBranch`, not this.
    StaleSourceHead,
    /// The component owner's freshly observed fork source no longer matches
    /// every shared exact axis of the admitted source basis, so forking it
    /// would silently create a branch from state the caller never observed.
    ForkSourceChanged,
}

impl ProductBranchObservation {
    pub(crate) fn owner_issued(
        snapshot: ProductBranchReferenceSnapshot,
        components: ObservationRetentionObligation,
        history: ExplicitCommitHistoryProtectionObligation,
    ) -> Result<Self, ProductBranchObservationAdmissionFailure> {
        if !components.matches_captured_head(snapshot.commit()) {
            return Err(ProductBranchObservationAdmissionFailure {
                denial: ProductBranchObservationAuthorityDenial::ComponentCommitOrBasisMismatch,
                snapshot,
                components,
                history,
            });
        }
        if !history.matches_commit(snapshot.commit())
            || history.owner_identity() != snapshot.owner()
        {
            return Err(ProductBranchObservationAdmissionFailure {
                denial: ProductBranchObservationAuthorityDenial::HistoryCommitOrOwnerMismatch,
                snapshot,
                components,
                history,
            });
        }
        Ok(Self {
            snapshot: Arc::new(snapshot),
            obligation: Arc::new(ProductBranchObservationObligation::owner_issued(
                components, history,
            )),
        })
    }

    pub fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.snapshot.owner()
    }

    pub fn branch_identity(&self) -> &ProductBranchIdentity {
        self.snapshot.branch()
    }

    pub fn lifecycle_incarnation(&self) -> ProductBranchIncarnation {
        self.snapshot.lifecycle()
    }

    pub fn reference_generation(&self) -> ProductBranchReferenceGeneration {
        self.snapshot.generation()
    }

    pub fn selected_commit(&self) -> &CompositeCommitIdentity {
        self.snapshot.commit().identity()
    }

    pub fn basis(&self) -> &AdmittedCompositeRuntimeWorldBasis {
        self.snapshot.commit().basis()
    }

    pub(crate) fn snapshot(&self) -> &ProductBranchReferenceSnapshot {
        &self.snapshot
    }

    pub(crate) fn mismatch_against_snapshot(
        &self,
        observed: &ProductBranchReferenceSnapshot,
    ) -> Option<ProductBranchObservationMismatch> {
        ProductBranchObservationMismatch::between(self.snapshot(), observed)
    }

    pub fn compare(&self, observed: &Self) -> Result<(), ProductBranchObservationMismatch> {
        self.mismatch_against_snapshot(observed.snapshot())
            .map_or(Ok(()), Err)
    }
}

#[cfg(test)]
#[path = "observation_tests.rs"]
mod tests;

impl std::fmt::Debug for ProductBranchObservationAdmissionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductBranchObservationAdmissionFailure")
            .field("denial", &self.denial)
            .field("snapshot", &self.snapshot)
            .field("components", &self.components)
            .field("history", &self.history)
            .finish()
    }
}
