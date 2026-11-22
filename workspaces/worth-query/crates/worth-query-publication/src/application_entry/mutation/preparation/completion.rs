//! Bind the completed handler program to its original input and workflow authority.
use super::*;
impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    #[inline(never)]
    pub(super) fn finish_executed_candidate<'request>(
        &'request self,
        phase: &AdvancementPhase<'_>,
        executed: handler_execution::ExecutedCandidate<'request, Schema, Intent::Binding>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        CandidatePreparation<'request, Schema, Intent::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let handler_execution::ExecutedCandidate {
            completed,
            identities,
            extension,
            work,
            workflow_authority,
        } = executed;
        let (mut program, result) = completed.into_parts();
        program = self.bind_executed_candidate_program(
            phase,
            program,
            &identities,
            workflow_authority.as_ref(),
            allocation_policy,
        )?;
        let program = Self::prepare_candidate_signed_creation(
            phase,
            self.request.application,
            program,
            &identities,
        )?;
        Ok(CandidatePreparation::Prepared(PreparedCandidate {
            program,
            result,
            identities,
            principal: self.request.principal,
            extension,
            decision_work: work,
        }))
    }

    #[inline(never)]
    fn bind_executed_candidate_program(
        &self,
        phase: &AdvancementPhase<'_>,
        mut program: worth_query_execution::facade::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        identities: &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
        workflow_authority: Option<
            &worth_query_execution::publication_boundary::workflow_advance::WorkflowOperationAuthority,
        >,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let publication_access =
            worth_query_execution::publication_boundary::program_publication_access();
        program = program.with_producer_round_limits(
            &publication_access,
            self.request.producer_round_limits.per_route(),
            self.request.producer_round_limits.per_attempt(),
        );
        if program.requires_original_source_capture(&publication_access) {
            program = program
                .retain_original_mutation_input::<Intent::Binding>(
                    &publication_access,
                    phase,
                    self.request.application,
                    identities.encoded_input(),
                )
                .map_err(|cause| {
                    WorthQueryApplicationRequestMutationDenial::Handler(
                        MutationHandlerExecutionDenial::Attempt(cause),
                    )
                })?;
        }
        if self.workflow_birth_identity.is_some() {
            let authority = self
                .workflow_birth_authority
                .as_ref()
                .and_then(|slot| slot.take())
                .ok_or(WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent)?;
            program = program
                .bind_workflow_birth_authority(phase, authority, allocation_policy)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        if self.workflow_succession_identity.is_some() {
            let authority = self
                .workflow_succession_authority
                .as_ref()
                .and_then(|slot| slot.take())
                .ok_or(WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent)?;
            program = program
                .bind_workflow_succession_authority(phase, authority, allocation_policy)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        if let Some(authority) = workflow_authority {
            program = program
                .bind_workflow_operation_authority(phase, authority, allocation_policy)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        Ok(program)
    }

    #[inline(never)]
    fn prepare_candidate_signed_creation(
        phase: &AdvancementPhase<'_>,
        application: &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        program: worth_query_execution::facade::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        identities: &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        worth_query_execution::publication_boundary::signed_creation::WorthQuerySignedCreationAdapter::prepare::<Schema, Intent::Binding>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            phase,
            application,
            program,
            identities,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::SignedCreation)
    }
}
