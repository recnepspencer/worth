//! Alternate-row selector capability & alias-shift mutation contract for watch-entity courts.

use worth_query_decl::facade::{
    application_capability::*,
    application_operation::*,
    application_program::*,
    application_schema::{fragment::Registration, *},
    worth_query_capability, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget, WorthQueryOperationDurableCapability,
};

use super::capabilities::{
    build_capability_contract, text, ProjectCapabilityInputBinding, SharedAccessContext,
    SharedPrincipalBinding,
};
use super::mutations::{ProjectFieldScope, U64ResultBinding, UnitDenialBinding};
use super::schema::{
    ExternalMapping, GrantModeField, Principal, Project, ProjectAliasField, ProjectKeyField,
    ProjectVersionField, SharedProjectSchema,
};

worth_query_capability!(pub WatchProjectByAliasCapability in SharedProjectSchema);
worth_query_operation!(
    pub WatchProjectByAliasOperation for SharedProjectSchema,
    input ProjectCapabilityInputBinding
);
worth_query_operation_reads!(WatchProjectByAliasOperation => [
    ProjectKeyField,
    ProjectVersionField
]);

impl ApplicationCapabilityRequest<SharedProjectSchema, WatchProjectByAliasCapability> for String {
    type Scope = Project;
    type Context = SharedAccessContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<SharedProjectSchema, Project, SharedAccessContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ProjectAliasField::at("ProjectAlias"),
                text(self),
            )
            .unwrap(),
            text("access"),
            text("shared"),
            ApplicationCapabilityRequestContext::new(SharedAccessContext::reference()),
        )
        .field(text("read_and_watch")))
    }
}

impl Registration<SharedProjectSchema> for WatchProjectByAliasOperation {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .operation(
                Self::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(Self::reference(), 64)
            .operation_read_field(Self::reference(), ProjectKeyField::at("ProjectKey"))
            .operation_read_field(Self::reference(), ProjectVersionField::at("ProjectVersion"))
            .operation_read_field(Self::reference(), ProjectAliasField::at("ProjectAlias")))
    }
}

impl Registration<SharedProjectSchema> for WatchProjectByAliasCapability {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability(build_capability_contract(
            WatchProjectByAliasCapability::reference(),
            WatchProjectByAliasOperation::reference(),
            GrantModeField::at("GrantMode"),
            vec![text("read_and_watch")],
        )))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SetProjectAliasInput {
    pub project_key: String,
    pub next_alias: String,
}
worth_query_portable_type!(SetProjectAliasInput => "worth.query.court.shared_access.set_alias_input.v1");
worth_query_structured_value_binding!(pub SetProjectAliasInputBinding for SetProjectAliasInput {
    identity: "worth.query.court.shared_access.set_alias_input_binding.v1"
});
worth_query_operation!(pub SetProjectAlias for SharedProjectSchema, input SetProjectAliasInputBinding);
worth_query_operation_reads!(SetProjectAlias => [ProjectKeyField]);
worth_query_operation_writes!(SetProjectAlias => [ProjectAliasField]);

pub struct SetProjectAliasBinding;
impl ApplicationMutationAdmissionBinding<SharedProjectSchema> for SetProjectAliasBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<SharedProjectSchema> for SetProjectAliasBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = SetProjectAliasInput;
    type InputBinding = SetProjectAliasInputBinding;
    type Result = u64;
    type ResultBinding = U64ResultBinding;
    type IdempotencyKey = u64;
    type Operation = SetProjectAlias;
    type Decision = WorthQueryInvariantMutationTarget<SharedProjectSchema, Project>;
    type Denial = ();
    type DenialBinding = UnitDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ProjectFieldScope;
    type PrincipalBinding = SharedPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.court.shared_access.set_alias.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.court.shared_access.set_alias_handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.court.shared_access.set_alias_cmd.v1";
    fn scope_field() -> ProjectKeyField {
        ProjectKeyField::at("ProjectKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        SharedProjectSchema,
        SharedPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        SharedPrincipalBinding::reference()
    }
}

#[derive(Clone, Debug)]
pub struct SetProjectAliasIntent(pub SetProjectAliasInput);
impl ApplicationMutationIntent<SharedProjectSchema> for SetProjectAliasIntent {
    type Binding = SetProjectAliasBinding;
    fn input(&self) -> &SetProjectAliasInput {
        &self.0
    }
    fn scope_binding(&self) -> ProjectFieldScope {
        ProjectFieldScope::new(
            ProjectKeyField::at("ProjectKey"),
            self.0.project_key.clone(),
        )
    }
}

pub struct AliasCompletion;
impl ApplicationActionDelivery<SharedProjectSchema, SetProjectAliasBinding> for AliasCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.court.shared_access.alias_completion.v1";
}

impl Registration<SharedProjectSchema> for SetProjectAlias {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.operation(
            Self::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(Self::reference(), 64)
        .operation_read_field(Self::reference(), ProjectKeyField::at("ProjectKey"))
        .operation_read_field(Self::reference(), ProjectAliasField::at("ProjectAlias"))
        .operation_write(Self::reference(), ProjectAliasField::at("ProjectAlias")))
    }
}

impl Registration<SharedProjectSchema> for SetProjectAliasBinding {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.application_mutation_binding::<Self>())
    }
}

pub struct SetProjectAliasHandler;
impl OperationHandler<SharedProjectSchema, SetProjectAliasBinding> for SetProjectAliasHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &SetProjectAliasInput,
        reader: &mut DecisionReader<'_, '_, '_, SharedProjectSchema, SetProjectAliasBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<SharedProjectSchema, Project>, ()> {
        let project = match reader
            .resolve_entity(ProjectKeyField::at("ProjectKey"), input.project_key.clone())
        {
            Ok(p) => p,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Err(cause) = reader.field(&project, ProjectAliasField::at("ProjectAlias")) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&project) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &SetProjectAliasInput,
        target: WorthQueryInvariantMutationTarget<SharedProjectSchema, Project>,
        writer: &mut CandidateWriter<'_, SharedProjectSchema, SetProjectAliasBinding>,
    ) -> HandlerResult<u64, ()> {
        let entity = match writer.projected_entity(&target) {
            Ok(e) => e,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        match writer.write_field(
            &entity,
            ProjectAliasField::at("ProjectAlias"),
            input.next_alias.clone(),
        ) {
            Ok(()) => HandlerResult::Completed(1),
            Err(c) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        }
    }
}
