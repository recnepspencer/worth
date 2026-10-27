//! The real admitted root retained with its operation reader, never recovered from selectors.
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use std::sync::Arc;
use worth_relational::facade::identity::{EntityId, KindId};

pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) struct AdmittedReadRoot
{
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) entity:
        EntityId,
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) kind:
        KindId,
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) name:
        Arc<str>,
}
impl AdmittedReadRoot {
    pub(super) fn from_admission<Schema, Operation, Input, Scope>(
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Self {
        Self {
            entity: admission.scope_entity_id(),
            kind: admission.scope_entity_kind(),
            name: Arc::from(admission.scope_entity_name()),
        }
    }
}
