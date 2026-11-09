//! Exact admitted operation targeting, distinct from the document's model root.
use super::HistoryModelIdentity;
use crate::domain_computation::{
    authorization::WorthQueryAdmittedApplicationOperation,
    execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    primary_graph::WorthQueryApplicationCommitReceipt,
};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::{EntityId, KindId};

#[derive(Clone, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct HistoryOperationScope {
    entity: EntityId,
    kind: KindId,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: ApplicationSchemaBindingIdentity,
}

impl HistoryOperationScope {
    pub(in crate::domain_computation::primary_graph) fn from_admitted<S, O, I, E>(
        admitted: &WorthQueryAdmittedApplicationOperation<S, O, I, E>,
    ) -> Self {
        Self {
            entity: admitted.scope_entity_id(),
            kind: admitted.scope_entity_kind(),
            runtime: admitted.runtime_authority(),
            binding: admitted.binding_identity().clone(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn belongs_to_model_runtime(
        &self,
        model: &HistoryModelIdentity,
    ) -> bool {
        model.matches_runtime(self.runtime, &self.binding)
    }
    pub(super) fn matches_operation<S, O, I, E>(
        &self,
        admitted: &WorthQueryAdmittedApplicationOperation<S, O, I, E>,
    ) -> bool {
        self.entity == admitted.scope_entity_id()
            && self.kind == admitted.scope_entity_kind()
            && self.runtime == admitted.runtime_authority()
            && &self.binding == admitted.binding_identity()
    }
    pub(super) fn matches_receipt(&self, receipt: &WorthQueryApplicationCommitReceipt) -> bool {
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
            self.entity,
        ) == receipt.principal_scope().scope()
            && self.runtime == receipt.authority_binding().runtime_authority()
            && &self.binding == receipt.principal_scope().binding_identity()
    }
    pub(super) fn from_resolved<S, E>(
        resolved: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            S,
            E,
        >,
    ) -> Self {
        Self {
            entity: resolved.entity_id(),
            kind: resolved.entity_kind(),
            runtime: resolved.runtime_authority(),
            binding: resolved.binding_identity().clone(),
        }
    }
}
