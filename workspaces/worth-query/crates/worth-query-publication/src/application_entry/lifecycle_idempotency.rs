//! Observe original lifecycle meaning after fresh admission, without a stage owner.
use super::WorthQueryApplicationRequest;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_operation::ApplicationEncodedInput,
    application_program::ApplicationProgramDefinition,
    application_schema::{
        ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
        ApplicationOperationRef, ApplicationPrincipalBindingRef, ApplicationStructuredValueBinding,
        ApplicationValueEncodeDenial, TypedMutationPreconditions,
    },
};
use worth_query_execution::{
    facade::{
        application_contribution::WorthQueryAdvancementDenial,
        application_installation::WorthQueryProgramApplicationRuntime,
        primary_graph::{
            WorthQueryApplicationCommitDenial, WorthQueryApplicationIdempotencyResolution,
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryOperationAuthorizationDenial, WorthQueryPrincipalResolutionDenial,
            WorthQueryPrincipalResolutionMode, WorthQueryProductBranchAdmissionDenial,
        },
    },
    publication_boundary::{
        capability_workflow::WorthQueryCapabilityWorkflowIdempotency, program_publication_access,
    },
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationOperationInstallationDenial, WorthQueryPrincipalBindingInstallationDenial,
};

/// A read refusal carries no lifecycle successor or fabricated original evidence.
#[derive(Debug)]
pub enum WorthQueryCapabilityLifecycleIdempotencyDenial {
    ExecutionRequest(WorthQueryAdvancementDenial),
    Program(WorthQueryApplicationCommitDenial),
    ProgramMismatch,
    PrincipalBindingInstallation(WorthQueryPrincipalBindingInstallationDenial),
    IdentityEncoding(ApplicationValueEncodeDenial),
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    OperationInstallation(WorthQueryApplicationOperationInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    Idempotency(WorthQueryApplicationIdempotencyResolutionDenial),
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    /// Reads the original request/approval/close/review key under current capability
    /// authorization. It runs no handler or lifecycle transition and mints no stage.
    /// Pending original publication retains the ordinary owner's typed disposition.
    pub fn observe_capability_lifecycle_idempotency_in_program<
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
        preconditions: TypedMutationPreconditions<Schema, Operation, Input::Scope>,
    ) -> Result<
        WorthQueryApplicationIdempotencyResolution,
        WorthQueryCapabilityLifecycleIdempotencyDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Key: serde::Serialize,
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + serde::Serialize
            + Clone
            + Send
            + Sync
            + 'static,
    {
        use WorthQueryCapabilityLifecycleIdempotencyDenial as Denial;
        self.application
            .with_application_advancement(self.scope, |phase| {
                if !std::ptr::eq(program.runtime(), self.application) {
                    return Err(Denial::ProgramMismatch);
                }
                let _program_action = program
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
                    .select(&phase)
                    .map_err(Denial::ProductSelection)?;
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &principal_binding,
                        self.principal,
                        self.scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .map_err(Denial::PrincipalResolution)?;
                let input = ApplicationEncodedInput::<Operation::InputBinding>::encode(input)
                    .map_err(Denial::IdentityEncoding)?;
                let workflow = WorthQueryCapabilityWorkflowIdempotency::bind::<
                    Schema,
                    Operation,
                    Key,
                    PrincipalIdentity,
                    PrincipalIdentityBinding,
                >(key, &input, principal.principal_identity())
                .map_err(Denial::IdentityEncoding)?;
                let publication = program_publication_access();
                let mut access = selected
                    .admit_encoded_capability_access(
                        &phase,
                        &publication,
                        &principal,
                        &capability,
                        input,
                        self.scope,
                        None,
                    )
                    .map_err(Denial::Authorization)?;
                access.record_request_identity_work(&publication, workflow.key_work());
                let operation = self
                    .application
                    .installed_schema()
                    .installed_operation(operation)
                    .map_err(Denial::OperationInstallation)?;
                self.application
                    .resolve_capability_lifecycle_idempotency(
                        &phase,
                        access,
                        &operation,
                        preconditions,
                        workflow.binding(),
                    )
                    .map_err(Denial::Idempotency)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
