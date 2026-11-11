//! Receiver-scoped corruption of actual retained recipe data for courts.
use super::super::WorthQueryDocumentHistory;
use super::*;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    #[doc(hidden)]
    pub fn clear_history_discovery_recipes_for_test(
        &mut self,
        step: WorthQueryHistoryNodeId,
    ) -> bool {
        self.selection
            .lock()
            .journal
            .retained_recipe_source_mut(step)
            .is_some_and(|node| node.discovery_recipes.take().is_some())
    }
    #[doc(hidden)]
    pub fn corrupt_history_discovery_recipe_for_test(
        &mut self,
        step: WorthQueryHistoryNodeId,
    ) -> bool {
        let mut selection = self.selection.lock();
        let Some(node) = selection.journal.retained_recipe_source_mut(step) else {
            return false;
        };
        let Some(recipes) = node.discovery_recipes.as_mut().and_then(Arc::get_mut) else {
            return false;
        };
        let Some(entry) = recipes
            .entries
            .values_mut()
            .next()
            .and_then(|slot| Arc::get_mut(&mut slot.entry))
        else {
            return false;
        };
        let Some(request) = Arc::get_mut(&mut entry.request) else {
            return false;
        };
        request.corrupt_seal_for_test();
        true
    }
    #[doc(hidden)]
    pub fn replace_history_discovery_recipes_from_for_test(
        &mut self,
        step: WorthQueryHistoryNodeId,
        source: &WorthQueryDocumentHistory<'_, S, H>,
        source_step: WorthQueryHistoryNodeId,
    ) -> bool {
        let recipes = {
            let mut source = source.selection.lock();
            source
                .journal
                .retained_recipe_source_mut(source_step)
                .and_then(|node| node.discovery_recipes.clone())
        };
        let Some(recipes) = recipes else {
            return false;
        };
        let mut selected = self.selection.lock();
        let Some(node) = selected.journal.retained_recipe_source_mut(step) else {
            return false;
        };
        node.discovery_recipes = Some(recipes);
        true
    }
}
impl<S> HistoryJournal<'_, S> {
    fn retained_recipe_source_mut(
        &mut self,
        step: WorthQueryHistoryNodeId,
    ) -> Option<&mut RetainedHistorySource> {
        if step.document != self.incarnation {
            return None;
        }
        self.retained
            .get_mut(&step.ordinal)
            .filter(|source| source.source.is_some())
    }
}
