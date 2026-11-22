//! One project-scoped edit rewrites a task's filter-only notes.
use super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, WorthValue)]
pub struct EditNotes {
    pub project: u64,
    pub task: u64,
    pub notes: String,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug)]
pub enum Denied {
    TaskNotOwned,
}
worth_query_structured_value_binding!(pub InputBinding for EditNotes {identity:"where-filter.edit.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Edited {identity:"where-filter.edit.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denied {identity:"where-filter.edit.denial.v1"});
worth_query_operation!(pub EditTask for Schema, input InputBinding);
worth_query_operation_reads!(EditTask => [Contains]);
fn project(input: &EditNotes) -> u64 {
    input.project
}
worth_query_mutation_binding!(pub Binding for EditNotes, schema Schema,
    identity "where-filter.edit.binding.v1", input InputBinding, operation EditTask,
    result ResultBinding, idempotency u64, identity "where-filter.edit.key.v1",
    decision WorthQueryInvariantMutationTarget<Schema, Task>, denial DenialBinding,
    handler identity "where-filter.edit.handler.v1",
    admission ApplicationConventionalAdmission, program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(), outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKey,
    field ProjectKey::at("ProjectKey"), value project,
);

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &EditNotes,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Task>, Denied> {
        let project = reader.scope().clone();
        let target = (|| -> Result<_, HandlerExecutionDenial> {
            reader.field(&project, ProjectKey::at("ProjectKey"))?;
            let Some(task) = reader.resolve_optional_entity(TaskKey::at("TaskKey"), input.task)?
            else {
                return Ok(None);
            };
            for relation in reader
                .relations_to(Contains::reference(), &task)
                .map_err(HandlerExecutionDenial::new)?
            {
                if relation.from().entity_id() == project.entity_id() {
                    reader.field(&task, Notes::at("Notes"))?;
                    return reader.mutation_target(&task).map(Some);
                }
            }
            Ok(None)
        })();
        match target {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denied::TaskNotOwned),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &EditNotes,
        target: WorthQueryInvariantMutationTarget<Schema, Task>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denied> {
        let effect = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|task| {
                writer
                    .write_field(&task, Notes::at("Notes"), input.notes.clone())
                    .map_err(HandlerExecutionDenial::new)
            });
        match effect {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for EditTask {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        let operation = EditTask::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 32)
            .operation_read_field(operation, ProjectKey::at("ProjectKey"))
            .operation_read_relation(operation, Contains::reference())
            .operation_read_field(operation, TaskKey::at("TaskKey"))
            .operation_read_field(operation, Notes::at("Notes"))
            .operation_write(operation, Notes::at("Notes"))
            .application_mutation_binding::<Binding>())
    }
}
