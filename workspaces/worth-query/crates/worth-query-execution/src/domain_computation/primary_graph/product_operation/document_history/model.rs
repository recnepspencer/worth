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
