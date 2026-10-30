//! Ordinary domain creation issues the model and its initial target occurrence.
use super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct CreateModel {
    pub name: String,
    pub key: u64,
    pub days: u64,
}
#[derive(Debug)]
pub struct Created;
#[derive(Debug)]
pub enum Denial {
    MissingRegistry,
}
worth_query_structured_value_binding!(pub InputBinding for CreateModel { identity:"proposal-application.create.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Created { identity:"proposal-application.create.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity:"proposal-application.create.denial.v1" });
worth_query_operation!(pub Create for Schema, input InputBinding);
worth_query_operation_reads!(Create => [RegistryName]);
worth_query_operation_writes!(Create => [ModelName, TargetName, Retention]);
worth_query_operation_creates!(Create => [Model, Target]);
worth_query_operation_links!(Create => [ModelTarget]);
fn registry(_: &CreateModel) -> String {
    "models".to_owned()
}
worth_query_mutation_binding!(pub Binding for CreateModel, schema Schema,
    identity "proposal-application.create.binding.v1", input InputBinding, operation Create,
    result ResultBinding, idempotency u64, identity "proposal-application.create.key.v1",
    decision WorthQueryInvariantMutationTarget<Schema, Registry>, denial DenialBinding,
    handler identity "proposal-application.create.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Registry, RegistryFacts, RegistryName, String, ReadOnly, NoApplicationUnit,
    field RegistryName::reference(), value registry,
);

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &CreateModel,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Registry>, Denial> {
        let observed = (|| -> Result<_, HandlerExecutionDenial> {
            let registry = reader.resolve_entity(RegistryName::reference(), "models".to_owned())?;
            reader.field(&registry, RegistryName::reference())?;
            reader.mutation_target(&registry)
        })();
        match observed {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &CreateModel,
        decision: WorthQueryInvariantMutationTarget<Schema, Registry>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Created, Denial> {
        let applied = (|| -> Result<(), HandlerExecutionDenial> {
            let registry = writer
                .projected_entity(&decision)
                .map_err(HandlerExecutionDenial::new)?;
            let model = writer
                .create_entity_in_context(
                    &registry,
                    Model::reference(),
                    WorthQueryApplicationEntityKey::new(format!("model-{}", input.key))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&model, ModelName::reference(), input.name.clone())
                .map_err(HandlerExecutionDenial::new)?;
            let target = writer
                .create_entity_in_context(
                    &registry,
                    Target::reference(),
                    WorthQueryApplicationEntityKey::new(format!("target-{}", input.key))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&target, TargetName::reference(), "feature".to_owned())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&target, Retention::reference(), input.days)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    ModelTarget::reference(),
                    format!("membership-{}", input.key),
                    &model,
                    &target,
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(())
        })();
        match applied {
            Ok(()) => HandlerResult::Completed(Created),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = Create::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, RegistryName::reference())
        .operation_create(op, Model::reference())
        .operation_create(op, Target::reference())
        .operation_write(op, ModelName::reference())
        .operation_write(op, TargetName::reference())
        .operation_write(op, Retention::reference())
        .operation_link(op, ModelTarget::reference())
        .application_mutation_binding::<Binding>()
}
