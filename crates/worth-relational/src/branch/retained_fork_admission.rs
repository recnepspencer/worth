//! Admit the fork receiver's live lifecycle before retaining its selected source.
use super::retained_fork_source::{
    require_retained_fork_cell, AdmittedRelationalRetainedForkSource,
};
use super::{AdmittedRelationalBranchBasis, RelationalForkDenial, RelationalForkPort};

impl RelationalForkPort {
    pub fn admit_retained_fork_source(
        &self,
        source: &AdmittedRelationalBranchBasis,
    ) -> Result<AdmittedRelationalRetainedForkSource, RelationalForkDenial> {
        self.validate_retained_fork_profile()?;
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        self.validate_retained_basis(source)?;
        Ok(AdmittedRelationalRetainedForkSource {
            basis: source.clone(),
        })
    }

    pub(super) fn validate_retained_fork_profile(&self) -> Result<(), RelationalForkDenial> {
        if self.medium.is_some() {
            return Err(RelationalForkDenial::RetainedSourceDurabilityUnsupported);
        }
        Ok(())
    }

    pub(super) fn validate_retained_basis(
        &self,
        source: &AdmittedRelationalBranchBasis,
    ) -> Result<(), RelationalForkDenial> {
        if source.identity().runtime_instance_id() != self.runtime_instance_id {
            return Err(RelationalForkDenial::ForeignRuntime);
        }
        let cell = self
            .owner
            .branch_cell(source.identity().branch_id())
            .ok_or(RelationalForkDenial::SourceBranchMissing)?;
        require_retained_fork_cell(source, &cell)
    }
}
