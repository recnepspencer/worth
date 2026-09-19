use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityDelegationRequest, ApplicationCapabilityRef,
        ApplicationCapabilityRequest, ApplicationCapabilityWorkflowIdempotency,
    },
    application_program::ApplicationProgramDefinition,
    application_schema::{
        ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
        ApplicationOperationRef, ApplicationPrincipalBindingRef, ApplicationStructuredValueBinding,
        ApplicationValueEncodeDenial, TypedMutationPreconditions,
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

use super::{workflow_key::workflow_idempotency, WorthQueryApplicationRequest};

#[derive(Debug)]
pub enum WorthQueryApplicationCapabilityDelegationDenial<PreparationDenial> {
    Program(worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitDenial),
    ProgramMismatch,
    PrincipalBindingInstallation(
        worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenial,
    ),
    CapabilityInstallation(
        worth_query_installation::facade::WorthQueryApplicationCapabilityInstallationDenial,
    ),
    OperationInstallation(
        worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenial,
    ),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    PrincipalResolution(
        worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionDenial,
    ),
    PrincipalIdentityEncoding(ApplicationValueEncodeDenial),
    Authorization(
        worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenial,
    ),
    Idempotency(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    IdempotencyIntentDrift,
    Preparation(PreparationDenial),
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    #[allow(clippy::too_many_arguments)]
    pub fn execute_capability_delegation_in_program<
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
        key: &Key,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        prepare: impl FnOnce(
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
        >,
    ) -> Result<
        WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationCapabilityDelegationDenial<PreparationDenial>,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema>
            + ApplicationCapabilityWorkflowIdempotency<Schema, Input, Key>
            + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
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

        if !std::ptr::eq(program.runtime(), self.application) {
            return Err(Denial::ProgramMismatch);
        }
        let program_action = program
            .admit_program_operation::<Operation>()
            .map_err(Denial::Program)?;
        let installed = self.application.installed_schema();
        let command_capability = installed
            .capability(command_capability, operation)
            .map_err(Denial::CapabilityInstallation)?;
        let target_capability = installed
            .capability(target_capability, target_operation)
            .map_err(Denial::CapabilityInstallation)?;
        let principal_binding = installed
            .principal_binding(principal_binding)
            .map_err(Denial::PrincipalBindingInstallation)?;
        let selected = self
            .application
            .on_branch(self.branch)
            .select()
            .map_err(Denial::ProductSelection)?;
        let principal = selected
            .resolve_authenticated_principal(
                &principal_binding,
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(Denial::PrincipalResolution)?;
        let idempotency = workflow_idempotency::<
            Schema,
            Operation,
            Input,
            Key,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(key, &input, principal.principal_identity())
        .map_err(Denial::PrincipalIdentityEncoding)?;
        let replay_input = input.clone();
        let access = selected
            .admit_capability_access(&principal, &command_capability, input, self.scope)
            .map_err(Denial::Authorization)?;
        let operation = installed
            .installed_operation(operation)
            .map_err(Denial::OperationInstallation)?;
        let admission = match self.application.authorize_capability_delegation(
            access,
            &target_capability,
            &operation,
            preconditions.clone(),
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                if let Ok(replay_access) = selected.admit_capability_access(
                    &principal,
                    &command_capability,
                    replay_input,
                    self.scope,
                ) {
                    match self
                        .application
                        .replay_capability_delegation_after_fresh_denial(
                            replay_access,
                            &target_capability,
                            &operation,
                            preconditions,
                            idempotency,
                        ) {
                        Ok(Some(outcome)) => return Ok(outcome),
                        Ok(None) => {}
                        Err(lookup_denial) => return Err(Denial::Idempotency(lookup_denial)),
                    }
                }
                return Err(Denial::Authorization(denial));
            }
        };
        match self
            .application
            .resolve_admitted_application_idempotency(&admission, idempotency)
            .map_err(Denial::Idempotency)?
            .into_resolution()
        {
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
                return Ok(WorthQueryApplicationCommitOutcome::AlreadyCommitted(
                    receipt,
                ));
            }
            WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                return Err(Denial::IdempotencyIntentDrift);
            }
            WorthQueryApplicationIdempotencyResolution::Unseen => {}
        }
        let prepared = prepare(admission).map_err(Denial::Preparation)?;
        Ok(program_action.compare_and_commit_capability_delegation(prepared, idempotency))
    }
}
