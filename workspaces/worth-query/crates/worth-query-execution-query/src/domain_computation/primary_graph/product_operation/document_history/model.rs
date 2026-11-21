use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::{EntityId, KindId};

/// Copied identity correspondence, never a retained permission or replacement lookup.
#[derive(Clone, Eq, PartialEq)]
pub struct HistoryModelIdentity {
    entity: EntityId,
    kind: KindId,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: ApplicationSchemaBindingIdentity,
}

impl HistoryModelIdentity {
    pub fn matches_runtime(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime == runtime && &self.binding == binding
    }
    pub fn entity(&self) -> EntityId {
        self.entity
    }
    pub fn kind(&self) -> KindId {
        self.kind
    }
    pub fn record(
        &self,
    ) -> worth_relational::facade::transactions::RecordRef {
        worth_relational::facade::transactions::RecordRef::Entity(self.entity)
    }
    pub fn matches_operation<S, O, I, E>(
        &self,
        admitted: &crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            S,
            O,
            I,
            E,
        >,
    ) -> bool {
        self.entity == admitted.scope_entity_id()
            && self.kind == admitted.scope_entity_kind()
            && self.runtime == admitted.runtime_authority()
            && &self.binding == admitted.binding_identity()
    }

    pub fn from_resolved<S, E>(
        resolved: &WorthQueryApplicationEntityIdentity<S, E>,
    ) -> Self {
        Self {
            entity: resolved.entity_id(),
            kind: resolved.entity_kind(),
            runtime: resolved.runtime_authority(),
            binding: resolved.binding_identity().clone(),
        }
    }
}

mod court_doors_for_test;
