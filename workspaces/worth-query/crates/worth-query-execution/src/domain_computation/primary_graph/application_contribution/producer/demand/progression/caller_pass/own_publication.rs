//! Reobservation may follow only this pass's still-selected own publication.
use super::*;
impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn own_publication_is_selected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: crate::basis::WorthQueryProductBranch,
        publication: Option<&super::super::validated::OwnPublication>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let Some(publication) = publication else {
            return Ok(true);
        };
        #[cfg(test)]
        own_effect_observation::record(Stage::OwnPublicationContinuation);
        self.primary_provider
            .application_attempt_operation_control
            .caller_pass
            .run_interleaving();
        let selected = self
            .on_branch(branch)
            .select_with_request(execution)
            .map_err(|stop| {
                WorthQueryOutputDemandDenial::product_selection(
                    stop,
                    "own publication continuation",
                )
            })?;
        let current = publication
            .0
            .is_selected_at(selected.product().observation());
        #[cfg(test)]
        if !current {
            own_effect_observation::record(Stage::OwnPublicationReplaced);
        }
        Ok(current)
    }
}
