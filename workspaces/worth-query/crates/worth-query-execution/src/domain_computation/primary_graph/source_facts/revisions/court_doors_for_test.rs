//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedAdjacencyRevision {
    #[doc(hidden)]
    pub fn new_for_test(anchor: EntityId, relation_kind: KindId, direction: worth_relational::facade::runtime::RelationalAdjacencyDirection, native_revision: Option<VersionId>, comparison_work_limit: usize, endpoints: Vec<EntityId>) -> Self {
        Self { anchor, relation_kind, direction, native_revision, comparison_work_limit, endpoints }
    }
}
