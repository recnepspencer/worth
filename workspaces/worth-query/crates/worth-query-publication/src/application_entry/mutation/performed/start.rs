use super::*;

impl<'application, Schema, Intent, Program, Inventory, Root>
    WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Inventory, Root>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryProgramConnectionPlan<Schema, Program, Inventory>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
    ProgramDemand<Schema, Root>: Clone + 'static,
    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, Root>>,
    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query: 'static,
    <<DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
        WorthQueryApplicationProjection<
                Schema,
                <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query,
            > + Clone
            + 'static,
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
        WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program, Inventory, Root>,
        WorthQueryRequiredOutputStartFailure<'application, Schema, Intent, Program, Inventory, Root>,
    > {
        let Self {
            receipt,
            result,
            application,
            demand,
            prepared,
            retained_source,
            marker,
        } = self;
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
                        marker,
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
                    marker,
                },
                denial: WorthQueryRequiredOutputPreparationDenial::MissingSource,
            });
        }
        let root = request
            .demand(demand.clone())
            .controls(controls)
            .start_performed::<Program, Inventory>(
                application,
                &prepared,
                source_result.into_output_demand_source(),
            );
        match root {
            Ok(root) => {
                match crate::application_entry::WorthQueryApplicationProgramOutputHandle::new(
                    application,
                    receipt
                        .committed_product_publication()
                        .composite_commit()
                        .clone(),
                    Root::TARGET_FEATURE,
                    Root::target_feature_type(),
                    root,
                    controls,
                ) {
                    Ok(required_output) => Ok(WorthQueryStartedRequiredOutputs {
                        receipt,
                        result,
                        required_output,
                        marker: std::marker::PhantomData,
                    }),
                    Err(denial) => Err(WorthQueryRequiredOutputStartFailure {
                        performed: Self {
                            receipt,
                            result,
                            application,
                            demand,
                            prepared,
                            retained_source: std::sync::Arc::clone(&retained_source.retained),
                            marker,
                        },
                        denial,
                    }),
                }
            }
            Err(denial) => Err(WorthQueryRequiredOutputStartFailure {
                performed: Self {
                    receipt,
                    result,
                    application,
                    demand,
                    prepared,
                    retained_source: std::sync::Arc::clone(&retained_source.retained),
                    marker,
                },
                denial: WorthQueryRequiredOutputPreparationDenial::Demand(denial),
            }),
        }
    }
}
