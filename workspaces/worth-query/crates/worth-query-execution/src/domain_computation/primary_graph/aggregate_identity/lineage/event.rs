//! Typed endpoints borrowed from an actual retained Native incident or merge step.
use super::WorthQueryAggregateIdentity;
use worth_foundational::facade::AggregateLineageKind;
use worth_relational::facade::{history::CommitId, runtime::AggregateReadLineageEvent};

/// One actual historical event with typed endpoints in the original installation.
pub struct WorthQueryAggregateLineageEvent<'read, Schema, Relation> {
    native: AggregateReadLineageEvent<'read>,
    selected: &'read WorthQueryAggregateIdentity<Schema, Relation>,
}

impl<'read, Schema, Relation> WorthQueryAggregateLineageEvent<'read, Schema, Relation> {
    pub(super) fn new(
        native: AggregateReadLineageEvent<'read>,
        selected: &'read WorthQueryAggregateIdentity<Schema, Relation>,
    ) -> Self {
        Self { native, selected }
    }

    pub fn occurrence(&self) -> CommitId {
        self.native.occurrence()
    }

    pub fn kind(&self) -> AggregateLineageKind {
        self.native.kind()
    }

    pub fn from(&self) -> WorthQueryAggregateIdentity<Schema, Relation> {
        self.selected.related(self.native.from())
    }

    pub fn to(&self) -> WorthQueryAggregateIdentity<Schema, Relation> {
        self.selected.related(self.native.to())
    }
}
