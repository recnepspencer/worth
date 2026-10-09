use worth_query_declaration::facade::application_operation::ApplicationEncodedInput;
use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityElevationRequest, ApplicationCapabilityRef,
        ApplicationCapabilityRequest,
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
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitDenial,
        WorthQueryApplicationInvariantProjectionAuthority,
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryElevationRequestOutcome,
        WorthQueryInvariantEntityIdentity, WorthQueryOperationAuthorizationDenial,
        WorthQueryOperationProjectionDenial, WorthQueryPrincipalResolutionDenial,
        WorthQueryPrincipalResolutionMode, WorthQueryProductBranchAdmissionDenial,
    },
};
use worth_query_execution::publication_boundary::{
    capability_workflow::WorthQueryCapabilityWorkflowIdempotency, program_publication_access,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationOperationInstallationDenial, WorthQueryPrincipalBindingInstallationDenial,
};

use super::WorthQueryApplicationRequest;

/// Why `execute_elevation_request_in_program` refused. `ProgramMismatch` means the program
/// runtime is not this request's runtime.
#[derive(Debug)]
pub enum WorthQueryApplicationElevationRequestDenial<DecisionDenial> {
    Program(WorthQueryApplicationCommitDenial),
    ProgramMismatch,
    PrincipalBindingInstallation(WorthQueryPrincipalBindingInstallationDenial),
    /// The idempotency key, the input or the principal identity could not be
    /// canonically encoded; the payload names the binding whose value was rejected.
    IdentityEncoding(ApplicationValueEncodeDenial),
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    OperationInstallation(WorthQueryApplicationOperationInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Decision(DecisionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    pub fn execute_elevation_request_in_program<
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
        DecisionDenial,
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
        invariant_projection: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
        project: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<
                Schema,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
        ) -> Result<(), DecisionDenial>,
    ) -> Result<
        WorthQueryElevationRequestOutcome,
        WorthQueryApplicationElevationRequestDenial<DecisionDenial>,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: serde::Serialize,
        Key: serde::Serialize,
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + ApplicationCapabilityElevationRequest<Schema, Operation>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        use WorthQueryApplicationElevationRequestDenial as Denial;

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
        let idempotency = workflow.binding();
        let publication = program_publication_access();
        let mut access = selected
            .admit_encoded_capability_access(
                &publication,
                &principal,
                &capability,
                input,
                self.scope,
            )
            .map_err(Denial::Authorization)?;
        access.record_request_identity_work(&publication, workflow.key_work());
        let operation = self
            .application
            .installed_schema()
            .installed_operation(operation)
            .map_err(Denial::OperationInstallation)?;
        let admission = self
            .application
            .authorize_elevation_request(access, &operation, preconditions)
            .map_err(Denial::Authorization)?;
        let projected = invariant_projection
            .project_admitted_operation(
                &admission,
                project,
                worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(Denial::Projection)?;
        let (decision, projection, _) = projected.into_parts();
        decision.map_err(Denial::Decision)?;
        let program = self
            .application
            .begin_projected_application_read_attempt(
                admission,
                projection,
                worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(Denial::Attempt)?
            .complete_projected_dependencies(
                worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(Denial::Attempt)?
            .materialize_elevation_request_program()
            .map_err(Denial::Attempt)?;
        Ok(program_action.compare_and_commit_elevation_request(program, idempotency))
    }
}
