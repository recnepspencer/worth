//! Saving a proposal is an ordinary typed operation on its storage branch.
use super::schema::{
    data::{Coordinates, StoredProposal},
    *,
};
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct RecordProposal {
    pub model: String,
    pub key: String,
    pub data: StoredProposal,
}
#[derive(Debug)]
pub struct Recorded;
#[derive(Debug)]
pub enum Denial {
    ModelMismatch,
}
worth_query_structured_value_binding!(pub InputBinding for RecordProposal { identity:"proposal-application.record.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Recorded { identity:"proposal-application.record.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity:"proposal-application.record.denial.v1" });
worth_query_operation!(pub Record for Schema, input InputBinding);
worth_query_operation_reads!(Record => [ModelName]);
worth_query_operation_writes!(Record => [ProposalName,ProposalData]);
worth_query_operation_creates!(Record => [Proposal]);
worth_query_operation_links!(Record => [ModelProposal]);
fn model(input: &RecordProposal) -> String {
    input.model.clone()
}
worth_query_mutation_binding!(pub Binding for RecordProposal, schema Schema,
    identity "proposal-application.record.binding.v1", input InputBinding, operation Record,
    result ResultBinding, idempotency u64, identity "proposal-application.record.key.v1",
    decision WorthQueryInvariantMutationTarget<Schema,Model>, denial DenialBinding,
    handler identity "proposal-application.record.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,
    field ModelName::reference(), value model,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &RecordProposal,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, Denial> {
        let observed = (|| -> Result<_, HandlerExecutionDenial> {
            let model = reader.resolve_entity(ModelName::reference(), input.model.clone())?;
            reader.field(&model, ModelName::reference())?;
            let id = model.entity_id();
            let coordinate = Coordinates {
                partition: id.partition_value(),
                slot: id.local_slot_value(),
                generation: id.generation_value(),
            };
            if coordinate != input.data.model.occurrence || input.model != input.data.model.label {
                return Ok(None);
            }
            reader.mutation_target(&model).map(Some)
        })();
        match observed {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::ModelMismatch),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &RecordProposal,
        target: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Recorded, Denial> {
        let applied = (|| -> Result<(), HandlerExecutionDenial> {
            let model = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            let proposal = writer
                .create_entity_in_context(
                    &model,
                    Proposal::reference(),
                    WorthQueryApplicationEntityKey::new(input.key.clone())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&proposal, ProposalName::reference(), input.key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&proposal, ProposalData::reference(), input.data.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    ModelProposal::reference(),
                    input.key.clone(),
                    &model,
                    &proposal,
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(())
        })();
        match applied {
            Ok(()) => HandlerResult::Completed(Recorded),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = Record::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, ModelName::reference())
        .operation_create(op, Proposal::reference())
        .operation_write(op, ProposalName::reference())
        .operation_write(op, ProposalData::reference())
        .operation_link(op, ModelProposal::reference())
        .application_mutation_binding::<Binding>()
}
