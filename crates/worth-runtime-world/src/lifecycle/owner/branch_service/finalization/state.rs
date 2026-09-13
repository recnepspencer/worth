use std::sync::Arc;

use crate::branch::registry::ProductBranchRegistryReservation;
use crate::branch::ProductBranchReferenceSnapshot;
use crate::history::{CompositeHistoryCatalog, CompositeRuntimeWorldCommit};
use crate::identity::{
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchReferenceGeneration,
};
use crate::publication::{ActiveAttemptCustody, RetainedCommitDisposition};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedOwnerEffects};
use crate::retention::RuntimeWorldRetentionOwner;

use super::ForkedBranchInstallation;

/// Exact settled creation facts and the original linear owner-custody token.
/// Phase transitions borrow its resources; no phase recreates a recovery lane.
pub(super) struct ForkedBranchFinalization {
    pub(super) branch: ProductBranchIdentity,
    lifecycle: ProductBranchIncarnation,
    pub(super) reservation: ProductBranchRegistryReservation,
    pub(super) custody: ActiveAttemptCustody,
    commit: Arc<CompositeRuntimeWorldCommit>,
}

pub(super) struct PublicationBoundForkedBranch(ForkedBranchFinalization);

pub(super) struct ObservedForkedBranch {
    pub(super) state: ForkedBranchFinalization,
}

impl ForkedBranchFinalization {
    pub(super) fn from_installation(installation: ForkedBranchInstallation) -> Self {
        let ForkedBranchInstallation {
            branch,
            lifecycle,
            reservation,
            attempt,
            progress,
            successor_basis,
        } = installation;
        let (_, results) = progress
            .into_ready_results()
            .expect("settled creation carries ready results");
        let parts = attempt.into_parts();
        assert!(
            results.matches_creation_plan(&parts.plan),
            "owner evidence matches the admitted creation"
        );
        let mut custody = parts.custody;
        let commit = custody.prepare_commit(successor_basis, &results);
        Self {
            branch,
            lifecycle,
            reservation,
            custody,
            commit,
        }
    }

    pub(super) fn bind_publication(
        mut self,
    ) -> Result<PublicationBoundForkedBranch, ProductUnpublishedOwnerEffects> {
        if let Err(denial) = self.custody.bind_publication_pins(self.commit.basis()) {
            return Err(self.retain(
                ProductUnpublishedCause::from_retention_denial(&denial),
                None,
            ));
        }
        Ok(PublicationBoundForkedBranch(self))
    }

    pub(super) fn retain(
        self,
        cause: ProductUnpublishedCause,
        observed: Option<ProductBranchReferenceSnapshot>,
    ) -> ProductUnpublishedOwnerEffects {
        drop(self.reservation);
        self.custody
            .retain(cause, observed, RetainedCommitDisposition::InstallSuccessor)
    }
}

impl PublicationBoundForkedBranch {
    /// History installation atomically acquires protection. Component observation
    /// acquisition then runs outside all World locks with custody still leased.
    pub(super) fn observe<D, I, T>(
        self,
        retention: &RuntimeWorldRetentionOwner<D, I, T>,
        history: &CompositeHistoryCatalog,
    ) -> Result<ObservedForkedBranch, ProductUnpublishedOwnerEffects>
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        let mut state = self.0;
        state.custody.install_creation_history(&state.commit);
        let snapshot = ProductBranchReferenceSnapshot::owner_issued(
            state.branch.owner_identity(),
            state.branch.clone(),
            state.lifecycle,
            ProductBranchReferenceGeneration::initial(),
            Arc::clone(&state.commit),
        )
        .expect("the admitted destination forms its initial snapshot");
        if let Err(cause) =
            state
                .custody
                .issue_creation_observation(retention, history, snapshot.clone())
        {
            return Err(state.retain(cause, None));
        }
        state.custody.prepare_creation_cell(snapshot);
        Ok(ObservedForkedBranch { state })
    }
}
