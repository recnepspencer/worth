use super::AdmittedRelationalBranchBasis;
use super::{RelationalBranchRoot, RelationalBranchRootState};
use crate::identity::data::VersionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};
use worth_foundational::FoundationalBranchTarget;

#[derive(Debug, Clone)]
pub(crate) enum SelectedRelationalBranchState {
    Empty(RelationalBranchRootState),
    Committed(RelationalBranchRootState),
}

impl PartialEq for SelectedRelationalBranchState {
    fn eq(&self, other: &Self) -> bool {
        self.state().root().id() == other.state().root().id()
    }
}

impl Eq for SelectedRelationalBranchState {}

impl SelectedRelationalBranchState {
    pub(crate) fn from_admitted_basis(basis: &super::AdmittedRelationalBranchBasis) -> Self {
        let state = RelationalBranchRootState::new(basis.inner.root.clone());
        match basis.descriptor().reference().target() {
            FoundationalBranchTarget::Empty => Self::Empty(state),
            FoundationalBranchTarget::Basis(_) => Self::Committed(state),
        }
    }

    pub(crate) fn state(&self) -> &RelationalBranchRootState {
        match self {
            Self::Empty(state) | Self::Committed(state) => state,
        }
    }

    pub(crate) fn version_id(&self) -> VersionId {
        match self {
            Self::Empty(_) => VersionId(0),
            Self::Committed(state) => state.version_id(),
        }
    }

    pub(crate) fn root(&self) -> Option<&std::sync::Arc<RelationalBranchRoot>> {
        match self {
            Self::Empty(_) => None,
            Self::Committed(state) => Some(state.root()),
        }
    }
}

impl PartitionAccess for SelectedRelationalBranchState {
    fn get_partition(
        &self,
        partition_id: crate::identity::data::PartitionId,
    ) -> Option<&PartitionState> {
        self.state().get_partition(partition_id)
    }

    fn partition_ids(&self) -> Vec<crate::identity::data::PartitionId> {
        self.state().partition_ids()
    }

    fn partition_ids_iter(
        &self,
    ) -> Box<dyn Iterator<Item = crate::identity::data::PartitionId> + '_> {
        self.state().partition_ids_iter()
    }
}

impl crate::runtime::RelationalRuntime {
    pub(crate) fn selected_branch_state(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<SelectedRelationalBranchState, crate::transactions::data::CommitPreparationError>
    {
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(
                crate::transactions::data::CommitPreparationError::selected_branch_root_reference_mismatch(
                    basis.identity().branch_id().clone(),
                    basis.observation().commit_id().map(|id| id.0),
                    basis.observation().version_id(),
                ),
            );
        }
        Ok(SelectedRelationalBranchState::from_admitted_basis(basis))
    }
}
