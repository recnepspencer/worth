//! Consume this exact service bundle's local medium shares, retaining metadata.
use super::RelationalOwnerServicePorts;
use crate::durability::medium::RelationalMediumRetirementDenial;
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;
#[derive(Debug)]
pub enum RelationalOwnerServiceRetirementDenial {
    ForeignOwner,
    Medium(RelationalMediumRetirementDenial),
}
/// Retired metadata exposes no publication, fork or preparation port.
/// ```compile_fail,E0599
/// use worth_relational::facade::branch::RetiredRelationalOwnerServicePorts;
/// fn cannot_publish(retired: RetiredRelationalOwnerServicePorts) { retired.publication_port(); }
/// ```
#[derive(Debug)]
#[must_use]
pub struct RetiredRelationalOwnerServicePorts {
    _original: RelationalOwnerServicePorts,
}
impl RelationalOwnerServicePorts {
    pub fn into_home_retirement(
        mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<RetiredRelationalOwnerServicePorts, (Self, RelationalOwnerServiceRetirementDenial)>
    {
        if !self.basis.matches_home_retirement_owner(owner) {
            return Err((self, RelationalOwnerServiceRetirementDenial::ForeignOwner));
        }
        if let Err(cause) = self.publication.retire_home_service_medium_aliases(
            request,
            owner,
            &mut self.fork.medium,
        ) {
            return Err((self, RelationalOwnerServiceRetirementDenial::Medium(cause)));
        }
        Ok(RetiredRelationalOwnerServicePorts { _original: self })
    }
}
impl RetiredRelationalOwnerServicePorts {
    pub fn reclaim(self) {
        drop(self);
    }
}
