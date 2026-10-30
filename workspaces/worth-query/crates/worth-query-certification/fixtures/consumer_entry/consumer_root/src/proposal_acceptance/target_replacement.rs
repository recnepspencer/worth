//! Explicit management authority replaces an observed target in one ordinary native candidate.
use super::{
    access::{self, grants::*},
    schema::{data::*, *},
};
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

worth_query_ability!(pub ManageTargets scoped_to Model,in Schema);
worth_query_policy!(pub TargetManagementPolicy in Schema);
#[derive(Clone, Debug, serde::Serialize)]
pub struct ReplaceTarget {
    pub model: ModelReference,
    pub target: TargetReference,
    pub key: String,
    pub days: u64,
}
#[derive(Debug)]
pub struct Replaced;
#[derive(Debug)]
pub enum Denial {
    ModelMismatch,
    TargetMismatch,
    TargetCardinality,
}
pub struct Decision {
    model: WorthQueryInvariantMutationTarget<Schema, Model>,
    target: WorthQueryInvariantMutationTarget<Schema, Target>,
    name: String,
}
worth_query_structured_value_binding!(pub InputBinding for ReplaceTarget {identity:"proposal-application.replace-target.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Replaced {identity:"proposal-application.replace-target.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denial {identity:"proposal-application.replace-target.denial.v1"});
worth_query_operation!(pub Replace for Schema,input InputBinding);
worth_query_operation_requires!(Replace=>[ManageTargets]);
worth_query_operation_reads!(Replace=>[Model,Target,ModelName,TargetName,ModelTarget]);
worth_query_operation_writes!(Replace=>[TargetName,Retention]);
worth_query_operation_creates!(Replace=>[Target]);
worth_query_operation_deletes!(Replace=>[Target]);
worth_query_operation_links!(Replace=>[ModelTarget]);
worth_query_operation_unlinks!(Replace=>[ModelTarget]);
fn model(input: &ReplaceTarget) -> String {
    input.model.label.clone()
}
worth_query_mutation_binding!(pub Binding for ReplaceTarget,schema Schema,
    identity "proposal-application.replace-target.binding.v1",input InputBinding,operation Replace,
    result ResultBinding,idempotency u64,identity "proposal-application.replace-target.key.v1",
    decision Decision,denial DenialBinding,handler identity "proposal-application.replace-target.handler.v1",program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,field ModelName::reference(),value model,
);

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &ReplaceTarget,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let model = reader.scope().clone();
        let id = model.entity_id();
        let occurrence = Coordinates {
            partition: id.partition_value(),
            slot: id.local_slot_value(),
            generation: id.generation_value(),
        };
        if reader.reader().runtime_instance_id() != input.model.runtime
            || occurrence != input.model.occurrence
        {
            return HandlerResult::DomainDenied(Denial::ModelMismatch);
        }
        let observed = (|| -> Result<_, HandlerExecutionDenial> {
            if reader.field(&model, ModelName::reference())? != Some(input.model.label.clone()) {
                return Ok(Err(Denial::ModelMismatch));
            }
            let targets = reader
                .relations_from(ModelTarget::reference(), &model)
                .map_err(HandlerExecutionDenial::new)?;
            let [edge] = targets.as_slice() else {
                return Ok(Err(Denial::TargetCardinality));
            };
            let target = edge.to();
            let id = target.entity_id();
            let occurrence = Coordinates {
                partition: id.partition_value(),
                slot: id.local_slot_value(),
                generation: id.generation_value(),
            };
            let Some(name) = reader.field(target, TargetName::reference())? else {
                return Ok(Err(Denial::TargetMismatch));
            };
            if occurrence != input.target.occurrence || name != input.target.label {
                return Ok(Err(Denial::TargetMismatch));
            }
            // Record the entity-existence fact required by native retirement.
            reader.entity(target, Target::reference())?;
            Ok(Ok(Decision {
                model: reader.mutation_target(&model)?,
                target: reader.mutation_target(target)?,
                name,
            }))
        })();
        match observed {
            Ok(Ok(decision)) => HandlerResult::Completed(decision),
            Ok(Err(cause)) => HandlerResult::DomainDenied(cause),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &ReplaceTarget,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Replaced, Denial> {
        let applied = (|| -> Result<(), HandlerExecutionDenial> {
            let model = writer
                .projected_entity(&decision.model)
                .map_err(HandlerExecutionDenial::new)?;
            let target = writer
                .projected_entity(&decision.target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .unlink(ModelTarget::reference(), &model, &target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .delete_entity(Target::reference(), &target)
                .map_err(HandlerExecutionDenial::new)?;
            let replacement = writer
                .create_entity_in_context(
                    &model,
                    Target::reference(),
                    WorthQueryApplicationEntityKey::new(input.key.clone())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&replacement, TargetName::reference(), decision.name)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&replacement, Retention::reference(), input.days)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    ModelTarget::reference(),
                    input.key.clone(),
                    &model,
                    &replacement,
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(())
        })();
        match applied {
            Ok(()) => HandlerResult::Completed(Replaced),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = Replace::reference();
    schema
        .ability(ManageTargets::reference())
        .policy(TargetManagementPolicy::reference())
        .ability_policy(
            ManageTargets::reference(),
            TargetManagementPolicy::reference(),
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .forward(GrantGrantor::reference())
                    .where_equal(GrantStatus::reference(), access::encoded(1))
                    .forward(GrantResource::reference())
                    .allow(Model::reference()),
            ],
        )
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_requires_ability(op, ManageTargets::reference())
        .operation_projection_work_budget(op, 32)
        .operation_read_entity(op, Model::reference())
        .operation_read_entity(op, Target::reference())
        .operation_read_field(op, ModelName::reference())
        .operation_read_field(op, TargetName::reference())
        .operation_read_relation(op, ModelTarget::reference())
        .operation_unlink(op, ModelTarget::reference())
        .operation_delete(op, Target::reference())
        .operation_create(op, Target::reference())
        .operation_write(op, TargetName::reference())
        .operation_write(op, Retention::reference())
        .operation_link(op, ModelTarget::reference())
        .application_mutation_binding::<Binding>()
}
