use super::WorthQueryAdmittedOperation;

impl WorthQueryAdmittedOperation {
    pub fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.data.graph_work.identity()
    }

    pub fn graph_work_managed_run_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity {
        self.data.graph_work.managed_run_identity()
    }

    pub fn graph_work_branch(&self) -> &worth_relational::facade::history::BranchId {
        self.data.graph_work.branch().relational()
    }

    pub fn graph_work_decision_fact_count(&self) -> usize {
        self.data.graph_work.retained_decision_facts()
    }

    pub fn graph_work_runtime_ordinal(&self) -> u64 {
        self.data.graph_work.runtime_ordinal()
    }

    pub fn graph_work_principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.data.graph_work.principal()
    }

    pub fn graph_work_scope_entity_id(
        &self,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.data.graph_work.entity_access_context()
    }

    pub fn graph_work_capability_identity(&self) -> Option<[u8; 32]> {
        self.data.graph_work.capability_access_context()
    }

    pub fn graph_work_provider(&self) -> &str {
        self.data.graph_work.provider()
    }

    pub fn graph_work_mut(
        &mut self,
    ) -> &mut crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession {
        &mut self.data.graph_work
    }

    pub const fn graph_work(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession {
        &self.data.graph_work
    }
}
