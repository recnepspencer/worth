use crate::branch::root::RelationalBranchRoot;
use crate::branch::root::RelationalBranchRootState;
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
    pub(crate) fn with_symbol_admission<'read>(
        &'read self,
        symbols: &'read crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> crate::branch::symbol_read::SymbolAdmittedRootRead<'read> {
        crate::branch::symbol_read::SymbolAdmittedRootRead::new(self.state().root(), symbols)
    }

    pub(crate) fn from_admitted_basis(
        basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
    ) -> Self {
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
    fn probe_full_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        self.state().probe_full_partition(id)
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.state().demand_full_partition(execution, id)
    }

    fn get_partition(
        &self,
        partition_id: crate::identity::data::PartitionId,
    ) -> Option<&PartitionState> {
        self.state().get_partition(partition_id)
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for SelectedRelationalBranchState {
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.state().get_current_partition(id)
    }
    fn probe_head_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        self.state().probe_head_partition(id)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.state().demand_head_partition(execution, id)
    }

    fn partition_ids(&self) -> Vec<crate::identity::data::PartitionId> {
        self.state().partition_ids()
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<crate::identity::data::PartitionId>, crate::storage::overlay::PartitionIdsDenial>
    {
        self.state().partition_ids_with_memory(memory)
    }

    fn partition_ids_iter(
        &self,
    ) -> Box<dyn Iterator<Item = crate::identity::data::PartitionId> + '_> {
        self.state().partition_ids_iter()
    }
}
