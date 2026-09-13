use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::{
    ComponentBranchTarget, OwnerCreatedComponentCustodyRecord, ReservedCustodySlot,
    RuntimeWorldBranchAdmissionDenial,
};
use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};
use crate::publication::{
    CompositeAttemptProgress, ReservedBranchCreationAttempt, RuntimeWorldCancellationToken,
    SignalAttemptProgress,
};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedOwnerEffects};

use super::super::super::RuntimeWorldOwnerRoot;

#[cfg(test)]
#[path = "execution/boundary_control.rs"]
mod boundary_control;
#[cfg(test)]
#[path = "execution/custody_tests.rs"]
mod custody_tests;
#[path = "execution/forks.rs"]
mod forks;
#[cfg(test)]
#[path = "execution/owner_contact_tests.rs"]
mod owner_contact_tests;
#[path = "execution/terminal.rs"]
mod terminal;

use forks::{fork_relational, fork_signal, ForkFailure};

/// Where the two owner fork calls of one creation ended.
pub(super) enum BranchCreationExecution {
    NoEffect(RuntimeWorldBranchAdmissionDenial),
    ProductUnpublished(ProductUnpublishedOwnerEffects),
    Settled {
        attempt: ReservedBranchCreationAttempt,
        progress: CompositeAttemptProgress,
        successor_basis: AdmittedCompositeRuntimeWorldBasis,
    },
}

/// The destination identity the created custody records are keyed by. It is
/// issued before the first owner fork, so a performed fork is always
/// attributable to an exact product-branch occurrence.
pub(super) struct CreationDestination {
    pub(super) witness: std::sync::Arc<crate::branch::registry::ProductBranchInstallationWitness>,
    pub(super) branch: ProductBranchIdentity,
    pub(super) incarnation: ProductBranchIncarnation,
}

/// Run the Relational fork, then the Signal fork, in that fixed order. No
/// cross-owner lock is held across the two calls; the boundary between them is
/// rechecked exactly as a publication rechecks it.
pub(super) fn execute_creation<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    mut attempt: ReservedBranchCreationAttempt,
    destination: &CreationDestination,
    cancellation: &RuntimeWorldCancellationToken,
) -> BranchCreationExecution
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    attempt.bind_destination(std::sync::Arc::clone(&destination.witness));
    attempt.begin_owner_execution();
    if let Err(denial) = owner.creation_execution_is_admissible(&attempt, cancellation) {
        return BranchCreationExecution::NoEffect(denial);
    }

    let relational = match fork_relational(owner, &mut attempt, destination) {
        Ok(progress) => progress,
        Err(ForkFailure { denial }) => return BranchCreationExecution::NoEffect(denial),
    };
    let progress = CompositeAttemptProgress::new(relational, SignalAttemptProgress::untouched());

    #[cfg(test)]
    boundary_control::pause_between_creation_forks(owner.owner_identity());

    if let Err(naming) = owner.creation_boundary_gate(&attempt, cancellation) {
        return terminal::retain_or_no_effect(
            owner,
            terminal::SettledCreationAttempt { attempt, progress },
            naming,
        );
    }

    let signal = match fork_signal(owner, &mut attempt, destination, cancellation) {
        Ok(progress) => progress,
        Err(ForkFailure { denial }) => {
            return terminal::retain_or_no_effect(
                owner,
                terminal::SettledCreationAttempt { attempt, progress },
                terminal::CreationDenialNaming {
                    cause: if denial == RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect {
                        ProductUnpublishedCause::CancellationAfterEffect
                    } else {
                        ProductUnpublishedCause::SiblingOwnerDenied
                    },
                    denial,
                },
            )
        }
    };
    let (relational, _) = progress.into_parts();
    let progress = CompositeAttemptProgress::new(relational, signal);

    terminal::settle_creation(
        owner,
        terminal::SettledCreationAttempt { attempt, progress },
        cancellation,
    )
}

/// Attribute one performed owner fork to the destination product branch. The
/// slot was charged before the fork, so installation is total.
fn install_custody(
    slot: ReservedCustodySlot,
    destination: &CreationDestination,
    target: ComponentBranchTarget,
) {
    slot.install(OwnerCreatedComponentCustodyRecord::new(
        destination.branch.clone(),
        destination.incarnation,
        target,
    ));
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The last pre-effect recheck before the first owner fork. Nothing has
    /// moved yet, so every way this closes is a plain admission denial, and
    /// they are named apart because a displaced or retired source head is not
    /// an unavailable owner.
    fn creation_execution_is_admissible(
        &self,
        attempt: &ReservedBranchCreationAttempt,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<(), RuntimeWorldBranchAdmissionDenial> {
        if cancellation.is_cancelled() {
            return Err(RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect);
        }
        if self.deadline_expired(attempt.deadline()) {
            return Err(RuntimeWorldBranchAdmissionDenial::DeadlineBeforeEffect);
        }
        self.admit_source_head(attempt.source())
    }

    /// The between-owners boundary for a creation. A denial here may already
    /// have moved the Relational owner, so the caller retains rather than
    /// discards. A cancelled or expired attempt is retained as cancellation
    /// after effect; a displaced or retired source head is retained as a
    /// stale product head, and the displaced one alone carries the head that
    /// won.
    fn creation_boundary_gate(
        &self,
        attempt: &ReservedBranchCreationAttempt,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<(), terminal::CreationDenialNaming> {
        if cancellation.is_cancelled() {
            return Err(terminal::CreationDenialNaming {
                cause: ProductUnpublishedCause::CancellationAfterEffect,
                denial: RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect,
            });
        }
        if self.deadline_expired(attempt.deadline()) {
            return Err(terminal::CreationDenialNaming {
                cause: ProductUnpublishedCause::DeadlineAfterEffect,
                denial: RuntimeWorldBranchAdmissionDenial::DeadlineBeforeEffect,
            });
        }
        if let Err(denial) = self.admit_source_head(attempt.source()) {
            return Err(terminal::CreationDenialNaming {
                cause: ProductUnpublishedCause::StaleProductHead,
                denial,
            });
        }
        Ok(())
    }
}
