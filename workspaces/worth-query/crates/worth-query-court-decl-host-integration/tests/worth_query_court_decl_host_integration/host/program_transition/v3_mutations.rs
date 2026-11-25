//! Daymark v3 mutations (`CreateTaskV3`, `UpdateTaskV3`) and handlers.

use super::schema::*;
use super::v3_program::DaymarkV3Schema;
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
pub struct TaskV3Input {
    pub task_key: String,
    pub v2_value: String,
    pub priority: String,
}
worth_query_structured_value_binding!(pub TaskV3InputBinding for TaskV3Input {
    identity: "daymark.v3.task-input.v1"
});
worth_query_structured_value_binding!(pub UnitV3Binding for () {
    identity: "daymark.v3.unit.v1"
});

worth_query_operation!(pub CreateTaskV3Op for DaymarkV3Schema, input TaskV3InputBinding);
worth_query_operation_reads!(CreateTaskV3Op => [ProjectKeyField]);
worth_query_operation_creates!(CreateTaskV3Op => [Task]);
worth_query_operation_writes!(CreateTaskV3Op => [TaskKeyField, TaskPriorityField]);
worth_query_operation_links!(CreateTaskV3Op => [ProjectTasks]);

worth_query_operation!(pub UpdateTaskV3Op for DaymarkV3Schema, input TaskV3InputBinding);
worth_query_operation_reads!(UpdateTaskV3Op => [TaskKeyField, TaskPriorityField]);
worth_query_operation_writes!(UpdateTaskV3Op => [TaskV2ValueField, TaskPriorityField]);

