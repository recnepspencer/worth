//! Owner-only mutation preparation; all lent read capability ends here.
use super::super::DecisionReader;
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn prepare_mutation_handler_plan<
        'advance: 'authority,
        'authority,
        Binding,
    >(
        &self,
        phase: &'advance WorthQueryAdvancementPhase<'advance>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        on_contact: impl FnOnce(),
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, 'authority>,
    ) -> Result<
        PreparedMutationHandler<'advance, 'authority, Schema, Binding>,
        WorthQueryMutationHandlerExecutionReport<
            WorthQueryCompletedMutationCandidate<Schema, Binding>,
            Binding::Denial,
        >,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.prepare_mutation_handler_plan_with_source_access::<Binding>(
            phase,
            identities,
            principal_identity,
            None,
            admission,
            on_contact,
            allocation_policy,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn prepare_mutation_handler_plan_with_source_access<
        'advance: 'authority,
        'authority,
        Binding,
    >(
        &self,
        phase: &'advance WorthQueryAdvancementPhase<'advance>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        source_query: Option<
            &crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<
                '_,
                Schema,
            >,
        >,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        on_contact: impl FnOnce(),
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, 'authority>,
    ) -> Result<
        PreparedMutationHandler<'advance, 'authority, Schema, Binding>,
        WorthQueryMutationHandlerExecutionReport<
            WorthQueryCompletedMutationCandidate<Schema, Binding>,
            Binding::Denial,
        >,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let execution = match phase.execution_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return Err(WorthQueryMutationHandlerExecutionReport::new(
                    Err(MutationHandlerExecutionDenial::ForeignAdvancementPhase(
                        cause,
                    )),
                    WorthQueryMutationHandlerWork::NotStarted,
                ))
            }
        };
        if admission
            .governed_input_identity()
            .is_some_and(|admitted| admitted != identities.input_identity())
        {
            return Err(WorthQueryMutationHandlerExecutionReport::new(
                Err(MutationHandlerExecutionDenial::InputNotAdmitted),
                WorthQueryMutationHandlerWork::NotStarted,
            ));
        }
        let handler = match self.mutation_handler_for_attempt::<Binding>() {
            Ok(installed) => installed,
            Err(denial) => {
                return Err(WorthQueryMutationHandlerExecutionReport::new(
                    Err(denial),
                    WorthQueryMutationHandlerWork::NotStarted,
                ))
            }
        };

        let input = identities.mutation_input();
        let request = admission.publication_request();
        let allocation_control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(
            request,
            allocation_policy,
        );
        let operation_scope_binding = admission.operation_scope_binding().clone();
        let context_use = std::cell::Cell::new(
            crate::domain_computation::primary_graph::handler::DecisionContextUse::default(),
        );
        let interruption = std::cell::Cell::new(None);
        let mut handler_contacted = false;
        let phase_request = match phase.execution_request_for(&self.product_runtime) {
            Ok(request) => request,
            Err(cause) => {
                return Err(WorthQueryMutationHandlerExecutionReport::new(
                    Err(MutationHandlerExecutionDenial::ForeignAdvancementPhase(
                        cause,
                    )),
                    WorthQueryMutationHandlerWork::NotStarted,
                ))
            }
        };
        if let Err(denial) = self.validate_projected_source_facts(&phase_request, &admission) {
            return Err(WorthQueryMutationHandlerExecutionReport::new(
                Err(MutationHandlerExecutionDenial::Attempt(denial)),
                WorthQueryMutationHandlerWork::NotStarted,
            ));
        }
        let projected = {
            self.mutation_projection
                .project_admitted_operation_with_source_access(
                    &phase_request,
                    &admission,
                    |reader, scope| {
                        let mut decision_reader = DecisionReader::<Schema, Binding>::new(
                            reader,
                            scope,
                            principal_identity,
                            &operation_scope_binding,
                            identities,
                            execution,
                            &context_use,
                            &interruption,
                            crate::domain_computation::primary_graph::handler::ConditionalDefinitionAdmission::new(
                                self, admission.graph_work().mutation_product(),
                            ),
                        );
                        on_contact();
                        handler_contacted = true;
                        handler.prepare_decision(input, &mut decision_reader)
                    },
                    allocation_control.policy(),
                    source_query,
                    Some(&context_use),
                )
        };
        let projected = match projected {
            Ok(projected) => projected,
            Err(denial) => {
                let work = denial
                    .projection_work()
                    .map_or(WorthQueryMutationHandlerWork::NotStarted, |work| {
                        WorthQueryMutationHandlerWork::captured(work, handler_contacted)
                    });
                return Err(WorthQueryMutationHandlerExecutionReport::new(
                    Err(MutationHandlerExecutionDenial::Projection(denial)),
                    work,
                ));
            }
        };
        let (plan, projection, work) = projected.into_parts();
        // A decision can swallow its poll result, but cannot turn an observed
        // interruption into prepared compute, a candidate, or reusable input.
        if let Some(interruption) = interruption.get() {
            return Err(WorthQueryMutationHandlerExecutionReport::new(
                Ok(match interruption {
                    super::super::HandlerInterruption::Cancelled => HandlerResult::Cancelled,
                    super::super::HandlerInterruption::DeadlineExceeded => {
                        HandlerResult::DeadlineExceeded
                    }
                }),
                WorthQueryMutationHandlerWork::captured(work, handler_contacted),
            ));
        }
        let context_use = if plan.has_compute() {
            context_use.get().managed_computation()
        } else {
            context_use.get()
        };
        Ok(PreparedMutationHandler::new(
            plan,
            execution,
            projection,
            move |runtime, identities, decision, projection| {
                let outcome = (|| {
                    let decision = match decision {
                        HandlerResult::Completed(decision) => decision,
                        HandlerResult::DomainDenied(denial) => {
                            return Ok(HandlerResult::DomainDenied(denial));
                        }
                        HandlerResult::ExecutionDenied(denial) => {
                            return Err(MutationHandlerExecutionDenial::Handler(denial));
                        }
                        HandlerResult::Cancelled => return Ok(HandlerResult::Cancelled),
                        HandlerResult::DeadlineExceeded => {
                            return Ok(HandlerResult::DeadlineExceeded)
                        }
                    };
                    let mut admission = admission;
                    admission.record_decision_context_use(context_use);
                    let reads = runtime
                        .begin_projected_application_read_attempt_with_request(
                            &phase_request,
                            admission,
                            projection,
                            allocation_control.policy(),
                        )
                        .map_err(MutationHandlerExecutionDenial::Attempt)?
                        .complete_projected_dependencies_with_request(
                            &phase_request,
                            allocation_control.policy(),
                        )
                        .map_err(MutationHandlerExecutionDenial::Attempt)?;
                    super::build_handler_candidate(
                        reads,
                        handler.as_ref(),
                        identities,
                        decision,
                        &phase_request,
                        allocation_control.policy(),
                    )
                })();
                WorthQueryMutationHandlerExecutionReport::new(
                    outcome,
                    WorthQueryMutationHandlerWork::captured(work, handler_contacted),
                )
            },
        ))
    }
}
