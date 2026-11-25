//! Daymark v2 mutations (`CreateTaskV2`, `UpdateTaskV2`) and handlers.

use super::schema::*;
use super::v2_program::DaymarkV2Schema;
use worth_query_decl::facade::{
    application_operation::*,
    application_schema::{
        fragment::{self, Registration},
        *,
    },
    worth_query_operation, worth_query_operation_creates, worth_query_operation_links,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryInvariantMutationTarget,
    WorthQueryOperationDurableCapability,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskV2Input {
    pub task_key: String,
    pub v2_value: String,
}
worth_query_structured_value_binding!(pub TaskV2InputBinding for TaskV2Input {
    identity: "daymark.v2.task-input.v1"
});
worth_query_structured_value_binding!(pub UnitV2Binding for () {
    identity: "daymark.v2.unit.v1"
});

worth_query_operation!(pub CreateTaskV2Op for DaymarkV2Schema, input TaskV2InputBinding);
worth_query_operation_reads!(CreateTaskV2Op => [ProjectKeyField]);
worth_query_operation_creates!(CreateTaskV2Op => [Task]);
// TaskKeyField and TaskV2ValueField are aliases for FieldSelection<DaymarkCore, Task, TaskCore, String>.
worth_query_operation_writes!(CreateTaskV2Op => [TaskKeyField]);
worth_query_operation_links!(CreateTaskV2Op => [ProjectTasks]);

worth_query_operation!(pub UpdateTaskV2Op for DaymarkV2Schema, input TaskV2InputBinding);
worth_query_operation_reads!(UpdateTaskV2Op => [TaskKeyField]);
worth_query_operation_writes!(UpdateTaskV2Op => [TaskV2ValueField]);

pub struct CreateTaskV2Binding;
impl ApplicationMutationAdmissionBinding<DaymarkV2Schema> for CreateTaskV2Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DaymarkV2Schema> for CreateTaskV2Binding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = TaskV2Input;
    type InputBinding = TaskV2InputBinding;
    type Result = ();
    type ResultBinding = UnitV2Binding;
    type IdempotencyKey = u64;
    type Operation = CreateTaskV2Op;
    type Decision = WorthQueryInvariantMutationTarget<DaymarkV2Schema, Project>;
    type Denial = ();
    type DenialBinding = UnitV2Binding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<ProjectKeyField>;
    type PrincipalBinding = DaymarkPrincipalBinding<DaymarkV2Schema>;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "daymark.v2.create-task.v1";
    const HANDLER_IDENTITY: &'static str = "daymark.v2.create-task-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "daymark.v2.create-task-cmd.v1";
    fn scope_field() -> ProjectKeyField {
        project_key_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkV2Schema,
        Self::PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DaymarkPrincipalBinding::reference()
    }
}

#[derive(Clone, Debug)]
pub struct CreateTaskV2(pub TaskV2Input);
impl ApplicationMutationIntent<DaymarkV2Schema> for CreateTaskV2 {
    type Binding = CreateTaskV2Binding;
    fn input(&self) -> &TaskV2Input {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<ProjectKeyField> {
        ApplicationMutationFieldScope::new(project_key_field(), "p-main".to_owned())
    }
}

pub struct CreateTaskV2Handler;
impl OperationHandler<DaymarkV2Schema, CreateTaskV2Binding> for CreateTaskV2Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        _: &TaskV2Input,
        reader: &mut DecisionReader<'_, '_, '_, DaymarkV2Schema, CreateTaskV2Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<DaymarkV2Schema, Project>, ()> {
        let project = match reader.resolve_entity(project_key_field(), "p-main".to_owned()) {
            Ok(p) => p,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.mutation_target(&project) {
            Ok(t) => HandlerResult::Completed(t),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &TaskV2Input,
        project_target: WorthQueryInvariantMutationTarget<DaymarkV2Schema, Project>,
        writer: &mut CandidateWriter<'_, DaymarkV2Schema, CreateTaskV2Binding>,
    ) -> HandlerResult<(), ()> {
        let project = match writer.projected_entity(&project_target) {
            Ok(p) => p,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        let key = WorthQueryApplicationEntityKey::new(format!("task:{}", input.task_key)).unwrap();
        let task = match writer.create_entity_in_context(
            &project,
            fragment::entity(DaymarkCore, Task),
            key,
        ) {
            Ok(t) => t,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        if let Err(c) = writer.initialize_field(&task, task_key_field(), input.task_key.clone()) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c));
        }
        if let Err(c) =
            writer.initialize_field(&task, task_v2_value_field(), input.v2_value.clone())
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c));
        }
        if let Err(c) = writer.link(
            fragment::relation(DaymarkCore, Project, ProjectTasks),
            format!("project-task:{}", input.task_key),
            &project,
            &task,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c));
        }
        HandlerResult::Completed(())
    }
}

