//! Capture full source intent once; every later invocation still admits fresh C.
use super::{retained::*, *};
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;

impl<'runtime, S, H, Payload> WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Configure the original immutable demand for this document's required tree.
    /// Subsequent reads, edits and navigation preserve this typed output custody.
    pub fn with_outputs<P, R>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        demand: Demand<S, R>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<
            'runtime,
            S,
            H,
            Payload,
            WorthQueryHistoryWithOutputs<S, P, R>,
        >,
        (Self, WorthQueryHistoryOutputDenial),
    >
    where
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        <Source<S, R> as ApplicationQueryBinding<S>>::Input:
            ApplicationQueryIntent<S, Binding = Source<S, R>>,
        <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    {
        use WorthQueryHistoryOutputDenial as Denial;
        if !std::ptr::eq(self.owner.application(), request.application)
            || !std::ptr::eq(application.runtime(), request.application)
        {
            return Err((self, Denial::ForeignSource));
        }
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: self.owner.branch(),
        };
        let profile = current
            .query(demand.source_intent())
            .with_admitted_query(phase, |runtime, binding, _, plan| {
                Ok(runtime
                    .capture_history_output_intent(
                        phase,
                        &program_publication_access(),
                        &self.owner,
                        binding,
                        plan,
                    )
                    .map_err(Denial::Source)
                    .and_then(|intent| {
                        application
                            .bind_history_output_profile::<H, R, Demand<S, R>>(
                                phase,
                                &program_publication_access(),
                                &self.owner,
                                intent,
                            )
                            .map_err(Denial::Navigation)
                    }))
            })
            .map_err(Denial::Query)
            .and_then(|result| result);
        let profile = match profile {
            Ok(profile) => profile,
            Err(cause) => return Err((self, cause)),
        };
        let Self {
            owner,
            access,
            payload,
            outputs: _,
        } = self;
        Ok(WorthQueryApplicationHistoryDocument {
            owner,
            access,
            payload,
            outputs: WorthQueryHistoryWithOutputs {
                application: application.clone(),
                profile,
                demand,
                controls,
                slots: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            },
        })
    }
}
