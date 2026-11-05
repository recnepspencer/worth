//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderExecutionPlanContract {
    #[doc(hidden)]
    pub fn application_operation_attempt_for_test(
        &self,
    ) -> Option<crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity>
{
        Self::application_operation_attempt(self)
    }
}

impl WorthQueryProviderExecutionPlanContract {
    #[doc(hidden)]
    pub fn application_operation_slot_for_test(&self) -> Option<&str> {
        Self::application_operation_slot(self)
    }
}

impl WorthQueryProviderExecutionPlanContract {
    #[doc(hidden)]
    pub fn application_schema_binding_for_test(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationSchemaBindingIdentity> {
        Self::application_schema_binding(self)
    }
}

impl WorthQueryProviderExecutionPlanContract {
    #[doc(hidden)]
    pub fn application_snapshot_for_test(
        &self,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        Self::application_snapshot(self)
    }
}

impl WorthQueryProviderExecutionPlanContract {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(
        &self,
    ) -> crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity{
        Self::runtime_authority(self)
    }
}
