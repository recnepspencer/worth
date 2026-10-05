use std::collections::BTreeSet;

use worth_relational::facade::identity::EntityId;

/// Exact entity identities reached while one pinned invariant projection ran.
///
/// This is carried proof, not a caller-authored touch assertion. Only the
/// installation-owned projection reader can add identities.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRealizedProjectionScope {
    entity_ids: BTreeSet<EntityId>,
    /// Every identity one owner call of a partitioned computation reached,
    /// already reached or not, while that call holds the reader.
    call: Option<BTreeSet<EntityId>>,
}

impl WorthQueryRealizedProjectionScope {
    pub(super) fn record(&mut self, entity_id: EntityId) {
        self.entity_ids.insert(entity_id);
        if let Some(call) = &mut self.call {
            call.insert(entity_id);
        }
    }

    pub(super) fn record_relation(&mut self, from: EntityId, to: EntityId) {
        self.record(from);
        self.record(to);
    }

    /// Begins collecting what one owner call reaches.
    pub(super) fn begin_call(&mut self) {
        self.call = Some(BTreeSet::new());
    }

    /// Everything the call begun last reached, in order.
    pub(super) fn end_call(&mut self) -> Vec<EntityId> {
        self.call.take().unwrap_or_default().into_iter().collect()
    }

    /// Admits identities an earlier run of the same call reached.
    pub(super) fn extend(&mut self, entities: &[EntityId]) {
        self.entity_ids.extend(entities.iter().copied());
    }

    pub(in crate::domain_computation::primary_graph) fn contains(
        &self,
        entity_id: EntityId,
    ) -> bool {
        self.entity_ids.contains(&entity_id)
    }
}
