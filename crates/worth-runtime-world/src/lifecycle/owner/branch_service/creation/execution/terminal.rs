use crate::branch::RuntimeWorldBranchAdmissionDenial;
use crate::publication::{
    CompositeAttemptProgress, ReservedBranchCreationAttempt, RetainedCommitDisposition,
    RuntimeWorldCancellationToken,
};
use crate::recovery::ProductUnpublishedCause;

use super::super::super::super::RuntimeWorldOwnerRoot;
use super::BranchCreationExecution;

/// One creation attempt at the point where its owner legs are done with it:
/// the reservations it still owns and the exact evidence it produced. Every
/// creation terminal is handed the pair together, because neither half names a
/// terminal on its own.
pub(super) struct SettledCreationAttempt {
    pub(super) attempt: ReservedBranchCreationAttempt,
    pub(super) progress: CompositeAttemptProgress,
}

/// How one post-Relational denial is named on both sides of the effect
/// boundary: the cause a retained record carries, and the admission denial the
/// caller sees when no owner moved.
pub(super) struct CreationDenialNaming {
    pub(super) cause: ProductUnpublishedCause,
    pub(super) denial: RuntimeWorldBranchAdmissionDenial,
}

/// Classify a post-Relational denial. A creation that has not moved an owner
/// yet is a plain admission denial; one that has is retained, never discarded.
pub(super) fn retain_or_no_effect<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    settled: SettledCreationAttempt,
    naming: CreationDenialNaming,
) -> BranchCreationExecution
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let SettledCreationAttempt {
        mut attempt,
        progress,
    } = settled;
    if progress.owner_effect_count() == 0 {
        return BranchCreationExecution::NoEffect(naming.denial);
    }
    attempt.record_progress(&progress);
    attempt.begin_publication();
    // A head that moved under a performed fork is the one cause whose record
    // can name the occurrence that displaced it, so the retained evidence
    // carries the winner the caller must reconcile against.
    let last_observed_head = if naming.cause == ProductUnpublishedCause::StaleProductHead {
        owner.current_product_head_snapshot(attempt.source())
    } else {
        None
    };
    let successor_basis =
        owner.issue_successor_basis_from_progress(&progress, attempt.source().basis());
    BranchCreationExecution::ProductUnpublished(retain_creation_effects(
        attempt,
        RetainedCreation {
            progress,
            successor_basis,
            cause: naming.cause,
            last_observed_head,
        },
    ))
}

/// The settled creation terminal. The two owner forks are complete and the
/// destination basis is admitted; installing the product reference is the
/// caller's next step.
pub(super) fn settle_creation<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    settled: SettledCreationAttempt,
    cancellation: &RuntimeWorldCancellationToken,
) -> BranchCreationExecution
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let SettledCreationAttempt {
        mut attempt,
        progress,
    } = settled;
    attempt.record_progress(&progress);
    attempt.begin_publication();
    let successor_basis =
        owner.issue_successor_basis_from_progress(&progress, attempt.source().basis());
    attempt.record_successor(successor_basis.clone());
    if cancellation.is_cancelled() {
        return BranchCreationExecution::ProductUnpublished(retain_creation_effects(
            attempt,
            RetainedCreation {
                progress,
                successor_basis,
                cause: ProductUnpublishedCause::CancellationAfterEffect,
                last_observed_head: None,
            },
        ));
    }
    if !owner.successor_correspondence_is_valid(&successor_basis) {
        return BranchCreationExecution::ProductUnpublished(retain_creation_effects(
            attempt,
            RetainedCreation {
                progress,
                successor_basis,
                cause: ProductUnpublishedCause::CorrespondenceRebindRequired,
                last_observed_head: None,
            },
        ));
    }
    BranchCreationExecution::Settled {
        attempt,
        progress,
        successor_basis,
    }
}

/// Retain a creation's performed forks. The plan match enforced here is the
/// creation plan; a creation can never satisfy a publication plan. Retention
/// is reached from a sibling denial, so the later owner leg is legitimately
/// untouched: only the legs that moved are held to their plan.
fn retain_creation_effects(
    attempt: ReservedBranchCreationAttempt,
    retained: RetainedCreation,
) -> crate::recovery::ProductUnpublishedOwnerEffects {
    let RetainedCreation {
        progress,
        successor_basis,
        cause,
        last_observed_head,
    } = retained;
    let parts = attempt.into_parts();
    let (_, owner_results) = progress
        .into_recovery_results()
        .expect("creation fork evidence is representable owner progress");
    assert!(
        owner_results.matches_partial_creation_plan(&parts.plan),
        "every retained creation effect must be the evidence its plan asked for"
    );
    let mut custody = parts.custody;
    custody.prepare_commit(successor_basis, &owner_results);
    custody.retain(
        cause,
        last_observed_head,
        RetainedCommitDisposition::InstallSuccessor,
    )
}

/// The retained image of one creation that moved an owner and cannot install a
/// product reference.
struct RetainedCreation {
    progress: CompositeAttemptProgress,
    successor_basis: crate::basis::AdmittedCompositeRuntimeWorldBasis,
    cause: ProductUnpublishedCause,
    last_observed_head: Option<crate::branch::ProductBranchReferenceSnapshot>,
}
