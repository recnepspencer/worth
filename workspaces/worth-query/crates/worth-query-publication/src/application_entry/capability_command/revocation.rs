use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityRef, ApplicationCapabilityRequest,
        ApplicationCapabilityRevocationRequest,
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
        WorthQueryApplicationIdempotencyResolution, WorthQueryCapabilityRevocationProgram,
        WorthQueryPrincipalResolutionMode,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::capability_revocation::WorthQueryApplicationCapabilityRevocationDenial;
use super::super::WorthQueryApplicationRequest;
use super::{NativeCommandAction, NativeCommandResult};
use worth_query_execution::publication_boundary::{
    capability_workflow::WorthQueryCapabilityWorkflowIdempotency, program_publication_access,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    /// Retains original funded command meaning and principal association.
    /// Each advancement resolves fresh native authority; preparation never owns the effect.
    /// Capture refusal returns the untouched input, key, and preparation owner.
    pub fn retain_capability_revocation_in_program<
        'command,
        Program,
        PrincipalBinding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Capability,
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
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        key: Key,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        mut prepare: Prepare,
    ) -> Result<
        super::WorthQueryCapabilityCommand<
            'command,
            Schema,
            Program,
            WorthQueryApplicationCapabilityRevocationDenial<PreparationDenial>,
        >,
        (
            Input,
            Key,
            Prepare,
            WorthQueryApplicationCapabilityRevocationDenial<PreparationDenial>,
        ),
    >
    where
        Schema: 'command,
        PreparationDenial: 'command,
        Key: 'command,
        Input: 'command,
        Operation: 'command,
        Capability: 'command,
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
                    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
                >,
            ) -> Result<
                WorthQueryCapabilityRevocationProgram<
                    Schema,
                    Operation,
                    Input,
                    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
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
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + ApplicationCapabilityRevocationRequest<Schema, Capability>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        use WorthQueryApplicationCapabilityRevocationDenial as Denial;
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
                let capability = request
                    .application
                    .installed_schema()
                    .capability(capability, operation)
                    .map_err(Denial::CapabilityInstallation)?;
                let principal_binding = request
                    .application
                    .installed_schema()
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
                Ok((selected, (capability, principal)))
            },
            move |phase, request, selected, (capability, principal), action| {
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

                let mut access = selected
                    .admit_encoded_capability_access(
                        phase,
                        &publication,
                        &principal,
                        &capability,
                        input.clone(),
                        request.scope,
                        None,
                    )
                    .map_err(Denial::Authorization)?;
                access.record_request_identity_work(&publication, workflow.key_work());

                let operation = request
                    .application
                    .installed_schema()
                    .installed_operation(operation)
                    .map_err(Denial::OperationInstallation)?;
                let admission = request
                    .application
                    .authorize_capability_revocation(
                        phase,
                        access,
                        &capability,
                        &operation,
                        preconditions.clone(),
                    )
                    .map_err(Denial::Authorization)?;
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
                    program_action.compare_and_commit_capability_revocation_in_advancement(
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
