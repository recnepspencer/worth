//! A retained context and its complete recursive currentness footprint travel together.
use crate::domain_computation::primary_graph::application_query::{
    observed_source::{WorthQueryObservedRootSelection, WorthQueryObservedSourceFootprint},
    WorthQueryApplicationOneShotDenialKind as Denial,
};
use std::sync::Arc;
use worth_relational::facade::identity::EntityId;

pub(super) struct ResultSetFootprint<'a> {
    root: EntityId,
    selection: Arc<WorthQueryObservedRootSelection>,
    context_row: Option<&'a WorthQueryObservedSourceFootprint>,
}
impl<'a> ResultSetFootprint<'a> {
    pub(super) fn prepare(
        root: EntityId,
        selection: Arc<WorthQueryObservedRootSelection>,
        retains_context: bool,
        rows: &'a [WorthQueryObservedSourceFootprint],
    ) -> Result<Self, Denial> {
        let context_row = if retains_context {
            let [row] = rows else {
                return Err(Denial::ExpressionContextUnavailable);
            };
            if !row.complete {
                return Err(Denial::ExpressionContextIncomplete);
            }
            Some(row)
        } else {
            None
        };
        Ok(Self {
            root,
            selection,
            context_row,
        })
    }
    pub(super) fn materialize(self) -> WorthQueryObservedSourceFootprint {
        let mut footprint =
            self.context_row
                .cloned()
                .unwrap_or_else(|| WorthQueryObservedSourceFootprint {
                    root: self.root,
                    complete: true,
                    entities: Vec::new(),
                    aspects: Vec::new(),
                    adjacencies: Vec::new(),
                    root_selection: None,
                });
        footprint.root = self.root;
        footprint.root_selection = Some(self.selection);
        footprint
    }
}
