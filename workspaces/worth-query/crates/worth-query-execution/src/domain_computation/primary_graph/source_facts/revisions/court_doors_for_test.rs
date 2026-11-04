//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedAdjacencyRevision {
    #[doc(hidden)]
    pub fn new_for_test(
        anchor: EntityId,
        relation_kind: KindId,
        direction: worth_relational::facade::runtime::RelationalAdjacencyDirection,
        native_revision: Option<VersionId>,
        comparison_work_limit: usize,
        endpoints: Vec<EntityId>,
    ) -> Self {
        Self {
            anchor,
            relation_kind,
            direction,
            native_revision,
            comparison_work_limit,
            endpoints,
        }
    }
}

impl WorthQueryObservedFieldRevision {
    #[doc(hidden)]
    pub fn aspect_for_test(&self) -> &AspectKey {
        &self.aspect
    }
}

impl WorthQueryObservedFieldRevision {
    #[doc(hidden)]
    pub fn contract_revision_for_test(&self) -> &AspectContractRevision {
        &self.contract_revision
    }
}

impl WorthQueryObservedFieldRevision {
    #[doc(hidden)]
    pub fn entity_for_test(&self) -> &EntityId {
        &self.entity
    }
}

impl WorthQueryObservedFieldRevision {
    #[doc(hidden)]
    pub fn entity_name_for_test(&self) -> &String {
        &self.entity_name
    }
}

impl WorthQueryObservedFieldRevision {
    #[doc(hidden)]
    pub fn field_for_test(&self) -> &FieldKey {
        &self.field
    }
}

impl WorthQueryObservedAdjacencyRevision {
    #[doc(hidden)]
    pub fn anchor_for_test(&self) -> &EntityId {
        &self.anchor
    }
}

impl WorthQueryObservedAdjacencyRevision {
    #[doc(hidden)]
    pub fn comparison_work_limit_for_test(&self) -> &usize {
        &self.comparison_work_limit
    }
}

impl WorthQueryObservedAdjacencyRevision {
    #[doc(hidden)]
    pub fn endpoints_for_test(&self) -> &Vec<EntityId> {
        &self.endpoints
    }
}

impl WorthQueryObservedAdjacencyRevision {
    #[doc(hidden)]
    pub fn native_revision_for_test(&self) -> &Option<VersionId> {
        &self.native_revision
    }
}

impl WorthQueryObservedAdjacencyRevision {
    #[doc(hidden)]
    pub fn relation_kind_for_test(&self) -> &KindId {
        &self.relation_kind
    }
}
