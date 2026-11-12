//! Home carriage remains intact after capability-to-operation progression.
use super::WorthQueryAdmittedApplicationOperation;
impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn refuse_stale_home_authority(
        &self,
        current: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        self.core.refuse_stale_home_authority(current)
    }
}

impl super::WorthQueryAdmittedOperation {
    pub(in crate::domain_computation) fn refuse_stale_home_authority(
        &self,
        current: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        crate::domain_computation::authorization::refuse_stale_home_authority(
            current,
            self.home_owner,
            &self.data.operation,
        )
    }
}
