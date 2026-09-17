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
        WorthQueryApplicationOperationInvariantProjectionReader,
        WorthQueryElevationApprovalOutcome, WorthQueryInvariantEntityIdentity,
        WorthQueryOperationAuthorizationDenial, WorthQueryOperationProjectionDenial,
        WorthQueryPrincipalResolutionDenial, WorthQueryPrincipalResolutionMode,
        WorthQueryProductBranchAdmissionDenial, WorthQueryRequestedElevation,
    },
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationOperationInstallationDenial, WorthQueryPrincipalBindingInstallationDenial,
};

use super::{workflow_key::workflow_idempotency, WorthQueryApplicationRequest};

#[derive(Debug)]
pub enum WorthQueryApplicationElevationApprovalDenial<DecisionDenial> {
    Program(WorthQueryApplicationCommitDenial),
    ProgramMismatch,
    PrincipalBindingInstallation(WorthQueryPrincipalBindingInstallationDenial),
    PrincipalIdentityEncoding(ApplicationValueEncodeDenial),
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    OperationInstallation(WorthQueryApplicationOperationInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    ApprovalAuthorization(WorthQueryOperationAuthorizationDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Decision(DecisionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
    IdempotencyResolution(WorthQueryApplicationIdempotencyResolutionDenial),
}

#[derive(Debug)]
pub struct WorthQueryApplicationElevationApprovalFailure<DecisionDenial> {
    denial: WorthQueryApplicationElevationApprovalDenial<DecisionDenial>,
    requested: Option<WorthQueryRequestedElevation>,
}

impl<DecisionDenial> WorthQueryApplicationElevationApprovalFailure<DecisionDenial> {
    fn retained(
        denial: WorthQueryApplicationElevationApprovalDenial<DecisionDenial>,
        requested: WorthQueryRequestedElevation,
    ) -> Self {
        Self {
            denial,
            requested: Some(requested),
        }
    }

    fn consumed(denial: WorthQueryApplicationElevationApprovalDenial<DecisionDenial>) -> Self {
        Self {
            denial,
            requested: None,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationElevationApprovalDenial<DecisionDenial>,
        Option<WorthQueryRequestedElevation>,
    ) {
        (self.denial, self.requested)
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    pub fn execute_elevation_approval_in_program<
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
        requested: WorthQueryRequestedElevation,
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
        WorthQueryElevationApprovalOutcome,
        WorthQueryApplicationElevationApprovalFailure<DecisionDenial>,
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
        use WorthQueryApplicationElevationApprovalDenial as Denial;
        use WorthQueryApplicationElevationApprovalFailure as Failure;

        if !std::ptr::eq(program.runtime(), self.application) {
            return Err(Failure::retained(Denial::ProgramMismatch, requested));
        }
        let program_action = match program.admit_program_operation::<Operation>() {
            Ok(admitted) => admitted,
            Err(denial) => return Err(Failure::retained(Denial::Program(denial), requested)),
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
                    requested,
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
                    requested,
                ))
            }
        };
        let selected = match self.application.on_branch(self.branch).select() {
            Ok(selected) => selected,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::ProductSelection(denial),
                    requested,
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
                    requested,
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
                    requested,
                ))
            }
        };
        let access = match selected.admit_capability_access(
            &principal,
            &capability,
            input.clone(),
            self.scope,
        ) {
            Ok(access) => access,
            Err(denial) => return Err(Failure::retained(Denial::Authorization(denial), requested)),
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
                    requested,
                ))
            }
        };
        let mut admission = match self.application.authorize_elevation_approval(
            requested,
            access,
            &operation,
            preconditions.clone(),
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                let original = denial.denial().clone();
                let requested = denial.into_requested();
                if let Ok(replay_access) =
                    selected.admit_capability_access(&principal, &capability, input, self.scope)
                {
                    match self
                        .application
                        .replay_elevation_approval_after_fresh_denial(
                            requested,
                            replay_access,
                            &operation,
                            preconditions,
                            idempotency,
                        ) {
                        Ok(outcome) => return Ok(outcome),
                        Err((Some(denial), requested)) => {
                            return Err(Failure::retained(
                                Denial::IdempotencyResolution(denial),
                                requested,
                            ));
                        }
                        Err((None, requested)) => {
                            return Err(Failure::retained(
                                Denial::ApprovalAuthorization(original),
                                requested,
                            ));
                        }
                    }
                }
                return Err(Failure::retained(
                    Denial::ApprovalAuthorization(original),
                    requested,
                ));
            }
        };
        if let Some(outcome) = self
            .application
            .resolve_admitted_elevation_approval_replay(&mut admission, idempotency)
            .map_err(|(denial, requested)| {
                Failure::retained(Denial::IdempotencyResolution(denial), requested)
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
            .materialize_elevation_approval_program()
            .map_err(|denial| Failure::consumed(Denial::Attempt(denial)))?;
        Ok(program_action.compare_and_commit_elevation_approval(program, idempotency))
    }
}
