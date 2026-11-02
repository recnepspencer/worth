use super::WorthQueryAdmittedOperation;

impl WorthQueryAdmittedOperation {
    pub(in crate::domain_computation) fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.graph_work.identity()
    }

    pub(in crate::domain_computation) fn graph_work_managed_run_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity {
        self.graph_work.managed_run_identity()
    }

    pub(in crate::domain_computation) fn graph_work_branch(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        self.graph_work.branch().relational()
    }

    pub(in crate::domain_computation) fn graph_work_decision_fact_count(&self) -> usize {
        self.graph_work.retained_decision_facts()
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn graph_work_runtime_ordinal(&self) -> u64 {
        self.graph_work.runtime_ordinal()
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn graph_work_principal_entity_id(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.graph_work.principal()
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn graph_work_scope_entity_id(
        &self,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.graph_work.entity_access_context()
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn graph_work_capability_identity(&self) -> Option<[u8; 32]> {
        self.graph_work.capability_access_context()
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn graph_work_provider(&self) -> &str {
        self.graph_work.provider()
    }

    pub(in crate::domain_computation) fn graph_work_mut(
        &mut self,
    ) -> &mut crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession {
        &mut self.graph_work
    }

    pub(in crate::domain_computation) const fn graph_work(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession {
        &self.graph_work
    }
}
