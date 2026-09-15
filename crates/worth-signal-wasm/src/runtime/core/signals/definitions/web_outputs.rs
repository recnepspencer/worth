use worth_signal::facade::NodeId;

use crate::boundary::errors::WorthSignalJsError;
use crate::recipe::model::RecipeSpec;

use super::super::super::state::WebSignalKind;
use super::super::super::RuntimeCore;

impl RuntimeCore {
    pub fn define_web_output(
        &mut self,
        id: String,
        spec: RecipeSpec,
    ) -> Result<(), WorthSignalJsError> {
        self.define_recipe(spec)?;
        self.web_signals.insert(id, WebSignalKind::Output);
        Ok(())
    }

    pub(crate) fn mark_worker_public_outputs(
        &mut self,
        output_ids: Vec<String>,
    ) -> Result<(), WorthSignalJsError> {
        for output_id in output_ids {
            if !self.catalog.contains_key(&output_id) {
                return Err(WorthSignalJsError::invalid_input(format!(
                    "worker public output `{output_id}` is not published"
                )));
            }
            if !self.lock_store()?.recipes.contains_key(&output_id) {
                return Err(WorthSignalJsError::invalid_input(format!(
                    "worker public output `{output_id}` must be a recipe"
                )));
            }
            self.web_signals.insert(output_id, WebSignalKind::Output);
        }
        Ok(())
    }

    /// Nodes of every published output, in id order.
    ///
    /// Published outputs are standing demand: the host reads them after each
    /// commit, so a committing transaction settles the ones its change
    /// reaches (`SignalTransaction::evaluate_demand`). Otherwise the committed
    /// truth digest would exclude a value the first post-commit read then
    /// computes, and delivery would carry a different truth than the commit.
    /// Cost: O(published outputs) per transaction.
    pub(crate) fn standing_demand_nodes(&self) -> Vec<NodeId> {
        self.web_signals
            .iter()
            .filter(|(_, kind)| matches!(kind, WebSignalKind::Output))
            .filter_map(|(id, _)| self.catalog.get(id).map(|entry| entry.node))
            .collect()
    }

    pub(crate) fn is_web_output_signal(&self, id: &str) -> bool {
        matches!(self.web_signals.get(id), Some(WebSignalKind::Output))
    }

    #[cfg(test)]
    pub fn web_signal_kind(&self, id: &str) -> Option<WebSignalKind> {
        self.web_signals.get(id).copied()
    }
}
