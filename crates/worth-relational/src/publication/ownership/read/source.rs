//! Read-only custody sources keep candidate observations distinct from admitted bases.
use super::*;
#[derive(Clone, Copy)]
pub(super) enum OwnershipReadSource<'a> {
    Admitted(&'a AdmittedRelationalBranchBasis),
    Candidate(&'a crate::branch::RelationalBranchRoot),
}
impl<'a> OwnershipReadSource<'a> {
    pub(super) fn root(self) -> &'a crate::branch::RelationalBranchRoot {
        match self {
            Self::Admitted(basis) => &basis.inner.root,
            Self::Candidate(root) => root,
        }
    }
    pub(super) fn descriptor(self) -> Option<crate::branch::RelationalBranchBasisDescriptor> {
        match self {
            Self::Admitted(basis) => Some(basis.descriptor().clone()),
            Self::Candidate(_) => None,
        }
    }
    pub(super) fn demand(
        self,
        execution: &ExecutionRequest<'_, '_>,
        partition: crate::identity::data::PartitionId,
    ) -> Result<(), TransactionCommitError> {
        self.root()
            .demand_head_partition(execution, partition)
            .map_err(|cause| match self {
                Self::Admitted(basis) => {
                    crate::authority::commit::preparation::storage_read_denial(basis, cause)
                }
                Self::Candidate(_) => super::super::request(cause),
            })
    }
}
