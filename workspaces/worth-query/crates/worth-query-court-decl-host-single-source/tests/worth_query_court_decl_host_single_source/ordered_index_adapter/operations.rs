//! Operations (`CreateTask`, `RenameTask`, `RescheduleTask`, `DeleteTask`) for ordered-index courts.
use super::queries::{BodyResultBinding, DaymarkContext};
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_mutation_binding,
    worth_query_operation, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, WorthValue)]
pub struct CreateTask {
    pub workspace: String,
    pub key: u64,
    pub title: String,
    pub start_min: i64,
    pub end_min: i64,
}
worth_query_structured_value_binding!(pub CreateTaskInput for CreateTask {
    identity: "ordered-index-adapter.create-task-input.v1",
    capture_input: true
});
worth_query_operation!(pub CreateTaskOp for DaymarkSchema, input CreateTaskInput;
    access {
        read_entities: [Workspace];
        read_fields: [WorkspaceKeyField::at("WorkspaceKey")];
        writes: [
            TaskKeyField::at("TaskKey"),
            TaskTitleField::at("TaskTitle"),
            StartMinField::at("StartMin"),
            EndMinField::at("EndMin")
        ];
        creates: [TaskItem];
        links: [ContainsTask];
    }
);
fn create_scope(input: &CreateTask) -> String {
    input.workspace.clone()
}
worth_query_mutation_binding!(pub CreateTaskBinding for CreateTask, schema DaymarkSchema, context DaymarkContext,
    identity "ordered-index-adapter.create-task-binding.v1", input CreateTaskInput,
    operation CreateTaskOp, result BodyResultBinding,
    decision WorthQueryInvariantMutationTarget<DaymarkSchema, Workspace>,
    handler identity "ordered-index-adapter.create-task-handler.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope WorkspaceKeyField, field WorkspaceKeyField::at("WorkspaceKey"), value create_scope,
);
pub struct CreateTaskHandler;
impl OperationHandler<DaymarkSchema, CreateTaskBinding> for CreateTaskHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[])
    }
    fn decide(
        &self,
        input: &CreateTask,
        reader: &mut DecisionReader<'_, '_, '_, DaymarkSchema, CreateTaskBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<DaymarkSchema, Workspace>,
        WorthQueryOperationBodyDenial,
    > {
        let result = reader
            .resolve_entity(
                WorkspaceKeyField::at("WorkspaceKey"),
                input.workspace.clone(),
            )
            .and_then(|ws| {
                reader.entity(&ws, Workspace::reference())?;
                reader.mutation_target(&ws)
            });
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &CreateTask,
        ws_target: WorthQueryInvariantMutationTarget<DaymarkSchema, Workspace>,
        writer: &mut CandidateWriter<'_, DaymarkSchema, CreateTaskBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let result = (|| {
            let ws = writer.projected_entity(&ws_target)?;
            let key = WorthQueryApplicationEntityKey::new(format!("task-{}", input.key)).unwrap();
            let task = writer.create_entity_in_context(&ws, TaskItem::reference(), key)?;
            writer.initialize_field(&task, TaskKeyField::at("TaskKey"), input.key)?;
            writer.initialize_field(&task, TaskTitleField::at("TaskTitle"), input.title.clone())?;
            writer.initialize_field(&task, StartMinField::at("StartMin"), input.start_min)?;
            writer.initialize_field(&task, EndMinField::at("EndMin"), input.end_min)?;
            writer.link(
                ContainsTask::reference(),
                format!("contains-{}", input.key),
                &ws,
                &task,
            )
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct RenameTask {
    pub key: u64,
    pub title: String,
}
worth_query_structured_value_binding!(pub RenameTaskInput for RenameTask {
    identity: "ordered-index-adapter.rename-task-input.v1",
    capture_input: true
});
worth_query_operation!(pub RenameTaskOp for DaymarkSchema, input RenameTaskInput;
    access { read_fields: [TaskKeyField::at("TaskKey")]; };
    reads [TaskItem]
);
fn rename_scope(input: &RenameTask) -> u64 {
    input.key
}
worth_query_mutation_binding!(pub RenameTaskBinding for RenameTask, schema DaymarkSchema, context DaymarkContext,
    identity "ordered-index-adapter.rename-task-binding.v1", input RenameTaskInput,
    operation RenameTaskOp, result BodyResultBinding,
    decision WorthQueryOperationBodyDecision<DaymarkSchema, TaskItem>,
    handler identity "ordered-index-adapter.rename-task-body.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope TaskKeyField, field TaskKeyField::at("TaskKey"), value rename_scope,
);
pub fn rename_task_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RenameTaskOp, RenameTask> {
    let body = RenameTaskOp::reference()
        .body()
        .assign_input(TaskTitleField::at("TaskTitle"), "title")
        .unwrap()
        .finish()
        .unwrap();
    RenameTaskOp::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
        .with_body(body)
        .unwrap()
}

#[derive(Clone, Debug, WorthValue)]
pub struct RescheduleTask {
    pub key: u64,
    pub start_min: i64,
    pub end_min: i64,
}
worth_query_structured_value_binding!(pub RescheduleTaskInput for RescheduleTask {
    identity: "ordered-index-adapter.reschedule-task-input.v1",
    capture_input: true
});
worth_query_operation!(pub RescheduleTaskOp for DaymarkSchema, input RescheduleTaskInput;
    access { read_fields: [TaskKeyField::at("TaskKey")]; };
    reads [TaskItem]
);
fn reschedule_scope(input: &RescheduleTask) -> u64 {
    input.key
}
worth_query_mutation_binding!(pub RescheduleTaskBinding for RescheduleTask, schema DaymarkSchema, context DaymarkContext,
    identity "ordered-index-adapter.reschedule-task-binding.v1", input RescheduleTaskInput,
    operation RescheduleTaskOp, result BodyResultBinding,
    decision WorthQueryOperationBodyDecision<DaymarkSchema, TaskItem>,
    handler identity "ordered-index-adapter.reschedule-task-body.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope TaskKeyField, field TaskKeyField::at("TaskKey"), value reschedule_scope,
);
pub fn reschedule_task_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RescheduleTaskOp, RescheduleTask> {
    let body = RescheduleTaskOp::reference()
        .body()
        .assign_input(StartMinField::at("StartMin"), "start_min")
        .unwrap()
        .assign_input(EndMinField::at("EndMin"), "end_min")
        .unwrap()
        .finish()
        .unwrap();
    RescheduleTaskOp::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
        .with_body(body)
        .unwrap()
}

#[derive(Clone, Debug, WorthValue)]
pub struct DeleteTask {
    pub workspace: String,
    pub key: u64,
}
worth_query_structured_value_binding!(pub DeleteTaskInput for DeleteTask {
    identity: "ordered-index-adapter.delete-task-input.v1",
    capture_input: true
});
worth_query_operation!(pub DeleteTaskOp for DaymarkSchema, input DeleteTaskInput;
    access {
        read_entities: [Workspace, TaskItem];
        read_fields: [WorkspaceKeyField::at("WorkspaceKey"), TaskKeyField::at("TaskKey")];
        read_relations: [ContainsTask];
        deletes: [TaskItem];
        unlinks: [ContainsTask];
    }
);
fn delete_scope(input: &DeleteTask) -> String {
    input.workspace.clone()
}
pub struct DeleteDecision {
    ws: WorthQueryInvariantMutationTarget<DaymarkSchema, Workspace>,
    task: WorthQueryInvariantMutationTarget<DaymarkSchema, TaskItem>,
}
worth_query_mutation_binding!(pub DeleteTaskBinding for DeleteTask, schema DaymarkSchema, context DaymarkContext,
    identity "ordered-index-adapter.delete-task-binding.v1", input DeleteTaskInput,
    operation DeleteTaskOp, result BodyResultBinding,
    decision DeleteDecision,
    handler identity "ordered-index-adapter.delete-task-handler.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope WorkspaceKeyField, field WorkspaceKeyField::at("WorkspaceKey"), value delete_scope,
);
pub struct DeleteTaskHandler;
impl OperationHandler<DaymarkSchema, DeleteTaskBinding> for DeleteTaskHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[])
    }
    fn decide(
        &self,
        input: &DeleteTask,
        reader: &mut DecisionReader<'_, '_, '_, DaymarkSchema, DeleteTaskBinding>,
    ) -> HandlerResult<DeleteDecision, WorthQueryOperationBodyDenial> {
        let result = (|| {
            let ws = reader.resolve_entity(
                WorkspaceKeyField::at("WorkspaceKey"),
                input.workspace.clone(),
            )?;
            let task = reader.resolve_entity(TaskKeyField::at("TaskKey"), input.key)?;
            reader.entity(&ws, Workspace::reference())?;
            reader.entity(&task, TaskItem::reference())?;
            reader
                .relations_from(ContainsTask::reference(), &ws)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(DeleteDecision {
                ws: reader.mutation_target(&ws)?,
                task: reader.mutation_target(&task)?,
            })
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &DeleteTask,
        decision: DeleteDecision,
        writer: &mut CandidateWriter<'_, DaymarkSchema, DeleteTaskBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let result = (|| {
            let ws = writer.projected_entity(&decision.ws)?;
            let task = writer.projected_entity(&decision.task)?;
            writer.unlink(ContainsTask::reference(), &ws, &task)?;
            writer.delete_entity(TaskItem::reference(), &task)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