pub struct UpdateTaskV2Binding;
impl ApplicationMutationAdmissionBinding<DaymarkV2Schema> for UpdateTaskV2Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DaymarkV2Schema> for UpdateTaskV2Binding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = TaskV2Input;
    type InputBinding = TaskV2InputBinding;
    type Result = ();
    type ResultBinding = UnitV2Binding;
    type IdempotencyKey = u64;
    type Operation = UpdateTaskV2Op;
    type Decision = WorthQueryInvariantMutationTarget<DaymarkV2Schema, Task>;
    type Denial = ();
    type DenialBinding = UnitV2Binding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<TaskKeyField>;
    type PrincipalBinding = DaymarkPrincipalBinding<DaymarkV2Schema>;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "daymark.v2.update-task.v1";
    const HANDLER_IDENTITY: &'static str = "daymark.v2.update-task-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "daymark.v2.update-task-cmd.v1";
    fn scope_field() -> TaskKeyField {
        task_key_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkV2Schema,
        Self::PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DaymarkPrincipalBinding::reference()
    }
}

#[derive(Clone, Debug)]
pub struct UpdateTaskV2(pub TaskV2Input);
impl ApplicationMutationIntent<DaymarkV2Schema> for UpdateTaskV2 {
    type Binding = UpdateTaskV2Binding;
    fn input(&self) -> &TaskV2Input {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<TaskKeyField> {
        ApplicationMutationFieldScope::new(task_key_field(), self.0.task_key.clone())
    }
}

pub struct UpdateTaskV2Handler;
impl OperationHandler<DaymarkV2Schema, UpdateTaskV2Binding> for UpdateTaskV2Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &TaskV2Input,
        reader: &mut DecisionReader<'_, '_, '_, DaymarkV2Schema, UpdateTaskV2Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<DaymarkV2Schema, Task>, ()> {
        let task = match reader.resolve_entity(task_key_field(), input.task_key.clone()) {
            Ok(t) => t,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Err(cause) = reader.field(&task, task_v2_value_field()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&task) {
            Ok(t) => HandlerResult::Completed(t),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &TaskV2Input,
        target: WorthQueryInvariantMutationTarget<DaymarkV2Schema, Task>,
        writer: &mut CandidateWriter<'_, DaymarkV2Schema, UpdateTaskV2Binding>,
    ) -> HandlerResult<(), ()> {
        let task = match writer.projected_entity(&target) {
            Ok(t) => t,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        match writer.write_field(&task, task_v2_value_field(), input.v2_value.clone()) {
            Ok(()) => HandlerResult::Completed(()),
            Err(c) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        }
    }
}

impl Registration<DaymarkV2Schema> for CreateTaskV2Op {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder
            .operation(
                CreateTaskV2Op::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(CreateTaskV2Op::reference(), 8)
            .operation_read_field(CreateTaskV2Op::reference(), project_key_field())
            .operation_create(
                CreateTaskV2Op::reference(),
                fragment::entity(DaymarkCore, Task),
            )
            .operation_write(CreateTaskV2Op::reference(), task_key_field())
            .operation_write(CreateTaskV2Op::reference(), task_v2_value_field())
            .operation_link(
                CreateTaskV2Op::reference(),
                fragment::relation(DaymarkCore, Project, ProjectTasks),
            ))
    }
}
impl Registration<DaymarkV2Schema> for CreateTaskV2Binding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
impl Registration<DaymarkV2Schema> for UpdateTaskV2Op {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder
            .operation(
                UpdateTaskV2Op::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(UpdateTaskV2Op::reference(), 8)
            .operation_read_field(UpdateTaskV2Op::reference(), task_key_field())
            .operation_read_field(UpdateTaskV2Op::reference(), task_v2_value_field())
            .operation_write(UpdateTaskV2Op::reference(), task_v2_value_field()))
    }
}
impl Registration<DaymarkV2Schema> for UpdateTaskV2Binding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
