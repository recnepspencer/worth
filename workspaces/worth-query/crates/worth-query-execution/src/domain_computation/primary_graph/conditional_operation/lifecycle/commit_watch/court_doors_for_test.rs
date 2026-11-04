//! Sealed test doors over this module's private members.
use super::*;

impl WatchEntries {
    #[doc(hidden)]
    pub fn entities_for_test(&self) -> &BTreeSet<EntityId> {
        &self.entities
    }
}

impl WorthQueryConditionalCommitWatchSet {
    #[doc(hidden)]
    pub fn entities_for_test(&self) -> &BTreeSet<EntityId> {
        Self::entities(self)
    }
}

impl WorthQueryConditionalCommitWatchSet {
    #[doc(hidden)]
    pub fn retain_entities_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: Option<&Self>,
        incoming: impl Iterator<Item = EntityId>,
        incoming_count: usize,
        maximum_records: Option<usize>,
    ) -> Result<Arc<WatchEntries>, Denial> {
        Self::retain_entities(
            execution,
            predecessor,
            incoming,
            incoming_count,
            maximum_records,
        )
    }
}

impl WorthQueryConditionalCommitWatchSet {
    #[doc(hidden)]
    pub fn new_for_test(entities: Option<Arc<WatchEntries>>, whole_graph: bool) -> Self {
        Self {
            entities,
            whole_graph,
        }
    }
}
