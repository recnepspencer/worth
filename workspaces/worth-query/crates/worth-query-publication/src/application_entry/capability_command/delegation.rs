use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityDelegationRequest, ApplicationCapabilityRef,
        ApplicationCapabilityRequest,
    },
    application_program::ApplicationProgramDefinition,
    application_schema::{
        ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
        ApplicationOperationRef, ApplicationPrincipalBindingRef, ApplicationStructuredValueBinding,
        TypedMutationPreconditions,
    },
};
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationIdempotencyResolution, WorthQueryDelegationActivationProgram,
        WorthQueryPrincipalResolutionMode,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::capability_delegation::WorthQueryApplicationCapabilityDelegationDenial;
use super::super::WorthQueryApplicationRequest;
use super::{NativeCommandAction, NativeCommandResult};
use worth_query_execution::publication_boundary::{
    capability_workflow::WorthQueryCapabilityWorkflowIdempotency, program_publication_access,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    #[allow(clippy::too_many_arguments)]
    /// Retains original funded command meaning and principal association.
    /// Each advancement resolves fresh native authority; preparation never owns the effect.
    /// Capture refusal returns the untouched input, key, and preparation owner.
    pub fn retain_capability_delegation_in_program<
        'command,
        Program,
        PrincipalBinding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
        Key,
        PreparationDenial,
        Prepare,
    >(
        &self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        principal_binding: ApplicationPrincipalBindingRef<
            Schema,
            PrincipalBinding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        command_capability: ApplicationCapabilityRef<Schema, CommandCapability>,
        target_capability: ApplicationCapabilityRef<Schema, TargetCapability>,
        target_operation: ApplicationOperationRef<Schema, TargetOperation, TargetInput>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        key: Key,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        mut prepare: Prepare,
    ) -> Result<
        super::WorthQueryCapabilityCommand<
            'command,
            Schema,
            Program,
            WorthQueryApplicationCapabilityDelegationDenial<PreparationDenial>,
        >,
        (
            Input,
            Key,
            Prepare,
            WorthQueryApplicationCapabilityDelegationDenial<PreparationDenial>,
        ),
    >
    where
        Schema: 'command,
        PreparationDenial: 'command,
        Key: 'command,
        Input: 'command,
        Operation: 'command,
        TargetInput: 'command,
        TargetOperation: 'command,
        TargetCapability: 'command,
        CommandCapability: 'command,
        PrincipalIdentityBinding: 'command,
        PrincipalIdentity: 'command,
        Principal: 'command,
        Mapping: 'command,
        PrincipalBinding: 'command,
        Program: 'command,
        Prepare: FnMut(
                &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
                    '_,
                >,
                WorthQueryAdmittedApplicationOperation<
                    Schema,
                    Operation,
                    Input,
                    <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
                >,
            ) -> Result<
                WorthQueryDelegationActivationProgram<
                    Schema,
                    Operation,
                    Input,
                    <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
                >,
                PreparationDenial,
            > + 'command,
        Program: ApplicationProgramDefinition<Schema>,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: serde::Serialize,
        Key: serde::Serialize,
        TargetOperation: ApplicationOperationMarkerIdentity<Schema>,
        TargetOperation::InputBinding: ApplicationStructuredValueBinding<Value = TargetInput>,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>
            + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        use WorthQueryApplicationCapabilityDelegationDenial as Denial;
        let publication = program_publication_access();
        let capture = self
            .application
            .with_application_advancement(self.scope, |phase| {
                if !std::ptr::eq(program.runtime(), self.application) {
                    return Err(Denial::ProgramMismatch);
                }
                let binding = self
                    .application
                    .installed_schema()
                    .principal_binding(principal_binding.clone())
                    .map_err(Denial::PrincipalBindingInstallation)?;
                let selected = self
                    .application
                    .on_branch(self.branch)
                    .select(&phase)
                    .map_err(Denial::ProductSelection)?;
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &binding,
                        self.principal,
                        self.scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .map_err(Denial::PrincipalResolution)?;
                let encoded = self
                    .application
                    .capture_operation_input_in_advancement::<Operation::InputBinding>(
                        &phase,
                        &publication,
                        self.scope,
                        &input,
                    )
                    .map_err(Denial::InputCapture)?;
                Ok((encoded, principal.command_principal_pin(&publication)))
            })
            .map_err(Denial::ExecutionRequest)
            .and_then(|result| result);
        let (input, original_principal) = match capture {
            Ok(captured) => captured,
            Err(cause) => return Err((input, key, prepare, cause)),
        };
        let runtime = program.clone();
        let program = program.clone();
        let authentication_program = program.clone();
        let authentication_publication = program_publication_access();
        let backend = super::backend::authenticated_backend(
            move |phase, request| {
                if !std::ptr::eq(authentication_program.runtime(), request.application) {
                    return Err(Denial::ProgramMismatch);
                }
                let installed = request.application.installed_schema();
                let command_capability = installed
                    .capability(command_capability, operation)
                    .map_err(Denial::CapabilityInstallation)?;
                let target_capability = installed
                    .capability(target_capability, target_operation)
                    .map_err(Denial::CapabilityInstallation)?;
                let principal_binding = installed
                    .principal_binding(principal_binding.clone())
                    .map_err(Denial::PrincipalBindingInstallation)?;
                let selected = request
                    .application
                    .on_branch(request.branch)
                    .select(phase)
                    .map_err(Denial::ProductSelection)?;
                let principal = selected
                    .resolve_authenticated_principal(
                        phase,
                        &principal_binding,
                        request.principal,
                        request.scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .map_err(Denial::PrincipalResolution)?;
                if !principal
                    .matches_command_principal(&authentication_publication, &original_principal)
                {
                    return Err(Denial::PrincipalMismatch);
                }
                Ok((selected, (command_capability, target_capability, principal)))
            },
            move |phase,
                  request,
                  selected,
                  (command_capability, target_capability, principal),
                  action| {
                let installed = request.application.installed_schema();
                let program_action = program
                    .admit_program_operation::<Operation>()
                    .map_err(Denial::Program)?;

                let workflow = WorthQueryCapabilityWorkflowIdempotency::bind::<
                    Schema,
                    Operation,
                    Key,
                    PrincipalIdentity,
                    PrincipalIdentityBinding,
                >(&key, &input, principal.principal_identity())
                .map_err(Denial::IdentityEncoding)?;
                let idempotency = workflow.binding();

                let replay_input = input.clone();
                let mut access = selected
                    .admit_encoded_capability_access(
                        phase,
                        &publication,
                        &principal,
                        &command_capability,
                        input.clone(),
                        request.scope,
                        None,
                    )
                    .map_err(Denial::Authorization)?;
                access.record_request_identity_work(&publication, workflow.key_work());
                let operation = installed
                    .installed_operation(operation)
                    .map_err(Denial::OperationInstallation)?;
                let admission = match request.application.authorize_capability_delegation(
                    phase,
                    access,
                    &target_capability,
                    &operation,
                    preconditions.clone(),
                ) {
                    Ok(admission) => admission,
                    Err(denial) => {
                        // Receipt-only replay cannot authorize recovery of retained partial custody.
                        if !matches!(
                            action,
                            NativeCommandAction::Execute | NativeCommandAction::Resolve
                        ) {
                            return Err(Denial::Authorization(denial));
                        }
                        if let Ok(mut replay_access) = selected.admit_encoded_capability_access(
                            phase,
                            &publication,
                            &principal,
                            &command_capability,
                            replay_input,
                            request.scope,
                            None,
                        ) {
                            replay_access
                                .record_request_identity_work(&publication, workflow.key_work());

                            match request
                                .application
                                .replay_capability_delegation_after_fresh_denial(
                                    phase,
                                    replay_access,
                                    &target_capability,
                                    &operation,
                                    preconditions.clone(),
                                    idempotency,
                                ) {
                                Ok(Some(outcome)) => {
                                    return Ok(NativeCommandResult::Commit(outcome))
                                }
                                Ok(None) => {}
                                Err(lookup_denial) => {
                                    return Err(Denial::Idempotency(lookup_denial))
                                }
                            }
                        }
                        return Err(Denial::Authorization(denial));
                    }
                };
                if let NativeCommandAction::Recover(recovery) = action {
                    return request
                        .application
                        .recover_admitted_unpublished_source_operation(
                            phase,
                            &publication,
                            recovery,
                            &admission,
                            idempotency,
                        )
                        .map(NativeCommandResult::Recovered)
                        .map_err(Denial::Recovery);
                }
                let read = request
                    .application
                    .resolve_admitted_application_idempotency(phase, &admission, idempotency)
                    .map_err(Denial::Idempotency)?;
                if !matches!(action, NativeCommandAction::Execute) {
                    return Ok(NativeCommandResult::Resolved(read));
                }
                match read.into_resolution() {
                    WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
                        return Ok(NativeCommandResult::Commit(
                            WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt),
                        ))
                    }
                    WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                        return Err(Denial::IdempotencyIntentDrift)
                    }
                    WorthQueryApplicationIdempotencyResolution::Unseen => {}
                }
                let prepared = prepare(phase, admission).map_err(Denial::Preparation)?;
                Ok(NativeCommandResult::Commit(
                    program_action.compare_and_commit_capability_delegation_in_advancement(
                        phase,
                        prepared,
                        idempotency,
                    ),
                ))
            },
        );
        Ok(super::WorthQueryCapabilityCommand::new(
            runtime,
            self.branch,
            backend,
        ))
    }
}
