use std::sync::Arc;

use worth_runtime_bridge::facade::RelationalCommittedPatchRequest;

use super::super::runtime::WorthQueryProductRootIdentity;

/// The one Query delivery authorized by a fresh performed product publication.
///
/// This witness is move-only. Its patch request is projected from World's
/// committed Relational result, so callers cannot substitute a commit or
/// snapshot. Recovered application receipts never contain this value.
#[derive(Debug)]
pub struct WorthQueryPerformedRelationalProductChange {
    pub(crate) root_identity: Arc<WorthQueryProductRootIdentity>,
    publication: super::super::WorthQueryProductPublicationReceipt,
    patch: RelationalCommittedPatchRequest,
}

impl WorthQueryPerformedRelationalProductChange {
    pub(crate) fn new(publication: super::super::WorthQueryProductPublicationReceipt) -> Self {
        let root_identity = publication.root_identity();
        let relational = publication
            .publication()
            .component_results()
            .relational_commit_result()
            .expect("a performed Relational application publication retains its result");
        let commit = relational.commit.commit_id;
        let branch = publication
            .publication()
            .new_product_head()
            .basis()
            .relational_basis()
            .descriptor()
            .branch_id()
            .0
            .clone();
        Self {
            root_identity,
            publication,
            patch: RelationalCommittedPatchRequest::on_branch(
                worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(
                    commit.0,
                ),
                worth_runtime_bridge::facade::TruthBranchIdentity::from_relational_branch_id(
                    branch,
                ),
            ),
        }
    }

    pub fn product_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.publication.publication().commit().identity()
    }

    pub fn product_branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.publication
            .publication()
            .new_product_head()
            .branch_identity()
    }

    pub(crate) fn signal_basis(&self) -> &worth_signal::facade::branch::AdmittedSignalBranchBasis {
        self.publication
            .publication()
            .new_product_head()
            .basis()
            .signal_basis()
    }

    pub(crate) fn patch(&self) -> RelationalCommittedPatchRequest {
        self.patch.clone()
    }
}

impl PartialEq for WorthQueryPerformedRelationalProductChange {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.root_identity, &other.root_identity)
            && self.publication.publication().attempt_identity()
                == other.publication.publication().attempt_identity()
    }
}

impl Eq for WorthQueryPerformedRelationalProductChange {}
