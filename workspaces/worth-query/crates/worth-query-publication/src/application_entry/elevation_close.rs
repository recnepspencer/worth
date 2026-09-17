use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityRef, ApplicationCapabilityRequest,
        ApplicationCapabilityWorkflowIdempotency,
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
        WorthQueryApplicationIdempotencyResolutionDenial,
        WorthQueryApplicationInvariantProjectionAuthority,
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryApprovedElevation,
        WorthQueryElevationCloseOutcome, WorthQueryInvariantEntityIdentity,
        WorthQueryOperationAuthorizationDenial, WorthQueryOperationProjectionDenial,
        WorthQueryPrincipalResolutionDenial, WorthQueryPrincipalResolutionMode,
        WorthQueryProductBranchAdmissionDenial,
    },
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationOperationInstallationDenial, WorthQueryPrincipalBindingInstallationDenial,
};

use super::{workflow_key::workflow_idempotency, WorthQueryApplicationRequest};

#[derive(Debug)]
pub enum WorthQueryApplicationElevationCloseDenial<DecisionDenial> {
    Program(WorthQueryApplicationCommitDenial),
    ProgramMismatch,
    PrincipalBindingInstallation(WorthQueryPrincipalBindingInstallationDenial),
    PrincipalIdentityEncoding(ApplicationValueEncodeDenial),
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    OperationInstallation(WorthQueryApplicationOperationInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    CloseAuthorization(WorthQueryOperationAuthorizationDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Decision(DecisionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
    IdempotencyResolution(WorthQueryApplicationIdempotencyResolutionDenial),
}

#[derive(Debug)]
pub struct WorthQueryApplicationElevationCloseFailure<DecisionDenial> {
    denial: WorthQueryApplicationElevationCloseDenial<DecisionDenial>,
    approved: Option<WorthQueryApprovedElevation>,
}

impl<DecisionDenial> WorthQueryApplicationElevationCloseFailure<DecisionDenial> {
    fn retained(
        denial: WorthQueryApplicationElevationCloseDenial<DecisionDenial>,
        approved: WorthQueryApprovedElevation,
    ) -> Self {
        Self {
            denial,
            approved: Some(approved),
        }
    }

    fn consumed(denial: WorthQueryApplicationElevationCloseDenial<DecisionDenial>) -> Self {
        Self {
            denial,
            approved: None,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationElevationCloseDenial<DecisionDenial>,
        Option<WorthQueryApprovedElevation>,
    ) {
        (self.denial, self.approved)
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    pub fn execute_elevation_close_in_program<
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
        approved: WorthQueryApprovedElevation,
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
        WorthQueryElevationCloseOutcome,
        WorthQueryApplicationElevationCloseFailure<DecisionDenial>,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema>
            + ApplicationCapabilityWorkflowIdempotency<Schema, Input, Key>
            + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + Send + Sync + 'static,
    {
        use WorthQueryApplicationElevationCloseDenial as Denial;
        use WorthQueryApplicationElevationCloseFailure as Failure;

        if !std::ptr::eq(program.runtime(), self.application) {
            return Err(Failure::retained(Denial::ProgramMismatch, approved));
        }
        let program_action = match program.admit_program_operation::<Operation>() {
            Ok(admitted) => admitted,
            Err(denial) => return Err(Failure::retained(Denial::Program(denial), approved)),
        };
        let capability = match self
            .application
            .installed_schema()
            .capability(capability, operation)
        {
            Ok(installed) => installed,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::CapabilityInstallation(denial),
                    approved,
                ))
            }
        };
        let principal_binding = match self
            .application
            .installed_schema()
            .principal_binding(principal_binding)
        {
            Ok(installed) => installed,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::PrincipalBindingInstallation(denial),
                    approved,
                ))
            }
        };
        let selected = match self.application.on_branch(self.branch).select() {
            Ok(selected) => selected,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::ProductSelection(denial),
                    approved,
                ))
            }
        };
        let principal = match selected.resolve_authenticated_principal(
            &principal_binding,
            self.principal,
            self.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        ) {
            Ok(principal) => principal,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::PrincipalResolution(denial),
                    approved,
                ))
            }
        };
        let idempotency = match workflow_idempotency::<
            Schema,
            Operation,
            Input,
            Key,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(key, &input, principal.principal_identity())
        {
            Ok(binding) => binding,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::PrincipalIdentityEncoding(denial),
                    approved,
                ))
            }
        };
        let access =
            match selected.admit_capability_access(&principal, &capability, input, self.scope) {
                Ok(access) => access,
                Err(denial) => {
                    return Err(Failure::retained(Denial::Authorization(denial), approved))
                }
            };
        let operation = match self
            .application
            .installed_schema()
            .installed_operation(operation)
        {
            Ok(installed) => installed,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::OperationInstallation(denial),
                    approved,
                ))
            }
        };
        let mut admission = match self.application.authorize_elevation_close(
            approved,
            access,
            &operation,
            preconditions,
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::CloseAuthorization(denial.denial().clone()),
                    denial.into_approved(),
                ))
            }
        };
        if let Some(outcome) = self
            .application
            .resolve_admitted_elevation_close_replay(&mut admission, idempotency)
            .map_err(|(denial, approved)| {
                Failure::retained(Denial::IdempotencyResolution(denial), approved)
            })?
        {
            return Ok(outcome);
        }
        let projected = invariant_projection
            .project_admitted_operation(&admission, project)
            .map_err(|denial| Failure::consumed(Denial::Projection(denial)))?;
        let (decision, projection, _) = projected.into_parts();
        decision.map_err(|denial| Failure::consumed(Denial::Decision(denial)))?;
        let program = self
            .application
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(|denial| Failure::consumed(Denial::Attempt(denial)))?
            .complete_projected_dependencies()
            .map_err(|denial| Failure::consumed(Denial::Attempt(denial)))?
            .materialize_elevation_close_program()
            .map_err(|denial| Failure::consumed(Denial::Attempt(denial)))?;
        Ok(program_action.compare_and_commit_elevation_close(program, idempotency))
    }
}
