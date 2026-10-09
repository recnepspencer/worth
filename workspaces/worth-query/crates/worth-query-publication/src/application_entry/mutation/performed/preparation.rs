//! Move-only fixed root preparation shared by performed and recovered sources.
use super::*;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
/// Original source custody that can start its fixed demand with a fresh request.
pub struct WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    Binding: ApplicationMutationBinding<Schema>,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
    pub(super) program: std::marker::PhantomData<fn() -> (Program, Binding)>,
    pub(super) demand: ProgramDemand<Schema, Root>,
    pub(super) prepared: WorthQueryPreparedRequiredOutputSource,
    pub(super) retained_source: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
    pub(super) source_bound: bool,
}

impl<Schema, Program, Root, Binding>
    WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    Binding: ApplicationMutationBinding<Schema>,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

impl<Schema, Program, Root, Binding>
    WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    Binding: ApplicationMutationBinding<Schema>,
RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Root::Dependents:
        crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<Schema,
            Program,
            ProgramDemand<Schema, Root>,
        >,
    ProgramDemand<Schema, Root>: Clone,
    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, Root>>,
    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    <<DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
        WorthQueryApplicationProjection<
                Schema,
                <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query,
            > + Clone,
{
    pub fn start_required_outputs<'application, 'principal, 'scope>(
        self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            'principal,
            'scope,
            Schema,
        >,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>,
        (Self, WorthQueryRequiredOutputPreparationDenial),
    > {
        let mut retained = Some(self);
        match application.runtime().with_application_advancement(request.scope, |phase| {
            retained.take().expect("host call retains its performed facts").start_required_outputs_in_advancement(&phase, application, request, controls)
        }) {
            Ok(outcome) => outcome,
            Err(cause) => Err((retained.take().expect("refused request performs no work"), WorthQueryRequiredOutputPreparationDenial::advancement(cause))),
        }
    }

    fn start_required_outputs_in_advancement<'application, 'principal, 'scope>(
        self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            'principal,
            'scope,
            Schema,
        >,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>,
        (Self, WorthQueryRequiredOutputPreparationDenial),
    > {
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err((self, WorthQueryRequiredOutputPreparationDenial::ForeignProgram));
        }
        if let Err(denial) = application.validate_required_program_source::<Root>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            &self.prepared, &self.receipt, &self.retained_source,
            request.principal, request.scope, request.branch,
        ) {
            return Err((self, WorthQueryRequiredOutputPreparationDenial::Demand(crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))));
        }
        let Self {
            receipt,
            program,
            demand,
            prepared,
            retained_source,
            mut source_bound,
        } = self;
        if let Err(denial) = application
            .validate_program_root_artifact_source::<Root, Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
            )
        {
            return Err((Self {
                    receipt,
                            program,
                    demand,
                    prepared,
                    retained_source,
                    source_bound,
                },
                WorthQueryRequiredOutputPreparationDenial::Demand(
                    crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(
                        denial,
                    ),
                ),
            ));
        }
        let retained_source =
            crate::application_entry::WorthQueryApplicationReadObservation::new(retained_source);
        let source_result = match request
            .at(&retained_source)
            .query(demand.source_intent())
            .execute_in_advancement(phase)
        {
            Ok(source) => source,
            Err(denial) => {
                return Err((Self {
                        receipt,
                                    program,
                        demand,
                        prepared,
                        retained_source: std::sync::Arc::clone(&retained_source.retained),
                        source_bound,
                    },
                    WorthQueryRequiredOutputPreparationDenial::SourceQuery(denial),
                ))
            }
        };
        if source_result.rows().len() != 1 || source_result.observed_sources().len() != 1 {
            return Err((Self {
                    receipt,
                            program,
                    demand,
                    prepared,
                    retained_source: std::sync::Arc::clone(&retained_source.retained),
                    source_bound,
                },
                WorthQueryRequiredOutputPreparationDenial::MissingSource,
            ));
        }
        let output_source = source_result.into_output_demand_source();
        if !source_bound {
            if let Err(denial) = application.bind_prepared_program_root_source::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &prepared,
                &output_source,
            ) {
                return Err((Self {
                        receipt,
                                    program,
                        demand,
                        prepared,
                        retained_source: std::sync::Arc::clone(&retained_source.retained),
                        source_bound,
                    },
                    WorthQueryRequiredOutputPreparationDenial::Demand(
                        crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
                    ),
                ));
            }
            source_bound = true;
        }
        match request
            .demand(demand.clone())
            .controls(controls)
            .start_performed::<Program, Root>(
                phase,
                application,
                &prepared,
                output_source,
            )
        {
            Ok(required_output) => Ok(crate::application_entry::WorthQueryApplicationProgramOutputHandle::new(
                receipt, prepared, retained_source, required_output, demand, controls)),
            Err(denial) => Err((Self {
                    receipt,
                            program,
                    demand,
                    prepared,
                    retained_source: std::sync::Arc::clone(&retained_source.retained),
                    source_bound,
                },
                WorthQueryRequiredOutputPreparationDenial::Demand(denial),
            )),
        }
    }
}
