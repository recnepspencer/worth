//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::exact_observation::VisibilityProjectionViewOwnerPort as _;

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Project through an owner-issued exact branch observation without
    /// allocating a tracked snapshot handle.
    pub fn project_observation(
        &self,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<VisibilityProjectionView, crate::branch::basis_denial::RelationalBranchBasisDenial>
    {
        VisibilityProjectionView::from_observation(self.runtime().into(), observation)
    }
}
