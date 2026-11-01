//! Fresh Native admission lends each call a frame for the original SDK tree.
use super::{retained::*, *};
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryNavigationBinding,
    application_query::ApplicationQueryIntent,
};

impl<S, H, P, R> WorthQueryHistoryOutputsMode<S, H> for WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
}

impl<S, H, P, R> OutputDriver<S, H> for WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        self.drain(phase)
    }

    fn discard(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        navigation: &WorthQueryDocumentHistoryNavigation<'_, S>,
    ) -> Result<(), WorthQueryHistoryNavigationAdvanceDenial> {
        navigation
            .discard_after_output_cleanup(phase, &program_publication_access(), |key| {
                let slot = self.slot(phase, key)?;
                let result = slot.try_state()?.dispose(phase, &self.application);
                result
            })
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Navigation)?
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Outputs)
    }

    fn prepare<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
    ) -> Result<
        WorthQueryDocumentHistoryNavigation<'runtime, S>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        use WorthQueryHistoryOutputDenial as Denial;
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: document.branch(),
        };
        current
            .query(self.demand.source_intent())
            .with_admitted_query(phase, |runtime, binding, _, plan| {
                Ok(runtime
                    .prepare_document_history_navigation_with_outputs::<H, N, _>(
                        phase,
                        &program_publication_access(),
                        document,
                        history_access,
                        read,
                        expected,
                        intent,
                        key,
                        self.profile.clone(),
                        binding,
                        plan,
                    )
                    .map_err(Denial::from))
            })
            .map_err(Denial::Query)
            .and_then(|result| result)
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Outputs)
    }

    fn advance<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
    ) -> Result<
        Option<WorthQueryHistoryNavigationCompletion>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        use WorthQueryHistoryOutputDenial as Denial;
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: document.branch(),
        };
        let mut original_slot = None;
        let result = current
            .query(self.demand.source_intent())
            .with_admitted_query(phase, |runtime, binding, _, plan| {
                Ok(runtime
                    .advance_document_history_navigation_with_outputs::<H, N, _, Denial>(
                        phase,
                        &program_publication_access(),
                        document,
                        history_access,
                        read,
                        navigation,
                        key,
                        &self.profile,
                        binding,
                        plan,
                        |registered, step| {
                            original_slot = Some(registered);
                            self.progress(phase, &current, registered, step)
                        },
                    )
                    .map_err(Denial::from)
                    .and_then(|result| result))
            })
            .map_err(Denial::Query)
            .and_then(|result| result)
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Outputs);
        if matches!(result, Ok(Some(_))) {
            if let Some(key) = original_slot {
                if let Ok(slot) = self.slot(phase, key) {
                    if let Ok(mut state) = slot.try_state() {
                        // Actual selection survives failed cleanup. The document
                        // retains every remainder for its disposal-only close.
                        let _ = state.dispose(phase, &self.application);
                    }
                }
            }
        }
        result
    }
}
