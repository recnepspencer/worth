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
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
        WorthQueryMandatoryReview, WorthQueryMandatoryReviewOutcome,
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
pub enum WorthQueryApplicationMandatoryReviewDenial<DecisionDenial> {
    Program(WorthQueryApplicationCommitDenial),
    ProgramMismatch,
    PrincipalBindingInstallation(WorthQueryPrincipalBindingInstallationDenial),
    PrincipalIdentityEncoding(ApplicationValueEncodeDenial),
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    OperationInstallation(WorthQueryApplicationOperationInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    ReviewAuthorization(WorthQueryOperationAuthorizationDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Decision(DecisionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
    IdempotencyResolution(WorthQueryApplicationIdempotencyResolutionDenial),
}

#[derive(Debug)]
pub struct WorthQueryApplicationMandatoryReviewFailure<DecisionDenial> {
    denial: WorthQueryApplicationMandatoryReviewDenial<DecisionDenial>,
    mandatory: Option<WorthQueryMandatoryReview>,
}

impl<DecisionDenial> WorthQueryApplicationMandatoryReviewFailure<DecisionDenial> {
    fn retained(
        denial: WorthQueryApplicationMandatoryReviewDenial<DecisionDenial>,
        mandatory: WorthQueryMandatoryReview,
    ) -> Self {
        Self {
            denial,
            mandatory: Some(mandatory),
        }
    }

    fn consumed(denial: WorthQueryApplicationMandatoryReviewDenial<DecisionDenial>) -> Self {
        Self {
            denial,
            mandatory: None,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationMandatoryReviewDenial<DecisionDenial>,
        Option<WorthQueryMandatoryReview>,
    ) {
        (self.denial, self.mandatory)
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    pub fn execute_mandatory_review_in_program<
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
        mandatory: WorthQueryMandatoryReview,
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
        WorthQueryMandatoryReviewOutcome,
        WorthQueryApplicationMandatoryReviewFailure<DecisionDenial>,
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
        use WorthQueryApplicationMandatoryReviewDenial as Denial;
        use WorthQueryApplicationMandatoryReviewFailure as Failure;

        if !std::ptr::eq(program.runtime(), self.application) {
            return Err(Failure::retained(Denial::ProgramMismatch, mandatory));
        }
        let program_action = match program.admit_program_operation::<Operation>() {
            Ok(admitted) => admitted,
            Err(denial) => return Err(Failure::retained(Denial::Program(denial), mandatory)),
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
                    mandatory,
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
                    mandatory,
                ))
            }
        };
        let selected = match self.application.on_branch(self.branch).select() {
            Ok(selected) => selected,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::ProductSelection(denial),
                    mandatory,
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
                    mandatory,
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
                    mandatory,
                ))
            }
        };
        let access =
            match selected.admit_capability_access(&principal, &capability, input, self.scope) {
                Ok(access) => access,
                Err(denial) => {
                    return Err(Failure::retained(Denial::Authorization(denial), mandatory))
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
                    mandatory,
                ))
            }
        };
        let mut admission = match self.application.authorize_mandatory_review(
            mandatory,
            access,
            &operation,
            preconditions,
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                return Err(Failure::retained(
                    Denial::ReviewAuthorization(denial.denial().clone()),
                    denial.into_mandatory_review(),
                ))
            }
        };
        if let Some(outcome) = self
            .application
            .resolve_admitted_mandatory_review_replay(&mut admission, idempotency)
            .map_err(|(denial, mandatory)| {
                Failure::retained(Denial::IdempotencyResolution(denial), mandatory)
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
            .materialize_mandatory_review_program()
            .map_err(|denial| Failure::consumed(Denial::Attempt(denial)))?;
        Ok(program_action.compare_and_commit_mandatory_review(program, idempotency))
    }
}
