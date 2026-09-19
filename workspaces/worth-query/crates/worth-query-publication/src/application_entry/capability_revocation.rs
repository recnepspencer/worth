use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityRef, ApplicationCapabilityRequest,
        ApplicationCapabilityRevocationRequest, ApplicationCapabilityWorkflowIdempotency,
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
        WorthQueryApplicationIdempotencyResolution, WorthQueryCapabilityRevocationProgram,
        WorthQueryPrincipalResolutionMode,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use super::{workflow_key::workflow_idempotency, WorthQueryApplicationRequest};

#[derive(Debug)]
pub enum WorthQueryApplicationCapabilityRevocationDenial<PreparationDenial> {
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
    pub fn execute_capability_revocation_in_program<
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
        key: &Key,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        prepare: impl FnOnce(
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
        >,
    ) -> Result<
        WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationCapabilityRevocationDenial<PreparationDenial>,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema>
            + ApplicationCapabilityWorkflowIdempotency<Schema, Input, Key>
            + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + ApplicationCapabilityRevocationRequest<Schema, Capability>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        use WorthQueryApplicationCapabilityRevocationDenial as Denial;

        if !std::ptr::eq(program.runtime(), self.application) {
            return Err(Denial::ProgramMismatch);
        }
        let program_action = program
            .admit_program_operation::<Operation>()
            .map_err(Denial::Program)?;
        let capability = self
            .application
            .installed_schema()
            .capability(capability, operation)
            .map_err(Denial::CapabilityInstallation)?;
        let principal_binding = self
            .application
            .installed_schema()
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
        let access = selected
            .admit_capability_access(&principal, &capability, input, self.scope)
            .map_err(Denial::Authorization)?;
        let operation = self
            .application
            .installed_schema()
            .installed_operation(operation)
            .map_err(Denial::OperationInstallation)?;
        let admission = self
            .application
            .authorize_capability_revocation(access, &capability, &operation, preconditions)
            .map_err(Denial::Authorization)?;
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
        Ok(program_action.compare_and_commit_capability_revocation(prepared, idempotency))
    }
}
