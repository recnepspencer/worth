//! Owned historical inspection from one actual Native lineage acquisition.
use super::WorthQueryAggregateIdentity;
use worth_relational::facade::{
    history::CommitId, identity::VersionId, runtime::PinnedAggregateLineageRead,
};
mod event;
pub use event::WorthQueryAggregateLineageEvent;

/// Historical values read under privileged projection authority at one snapshot.
/// This body grants no read, history-disclosure, mutation or publication authority.
pub struct WorthQueryAggregateLineage<Schema, Relation> {
    native: PinnedAggregateLineageRead,
    selected: WorthQueryAggregateIdentity<Schema, Relation>,
    version: VersionId,
}

impl<Schema, Relation> WorthQueryAggregateLineage<Schema, Relation> {
    pub(in crate::domain_computation::primary_graph) fn new(
        native: PinnedAggregateLineageRead,
        selected: WorthQueryAggregateIdentity<Schema, Relation>,
        version: VersionId,
    ) -> Self {
        Self {
            native,
            selected,
            version,
        }
    }

    pub fn identity(&self) -> &WorthQueryAggregateIdentity<Schema, Relation> {
        &self.selected
    }

    /// Describes the selected snapshot; it cannot rebind this retained body.
    pub fn version(&self) -> VersionId {
        self.version
    }

    pub fn retired_at(&self) -> Option<CommitId> {
        self.native.retired_at()
    }

    /// Follows actual merge continuations only. Split children never advance it.
    /// None is a real retired terminal, not an identity missing from the source.
    pub fn latest_identity(&self) -> Option<WorthQueryAggregateIdentity<Schema, Relation>> {
        self.native
            .latest_identity()
            .map(|id| self.selected.related(id))
    }

    /// Stored events incident to the queried identity, in Native canonical order.
    pub fn events(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryAggregateLineageEvent<'_, Schema, Relation>> {
        self.native
            .lineage()
            .map(|native| WorthQueryAggregateLineageEvent::new(native, &self.selected))
    }

    /// Actual ordered merge path from the queried identity to the selected terminal.
    pub fn resolution_steps(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryAggregateLineageEvent<'_, Schema, Relation>> {
        self.native
            .resolution_steps()
            .map(|native| WorthQueryAggregateLineageEvent::new(native, &self.selected))
    }
}

impl<Schema, Relation> std::fmt::Debug for WorthQueryAggregateLineage<Schema, Relation> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryAggregateLineage")
            .field("identity", &self.selected)
            .field("version", &self.version)
            .field("native", &self.native)
            .finish()
    }
}
