//! Initial recipe attachment shares the completed source binding's owner.
use super::super::discovery_recipes::*;
use super::super::HistoryModelIdentity;
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::request_recipe::CapturedSourceQueryRequest,
    WorthQueryHistoryNavigationDenial, WorthQueryHistoryOutputReadDenial as ReadDenial,
};
impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn enroll_discovery_recipe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        step: WorthQueryHistoryNodeId,
        model: &HistoryModelIdentity,
        key: HistoryDiscoveryRecipeKey,
        request: Arc<CapturedSourceQueryRequest>,
    ) -> Result<(), ReadDenial> {
        let Some(node) = self
            .retained
            .get_mut(&step.ordinal)
            .filter(|source| source.source.is_some())
        else {
            return Err(ReadDenial::Authority(
                WorthQueryHistoryNavigationDenial::DiscoveryRecipeMissing { step },
            ));
        };
        if let Some(recipes) = &node.discovery_recipes {
            if recipes.step != step || &recipes.model != model {
                return Err(ReadDenial::Authority(
                    WorthQueryHistoryNavigationDenial::DiscoveryRecipeForeignDocument { step },
                ));
            }
            if recipes.entries.contains(&key) {
                recipes.get(execution, step, model, &key)?;
                return Ok(());
            }
        }
        let memory = super::super::allocation::reserve_arc::<HistoryDiscoveryRecipes>(execution)
            .map_err(ReadDenial::Preparation)?;
        let mut entries = HistoryDiscoveryRecipeIndex::default();
        if let Some(recipes) = &node.discovery_recipes {
            for (key, slot) in recipes.entries.iter() {
                execution
                    .run_native_with_checkpoint(|context| {
                        crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                            context, 1,
                        )
                    })
                    .map_err(|cause| ReadDenial::Preparation(cause.into()))?
                    .0
                    .map_err(|cause| ReadDenial::Preparation(cause.into()))?;
                entries
                    .insert(execution, key.clone(), Arc::clone(&slot.entry))
                    .map_err(ReadDenial::Preparation)?;
            }
        }
        let entry_memory = super::super::allocation::reserve_arc::<Entry>(execution)
            .map_err(ReadDenial::Preparation)?;
        entries
            .insert(
                execution,
                key.clone(),
                Arc::new(Entry {
                    step,
                    model: model.clone(),
                    key,
                    request,
                    _memory: entry_memory,
                }),
            )
            .map_err(ReadDenial::Preparation)?;
        node.discovery_recipes = Some(Arc::new(HistoryDiscoveryRecipes {
            step,
            model: model.clone(),
            entries,
            _memory: memory,
        }));
        Ok(())
    }
}
