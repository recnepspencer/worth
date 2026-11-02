//! Exactly-one Source completion uses the common real permission admission.
use super::*;

impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn read<Binding>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: Binding::Input,
        context_use: &std::cell::Cell<
            crate::domain_computation::primary_graph::handler::DecisionContextUse,
        >,
        native: &mut ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        ObservedQuerySource<Binding::Query, SourceValue<Schema, Binding>>,
        SourceQueryObservationStop,
    >
    where
        Binding: ApplicationQueryBinding<Schema>,
        Binding::Input: ApplicationQueryIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding: ApplicationQueryScopeResolution<Schema, Binding::PrincipalIdentity>,
        SourceValue<Schema, Binding>: WorthQueryApplicationProjection<Schema, Binding::Query>,
    {
        use SourceQueryObservationStop as Stop;
        let application = self.application;
        let selected = self.shared.selected();
        self.with_plan::<Binding, _, Stop>(
            execution,
            intent,
            context_use,
            native,
            |plan, native| {
                let query = plan.query;
                let completed = self.read_admitted::<Binding>(execution, plan, native)?;
                let (rows, disclosure) = completed.read.into_admitted_disclosed().into_parts();
                if rows.is_empty() {
                    return Err(Stop::Missing);
                }
                if rows.len() != 1 {
                    return Err(Stop::Ambiguous);
                }
                let (sources, affinity, receipt) = disclosure.into_parts();
                if sources.len() != 1
                    || !affinity
                        .as_ref()
                        .is_some_and(|affinity| affinity.admits(self.principal, self.request))
                    || !completed
                        .authorization
                        .belongs_to_session(receipt.read_completion().session_identity())
                {
                    return Err(Stop::ForeignClosure);
                }
                let source = sources
                    .into_iter()
                    .next()
                    .expect("exactly one native source");
                let facts = source
                    .admit_fact_materialization(
                        &application.primary_provider.graph.layout,
                        query.name(),
                        super::super::observed_source::SourceFactExpansionControl::in_request(
                            *execution,
                            Some(self.request),
                        ),
                    )
                    .and_then(|prepared| prepared.into_facts())
                    .map_err(Stop::SourceFacts)?;
                let graph = application
                    .runtime
                    .primary_graph()
                    .ok_or(Stop::ForeignClosure)?;
                native.with_runtime_mut(
                    graph,
                    execution,
                    || Stop::ForeignClosure,
                    |runtime| {
                        runtime
                            .read_truth()
                            .positioned_snapshot(selected.application_basis().snapshot_handle())
                            .map_err(|_| Stop::ForeignClosure)
                    },
                )?;
                let closure = SourceQueryReadClosure::retain(
                    application,
                    receipt,
                    completed.authorization,
                    facts,
                )
                .map_err(|stop| Stop::Read(WorthQueryAdmittedOneShotStop::Admission(stop)))?;
                Ok(ObservedQuerySource {
                    value: rows.into_iter().next().expect("exactly one disclosed row"),
                    source,
                    closure,
                })
            },
        )
        .map_err(|stop| self.record_history_refusal(stop))
    }
}
