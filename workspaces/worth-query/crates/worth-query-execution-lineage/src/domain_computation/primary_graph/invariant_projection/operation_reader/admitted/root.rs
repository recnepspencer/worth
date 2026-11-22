//! The real admitted root retained with its operation reader, never recovered from selectors.
use crate::ports::AdmittedOperationPort;
use std::sync::Arc;
use worth_relational::facade::identity::{EntityId, KindId};

pub struct AdmittedReadRoot {
    pub entity: EntityId,
    pub kind: KindId,
    pub name: Arc<str>,
}

impl AdmittedReadRoot {
    pub fn from_admission(admission: &dyn AdmittedOperationPort) -> Self {
        Self {
            entity: admission.scope_entity_id(),
            kind: admission.scope_entity_kind(),
            name: Arc::from(admission.scope_entity_name()),
        }
    }
}
