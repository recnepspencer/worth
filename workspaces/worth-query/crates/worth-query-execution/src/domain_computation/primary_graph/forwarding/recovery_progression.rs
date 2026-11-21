//! Thin forwarding shims for `recovery_progression` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_application_aftermath_recovery_progression_authority {
    use worth_query_execution_attempt::facade::domain_computation::application_aftermath::recovery_progression::authority::*;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandle;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleDenial;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_progression::disclose::WorthQueryRecoveryDisclosureAdmission;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn admit_recovery_effect_authority<Operation, Input, Scope>(&self, handle: &WorthQueryRecoveryHandle, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>) -> Result<WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandleDenial> {
            self.inner.admit_recovery_effect_authority::<Operation, Input, Scope>(handle, admission)
        }
        pub fn admit_recovery_inspect_authority<Operation, Input, Scope>(&self, handle: &WorthQueryRecoveryHandle, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, disclosure: &WorthQueryRecoveryDisclosureAdmission) -> Result<WorthQueryRecoveryInspectAuthority, WorthQueryRecoveryHandleDenial> {
            self.inner.admit_recovery_inspect_authority::<Operation, Input, Scope>(handle, admission, disclosure)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_application_aftermath_recovery_progression_disclose {
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleDenial;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_progression::disclose::WorthQueryRecoveryDisclosureAdmission;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn admit_recovery_inspection_disclosure<Capability, Operation, Input>(&self, access: &WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >) -> Result<WorthQueryRecoveryDisclosureAdmission, WorthQueryRecoveryHandleDenial>
        where
            Input: ApplicationCapabilityRequest<Schema, Capability>, {
            self.inner.admit_recovery_inspection_disclosure::<Capability, Operation, Input>(access)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_application_aftermath_recovery_progression_expiry {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandle;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleDenial;
    use worth_query_execution_attempt::facade::domain_computation::application_aftermath::recovery_progression::expiry::WorthQueryRecoveryExpiryEvaluation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn evaluate_recovery_expiry(&self, handle: &WorthQueryRecoveryHandle) -> Result<WorthQueryRecoveryExpiryEvaluation, WorthQueryRecoveryHandleDenial> {
            self.inner.evaluate_recovery_expiry(handle)
        }
    }
}
