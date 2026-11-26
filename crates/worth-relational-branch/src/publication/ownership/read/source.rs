//! Read-only custody sources keep candidate observations distinct from admitted bases.
use super::*;
#[derive(Clone, Copy)]
pub(super) enum OwnershipReadSource<'a> {
    Admitted(&'a AdmittedRelationalBranchBasis),
    Candidate(&'a crate::branch::root::RelationalRootRead<'a>),
}
impl<'a> OwnershipReadSource<'a> {
    pub(super) fn root(self) -> &'a crate::branch::root::RelationalBranchRoot {
        match self {
            Self::Admitted(basis) => &basis.inner.root,
            Self::Candidate(read) => read.root(),
        }
    }
    /// Partitions admitted by this source's own operation residency.
    pub(super) fn partitions(self) -> &'a dyn crate::storage::overlay::CurrentPartitionAccess {
        match self {
            Self::Admitted(basis) => basis,
            Self::Candidate(read) => read,
        }
    }
    pub(super) fn descriptor(
        self,
    ) -> Option<crate::branch::basis::RelationalBranchBasisDescriptor> {
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
        self.partitions()
            .demand_head_partition(execution, partition)
            .map_err(|cause| match self {
                Self::Admitted(basis) => {
                    crate::authority::commit::preparation::storage_denial::storage_read_denial(
                        basis, cause,
                    )
                }
                Self::Candidate(_) => super::super::read_request::request(cause),
            })
    }
}
