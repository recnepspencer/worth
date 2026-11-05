//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn governed_input_identity_for_test(&self) -> Option<&[u8; 32]> {
        Self::governed_input_identity(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn governed_proposal_identity_for_test(&self) -> Option<&[u8; 32]> {
        Self::governed_proposal_identity(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_for_test(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession {
        Self::graph_work(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_capability_identity_for_test(&self) -> Option<[u8; 32]> {
        Self::graph_work_capability_identity(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_decision_fact_count_for_test(&self) -> usize {
        Self::graph_work_decision_fact_count(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_managed_run_identity_for_test(
        &self,
    ) -> crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkManagedRunIdentity {
        Self::graph_work_managed_run_identity(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_provider_for_test(&self) -> &str {
        Self::graph_work_provider(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_runtime_ordinal_for_test(&self) -> u64 {
        Self::graph_work_runtime_ordinal(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_scope_entity_id_for_test(
        &self,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        Self::graph_work_scope_entity_id(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn graph_work_session_identity_for_test(
        &self,
    ) -> crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity {
        Self::graph_work_session_identity(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn operation_definition_identity_for_test(&self) -> &[u8; 32] {
        Self::operation_definition_identity(self)
    }
}
