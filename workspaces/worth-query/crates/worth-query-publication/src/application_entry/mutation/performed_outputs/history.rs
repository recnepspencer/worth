//! Fresh permission and refusal-preserving delivery of the same original tree.
use super::*;
impl<Schema, Program, Root>
    WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    RootDemand<Schema, Root>: Clone,
    Value<Schema, RootDemand<Schema, Root>>:
        WorthQueryApplicationProjection<Schema, Query<Schema, RootDemand<Schema, Root>>> + Clone,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, RootDemand<Schema, Root>>,
        >,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Root::Dependents:
        ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
{
    pub(in crate::application_entry) fn authorize_history_in_advancement(
        &self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        history: super::super::program_output_continuation::ProgramOutputHistoryFrame<'_, '_, '_>,
    ) -> Result<(), crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        use crate::application_entry::WorthQueryRequiredOutputPreparationDenial as Denial;
        // The real Native root checks C before any cached tree outcome is used.
        self.root.as_ref().ok_or(Denial::Closed)?
            .authorize_in_history(phase, application, request, history).map_err(Denial::Demand)?;
        if let Some(continuation) = &self.continuation {
            continuation.authorize_history(phase, application, request, history)?;
        }
        Ok(())
    }

    pub(in crate::application_entry) fn advance_history_in_advancement(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        history: super::super::program_output_continuation::ProgramOutputHistoryFrame<'_, '_, '_>,
    ) -> Result<WorthQueryApplicationProgramOutputProgress<Query<Schema, RootDemand<Schema, Root>>>, crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        use crate::application_entry::{WorthQueryApplicationOutputDemandDenial as DemandDenial, WorthQueryRequiredOutputPreparationDenial as Denial};
        self.authorize_history_in_advancement(phase, application, request, history)?;
        let progress = match self.pending_history_delivery.take() {
            Some(settlement) => WorthQueryApplicationProgramOutputProgress::Settled(settlement),
            None => self.advance_tree(phase, application, request, Some(history))?,
        };
        match application.confirm_history_output_delivery(phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            history.authority, request.scope, progress,
        ) {
            Ok(progress) => Ok(progress),
            Err((progress, cause)) => {
                // Completion is real even if delivery lost its C authority.
                // Keep the exact finished tree rather than advancing it again.
                if let WorthQueryApplicationProgramOutputProgress::Settled(settlement) = progress {
                    self.pending_history_delivery = Some(settlement);
                }
                Err(Denial::Demand(DemandDenial::History(
                    worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputAdvanceDenial::Authority(cause),
                )))
            }
        }
    }
}
