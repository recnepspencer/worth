use std::sync::Arc;

use worth_runtime_bridge::facade::RelationalCommittedPatchRequest;
use worth_runtime_world::facade::ConsumedCompositePublication;

use super::super::runtime::WorthQueryProductRootIdentity;

/// The one Query delivery authorized by a fresh performed product publication.
///
/// This witness is move-only. Its patch request is projected from World's
/// committed Relational result, so callers cannot substitute a commit or
/// snapshot. Recovered application receipts never contain this value.
#[derive(Debug)]
pub struct WorthQueryPerformedRelationalProductChange {
    pub(crate) root_identity: Arc<WorthQueryProductRootIdentity>,
    publication: Arc<ConsumedCompositePublication>,
    patch: RelationalCommittedPatchRequest,
}

impl WorthQueryPerformedRelationalProductChange {
    pub(crate) fn new(
        root_identity: Arc<WorthQueryProductRootIdentity>,
        publication: Arc<ConsumedCompositePublication>,
    ) -> Self {
        let relational = publication
            .component_results()
            .relational_commit_result()
            .expect("a performed Relational application publication retains its result");
        let commit = relational.commit.commit_id;
        let snapshot = worth_relational::facade::bridge::bridge_snapshot_identity_for_handle(
            &relational.snapshot,
        );
        Self {
            root_identity,
            publication,
            patch: RelationalCommittedPatchRequest::at_snapshot(
                worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(
                    commit.0,
                ),
                snapshot,
            ),
        }
    }

    pub fn product_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.publication.commit().identity()
    }

    pub(crate) fn signal_basis(&self) -> &worth_signal::facade::branch::AdmittedSignalBranchBasis {
        self.publication.new_product_head().basis().signal_basis()
    }

    pub(crate) fn patch(&self) -> RelationalCommittedPatchRequest {
        self.patch.clone()
    }
}

impl PartialEq for WorthQueryPerformedRelationalProductChange {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.root_identity, &other.root_identity)
            && self.publication.attempt_identity() == other.publication.attempt_identity()
    }
}

impl Eq for WorthQueryPerformedRelationalProductChange {}
