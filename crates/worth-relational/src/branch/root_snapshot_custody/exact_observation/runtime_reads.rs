//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

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
