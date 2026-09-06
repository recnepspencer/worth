use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::ProductBranchReferenceSnapshot;
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedOwnerEffects};

use super::custody::RetainedCommitDisposition;
use super::product_cas::{CompositePublicationReady, CompositePublicationReadyInputs};
use super::{
    CompositeAttemptProgress, ReservedCompositePublicationAttempt, ReservedPublicationAttemptParts,
};

/// Settled owner evidence and the same caller custody that was registered
/// before effects. Dropping this phase abandons that owner-held evidence.
pub struct OwnerExecutionSettlement {
    attempt: ReservedCompositePublicationAttempt,
    progress: CompositeAttemptProgress,
    successor_basis: Option<AdmittedCompositeRuntimeWorldBasis>,
}

impl std::fmt::Debug for OwnerExecutionSettlement {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OwnerExecutionSettlement")
            .field("progress", &self.progress)
            .field("successor_basis", &self.successor_basis)
            .finish_non_exhaustive()
    }
}

impl OwnerExecutionSettlement {
    pub(crate) fn new(
        attempt: ReservedCompositePublicationAttempt,
        progress: CompositeAttemptProgress,
    ) -> Self {
        Self {
            attempt,
            progress,
            successor_basis: None,
        }
    }

    pub(crate) fn with_successor_basis(
        attempt: ReservedCompositePublicationAttempt,
        progress: CompositeAttemptProgress,
        successor_basis: AdmittedCompositeRuntimeWorldBasis,
    ) -> Self {
        Self {
            attempt,
            progress,
            successor_basis: Some(successor_basis),
        }
    }

    #[cfg(test)]
    pub fn progress(&self) -> &CompositeAttemptProgress {
        &self.progress
    }

    pub(crate) fn successor_basis(&self) -> Option<&AdmittedCompositeRuntimeWorldBasis> {
        self.successor_basis.as_ref()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ReservedCompositePublicationAttempt,
        CompositeAttemptProgress,
    ) {
        (self.attempt, self.progress)
    }

    pub(crate) fn retain_with_cause(
        self,
        successor: AdmittedCompositeRuntimeWorldBasis,
        cause: ProductUnpublishedCause,
        observed: Option<ProductBranchReferenceSnapshot>,
    ) -> ProductUnpublishedOwnerEffects {
        let (attempt, progress) = self.into_parts();
        retain_publication(attempt.into_parts(), progress, successor, cause, observed)
    }

    pub(crate) fn ready(
        self,
        successor: AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<CompositePublicationReady, ProductUnpublishedOwnerEffects> {
        let (attempt, progress) = self.into_parts();
        let parts = attempt.into_parts();
        let Some((summary, results)) = progress.ready_results() else {
            return Err(retain_publication(
                parts,
                progress,
                successor,
                ProductUnpublishedCause::SettlementPending,
                None,
            ));
        };
        if parts.cancellation == super::CompositeAttemptCancellationPosture::CancellationObserved {
            return Err(retain_publication(
                parts,
                progress,
                successor,
                ProductUnpublishedCause::CancellationAfterEffect,
                None,
            ));
        }
        assert!(
            results.matches_plan(&parts.plan),
            "settled evidence matches the admitted plan"
        );
        let ReservedPublicationAttemptParts {
            identity,
            expected_head,
            mut custody,
            cancellation,
            deadline,
            counters,
            ..
        } = parts;
        let commit = custody.prepare_commit(successor, &results);
        if let Err(denial) = custody.bind_publication_pins(commit.basis()) {
            return Err(custody.retain(
                ProductUnpublishedCause::from_retention_denial(&denial),
                None,
                RetainedCommitDisposition::InstallSuccessor,
            ));
        }
        Ok(CompositePublicationReady::new(
            CompositePublicationReadyInputs {
                identity,
                expected_head,
                commit,
                owner_results: results,
                progress: summary,
                custody,
                cancellation,
                deadline,
                counters,
            },
        ))
    }
}

fn retain_publication(
    parts: ReservedPublicationAttemptParts,
    progress: CompositeAttemptProgress,
    successor: AdmittedCompositeRuntimeWorldBasis,
    cause: ProductUnpublishedCause,
    observed: Option<ProductBranchReferenceSnapshot>,
) -> ProductUnpublishedOwnerEffects {
    let (_, results) = progress
        .into_recovery_results()
        .expect("owner progress is representable for retention");
    let mut custody = parts.custody;
    custody.prepare_commit(successor, &results);
    custody.retain(cause, observed, RetainedCommitDisposition::InstallSuccessor)
}
