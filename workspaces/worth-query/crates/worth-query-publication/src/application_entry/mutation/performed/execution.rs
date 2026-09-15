use super::*;

impl<'application, 'principal, 'scope, 'key, Schema, Intent>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn execute_performed<Program, Inventory, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationPerformedMutationOutcome<
            'application,
            Schema,
            Intent,
            Program,
            Inventory,
            Root,
        >,
        WorthQueryPerformedMutationExecutionDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Connections: WorthQueryProgramConnectionPlan<Schema, Program, Inventory>,
        Inventory: ApplicationProgramInventoryIdentity,
        Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
        ProgramDemand<Schema, Root>: Send + Sync,
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
        if !std::ptr::eq(application.runtime(), self.request.application)
            || application.installed_program().schema_binding()
                != &self
                    .request
                    .application
                    .installed_schema()
                    .binding_identity()
        {
            return Err(WorthQueryPerformedMutationExecutionDenial::ForeignProgram);
        }
        if !application
            .installed_program()
            .contains_connection_type::<Root>()
        {
            return Err(WorthQueryPerformedMutationExecutionDenial::MissingConnection);
        }
        Root::validate_source(&self.request.intent)
            .map_err(WorthQueryPerformedMutationExecutionDenial::Connection)?;
        match application.installed_program().inventory_posture::<Inventory>() {
            worth_query_installation::facade::WorthQueryInstalledProgramInventoryPosture::Missing => {
                return Err(WorthQueryPerformedMutationExecutionDenial::Preparation(
                    WorthQueryRequiredOutputPreparationDenial::MissingInventory,
                ));
            }
            worth_query_installation::facade::WorthQueryInstalledProgramInventoryPosture::Unavailable { feature } => {
                return Err(WorthQueryPerformedMutationExecutionDenial::Preparation(
                    WorthQueryRequiredOutputPreparationDenial::Unavailable { feature },
                ));
            }
            worth_query_installation::facade::WorthQueryInstalledProgramInventoryPosture::Available => {}
        }
        if !application
            .installed_program()
            .inventory_accepts_root::<Inventory, Root>()
        {
            return Err(WorthQueryPerformedMutationExecutionDenial::Preparation(
                WorthQueryRequiredOutputPreparationDenial::WrongRoot,
            ));
        }
        let query_application = self.request.application;
        let query_principal = self.request.principal;
        let query_scope = self.request.scope;
        let query_branch = self.request.branch;
        let source_intent = self.request.intent.clone();
        let preparation_failure = std::cell::RefCell::new(None);
        let prepared_source = std::cell::RefCell::new(None);
        let outcome = self
            .execute_with_commit(true, |_, program, idempotency| {
                match worth_query_execution::facade::publication_integration::program_execution_port(application).compare_and_commit_required_output_source::<Intent::Binding>(
                    program,
                    idempotency,
                ) {
                    Ok((outcome, prepared)) => {
                        if let Some(prepared) = prepared {
                            prepared_source.replace(Some(prepared));
                        }
                        outcome
                    }
                    Err(failure) => {
                        let receipt = failure.receipt().clone();
                        preparation_failure.replace(Some(failure));
                        WorthQueryApplicationCommitOutcome::Committed(receipt)
                    }
                }
            })
            .map_err(WorthQueryPerformedMutationExecutionDenial::Mutation)?;
        let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
            return Ok(WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
                outcome,
            ));
        };
        let demand = Root::demand_from_committed_source(&source_intent, &result);
        if let Some(failure) = preparation_failure.into_inner() {
            return Ok(
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::DemandExecution(
                        failure.denial().clone(),
                    ),
                },
            );
        }
        let Some((prepared, retained_source)) = prepared_source.into_inner() else {
            return Ok(
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery,
                },
            );
        };
        if let Err(denial) =
            worth_query_execution::facade::publication_integration::program_execution_port(
                application,
            )
            .retain_program_recovery::<Inventory, Root, _>(&receipt, &demand, &prepared)
        {
            application
                .runtime()
                .discard_prepared_required_output_source(prepared);
            return Ok(
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial),
                },
            );
        }
        let retained_read = crate::application_entry::WorthQueryApplicationReadObservation::new(
            std::sync::Arc::clone(&retained_source),
        );
        let request = crate::application_entry::WorthQueryApplicationRequest {
            application: query_application,
            principal: query_principal,
            scope: query_scope,
            branch: query_branch,
        };
        let output_source = match request
            .at(&retained_read)
            .query(demand.source_intent())
            .execute()
        {
            Ok(source) => source.into_output_demand_source(),
            Err(denial) => {
                application
                    .runtime()
                    .discard_prepared_required_output_source(prepared);
                return Ok(
                    WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                        receipt,
                        result,
                        denial: WorthQueryRequiredOutputPreparationDenial::SourceQuery(denial),
                    },
                );
            }
        };
        if let Err(denial) = application
            .runtime()
            .bind_prepared_required_output_source(&prepared, &output_source)
        {
            application
                .runtime()
                .discard_prepared_required_output_source(prepared);
            return Ok(
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial),
                },
            );
        }
        Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(
            WorthQueryPerformedApplicationMutation {
                receipt,
                result,
                application,
                demand,
                prepared,
                retained_source,
                marker: std::marker::PhantomData,
            },
        ))
    }
}
