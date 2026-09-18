use super::*;

impl<'application, Schema, Intent, Program, Root>
    WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
    Root::Dependents:
        crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<
            'application,
            Schema,
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
    pub fn start_required_outputs<'principal, 'scope>(
        self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            'principal,
            'scope,
            Schema,
        >,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program, Root>,
        WorthQueryRequiredOutputStartFailure<'application, Schema, Intent, Program, Root>,
    > {
        let Self {
            receipt,
            result,
            application,
            demand,
            prepared,
            retained_source,
            mut source_bound,
        } = self;
        if let Err(denial) = application
            .validate_program_root_artifact_source::<Root, Intent::Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
            )
        {
            return Err(WorthQueryRequiredOutputStartFailure {
                performed: Self {
                    receipt,
                    result,
                    application,
                    demand,
                    prepared,
                    retained_source,
                    source_bound,
                },
                denial: WorthQueryRequiredOutputPreparationDenial::Demand(
                    crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(
                        denial,
                    ),
                ),
            });
        }
        let retained_source =
            crate::application_entry::WorthQueryApplicationReadObservation::new(retained_source);
        let source_result = match request
            .at(&retained_source)
            .query(demand.source_intent())
            .execute()
        {
            Ok(source) => source,
            Err(denial) => {
                return Err(WorthQueryRequiredOutputStartFailure {
                    performed: Self {
                        receipt,
                        result,
                        application,
                        demand,
                        prepared,
                        retained_source: std::sync::Arc::clone(&retained_source.retained),
                        source_bound,
                    },
                    denial: WorthQueryRequiredOutputPreparationDenial::SourceQuery(denial),
                })
            }
        };
        if source_result.rows().len() != 1 || source_result.observed_sources().len() != 1 {
            return Err(WorthQueryRequiredOutputStartFailure {
                performed: Self {
                    receipt,
                    result,
                    application,
                    demand,
                    prepared,
                    retained_source: std::sync::Arc::clone(&retained_source.retained),
                    source_bound,
                },
                denial: WorthQueryRequiredOutputPreparationDenial::MissingSource,
            });
        }
        let output_source = source_result.into_output_demand_source();
        if !source_bound {
            if let Err(denial) = application.bind_prepared_program_root_source::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &prepared,
                &output_source,
            ) {
                return Err(WorthQueryRequiredOutputStartFailure {
                    performed: Self {
                        receipt,
                        result,
                        application,
                        demand,
                        prepared,
                        retained_source: std::sync::Arc::clone(&retained_source.retained),
                        source_bound,
                    },
                    denial: WorthQueryRequiredOutputPreparationDenial::Demand(
                        crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
                    ),
                });
            }
            source_bound = true;
        }
        match request
            .demand(demand.clone())
            .controls(controls)
            .start_performed::<Program, Root>(
                application,
                &prepared,
                output_source,
            )
        {
            Ok(required_output) => Ok(WorthQueryStartedRequiredOutputs {
                receipt: receipt.clone(),
                result,
                required_output: crate::application_entry::WorthQueryApplicationProgramOutputHandle::new(
                    application,
                    receipt,
                    retained_source,
                    required_output,
                    demand,
                    controls,
                ),
            }),
            Err(denial) => Err(WorthQueryRequiredOutputStartFailure {
                performed: Self {
                    receipt,
                    result,
                    application,
                    demand,
                    prepared,
                    retained_source: std::sync::Arc::clone(&retained_source.retained),
                    source_bound,
                },
                denial: WorthQueryRequiredOutputPreparationDenial::Demand(denial),
            }),
        }
    }
}
