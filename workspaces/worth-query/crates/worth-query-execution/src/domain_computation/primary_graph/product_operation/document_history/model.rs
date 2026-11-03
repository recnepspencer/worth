use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::{EntityId, KindId};

/// Copied identity correspondence, never a retained permission or replacement lookup.
#[derive(Clone, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct HistoryModelIdentity {
    entity: EntityId,
    kind: KindId,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: ApplicationSchemaBindingIdentity,
}

impl HistoryModelIdentity {
    pub(super) fn matches_runtime(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime == runtime && &self.binding == binding
    }
    pub(in crate::domain_computation::primary_graph) fn entity(&self) -> EntityId {
        self.entity
    }
    pub(super) fn kind(&self) -> KindId {
        self.kind
    }
    pub(in crate::domain_computation::primary_graph) fn matches_output_source<Q>(
        &self,
        source: &crate::domain_computation::primary_graph::WorthQueryObservedSource<Q>,
        observation: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    ) -> bool {
        self.entity == source.model_root
            && self.runtime.as_u64() == source.runtime_authority
            && self.binding == source.schema_binding
            && matches!(&source.selection,
                crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(product)
                    if product.matches_observation(observation.product_observation()))
    }
    pub(in crate::domain_computation::primary_graph) fn record(
        &self,
    ) -> worth_relational::facade::transactions::RecordRef {
        worth_relational::facade::transactions::RecordRef::Entity(self.entity)
    }
    pub(in crate::domain_computation::primary_graph) fn matches_operation<S, O, I, E>(
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

    pub(in crate::domain_computation::primary_graph) fn from_resolved<S, E>(
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