pub struct CreateTaskV3Binding;
impl ApplicationMutationAdmissionBinding<DaymarkV3Schema> for CreateTaskV3Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DaymarkV3Schema> for CreateTaskV3Binding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = TaskV3Input;
    type InputBinding = TaskV3InputBinding;
    type Result = ();
    type ResultBinding = UnitV3Binding;
    type IdempotencyKey = u64;
    type Operation = CreateTaskV3Op;
    type Decision = WorthQueryInvariantMutationTarget<DaymarkV3Schema, Project>;
    type Denial = ();
    type DenialBinding = UnitV3Binding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<ProjectKeyField>;
    type PrincipalBinding = DaymarkPrincipalBinding<DaymarkV3Schema>;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "daymark.v3.create-task.v1";
    const HANDLER_IDENTITY: &'static str = "daymark.v3.create-task-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "daymark.v3.create-task-cmd.v1";
    fn scope_field() -> ProjectKeyField {
        project_key_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkV3Schema,
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
pub struct CreateTaskV3(pub TaskV3Input);
impl ApplicationMutationIntent<DaymarkV3Schema> for CreateTaskV3 {
    type Binding = CreateTaskV3Binding;
    fn input(&self) -> &TaskV3Input {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<ProjectKeyField> {
        ApplicationMutationFieldScope::new(project_key_field(), "p-main".to_owned())
    }
}

pub struct CreateTaskV3Handler;
impl OperationHandler<DaymarkV3Schema, CreateTaskV3Binding> for CreateTaskV3Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        _: &TaskV3Input,
        reader: &mut DecisionReader<'_, '_, '_, DaymarkV3Schema, CreateTaskV3Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<DaymarkV3Schema, Project>, ()> {
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
        input: &TaskV3Input,
        project_target: WorthQueryInvariantMutationTarget<DaymarkV3Schema, Project>,
        writer: &mut CandidateWriter<'_, DaymarkV3Schema, CreateTaskV3Binding>,
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
        if let Err(c) =
            writer.initialize_field(&task, task_priority_field(), input.priority.clone())
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

pub struct UpdateTaskV3Binding;
impl ApplicationMutationAdmissionBinding<DaymarkV3Schema> for UpdateTaskV3Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DaymarkV3Schema> for UpdateTaskV3Binding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = TaskV3Input;
    type InputBinding = TaskV3InputBinding;
    type Result = ();
    type ResultBinding = UnitV3Binding;
    type IdempotencyKey = u64;
    type Operation = UpdateTaskV3Op;
    type Decision = WorthQueryInvariantMutationTarget<DaymarkV3Schema, Task>;
    type Denial = ();
    type DenialBinding = UnitV3Binding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<TaskKeyField>;
    type PrincipalBinding = DaymarkPrincipalBinding<DaymarkV3Schema>;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "daymark.v3.update-task.v1";
    const HANDLER_IDENTITY: &'static str = "daymark.v3.update-task-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "daymark.v3.update-task-cmd.v1";
    fn scope_field() -> TaskKeyField {
        task_key_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkV3Schema,
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
pub struct UpdateTaskV3(pub TaskV3Input);
impl ApplicationMutationIntent<DaymarkV3Schema> for UpdateTaskV3 {
    type Binding = UpdateTaskV3Binding;
    fn input(&self) -> &TaskV3Input {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<TaskKeyField> {
        ApplicationMutationFieldScope::new(task_key_field(), self.0.task_key.clone())
    }
}

pub struct UpdateTaskV3Handler;
impl OperationHandler<DaymarkV3Schema, UpdateTaskV3Binding> for UpdateTaskV3Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &TaskV3Input,
        reader: &mut DecisionReader<'_, '_, '_, DaymarkV3Schema, UpdateTaskV3Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<DaymarkV3Schema, Task>, ()> {
        let task = match reader.resolve_entity(task_key_field(), input.task_key.clone()) {
            Ok(t) => t,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Err(cause) = reader.field(&task, task_v2_value_field()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        if let Err(cause) = reader.field(&task, task_priority_field()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&task) {
            Ok(t) => HandlerResult::Completed(t),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &TaskV3Input,
        target: WorthQueryInvariantMutationTarget<DaymarkV3Schema, Task>,
        writer: &mut CandidateWriter<'_, DaymarkV3Schema, UpdateTaskV3Binding>,
    ) -> HandlerResult<(), ()> {
        let task = match writer.projected_entity(&target) {
            Ok(t) => t,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        if let Err(c) = writer.write_field(&task, task_v2_value_field(), input.v2_value.clone()) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c));
        }
        if let Err(c) = writer.write_field(&task, task_priority_field(), input.priority.clone()) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c));
        }
        HandlerResult::Completed(())
    }
}

impl Registration<DaymarkV3Schema> for CreateTaskV3Op {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder
            .operation(
                CreateTaskV3Op::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(CreateTaskV3Op::reference(), 8)
            .operation_read_field(CreateTaskV3Op::reference(), project_key_field())
            .operation_create(
                CreateTaskV3Op::reference(),
                fragment::entity(DaymarkCore, Task),
            )
            .operation_write(CreateTaskV3Op::reference(), task_key_field())
            .operation_write(CreateTaskV3Op::reference(), task_v2_value_field())
            .operation_write(CreateTaskV3Op::reference(), task_priority_field())
            .operation_link(
                CreateTaskV3Op::reference(),
                fragment::relation(DaymarkCore, Project, ProjectTasks),
            ))
    }
}
impl Registration<DaymarkV3Schema> for CreateTaskV3Binding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
impl Registration<DaymarkV3Schema> for UpdateTaskV3Op {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder
            .operation(
                UpdateTaskV3Op::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(UpdateTaskV3Op::reference(), 8)
            .operation_read_field(UpdateTaskV3Op::reference(), task_key_field())
            .operation_read_field(UpdateTaskV3Op::reference(), task_v2_value_field())
            .operation_read_field(UpdateTaskV3Op::reference(), task_priority_field())
            .operation_write(UpdateTaskV3Op::reference(), task_v2_value_field())
            .operation_write(UpdateTaskV3Op::reference(), task_priority_field()))
    }
}
impl Registration<DaymarkV3Schema> for UpdateTaskV3Binding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
