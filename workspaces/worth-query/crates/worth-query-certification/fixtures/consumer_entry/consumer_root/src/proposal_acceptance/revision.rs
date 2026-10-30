//! An ordinary observed field update revises intent without replacing its occurrence or basis.
use super::schema::{data::*, *};
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ReviseProposal {
    pub reference: ProposalReference,
    pub intents: Vec<AuthoredIntent>,
}
#[derive(Debug)]
pub struct Revised {
    pub revision: u64,
}
#[derive(Debug)]
pub enum Denial {
    ModelMismatch,
    ProposalChanged,
    UnsupportedVersion,
    RevisionExhausted,
}
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<Schema, Proposal>,
    next: StoredProposal,
}
worth_query_structured_value_binding!(pub InputBinding for ReviseProposal {identity:"proposal-application.revise.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Revised {identity:"proposal-application.revise.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denial {identity:"proposal-application.revise.denial.v1"});
worth_query_operation!(pub Revise for Schema,input InputBinding);
worth_query_operation_reads!(Revise=>[ModelName,ModelProposal,ProposalName,ProposalData]);
worth_query_operation_writes!(Revise=>[ProposalData]);
fn model(input: &ReviseProposal) -> String {
    input.reference.model.label.clone()
}
worth_query_mutation_binding!(pub Binding for ReviseProposal,schema Schema,
    identity "proposal-application.revise.binding.v1",input InputBinding,operation Revise,
    result ResultBinding,idempotency u64,identity "proposal-application.revise.key.v1",
    decision Decision,denial DenialBinding,handler identity "proposal-application.revise.handler.v1",program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,field ModelName::reference(),value model,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &ReviseProposal,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let model = reader.scope().clone();
        let id = model.entity_id();
        let occurrence = Coordinates {
            partition: id.partition_value(),
            slot: id.local_slot_value(),
            generation: id.generation_value(),
        };
        if reader.reader().runtime_instance_id() != input.reference.model.runtime
            || occurrence != input.reference.model.occurrence
        {
            return HandlerResult::DomainDenied(Denial::ModelMismatch);
        }
        let observed = (|| -> Result<_, HandlerExecutionDenial> {
            if reader.field(&model, ModelName::reference())?
                != Some(input.reference.model.label.clone())
            {
                return Ok(Err(Denial::ModelMismatch));
            }
            let mut selected = None;
            for relation in reader
                .relations_from(ModelProposal::reference(), &model)
                .map_err(HandlerExecutionDenial::new)?
            {
                let proposal = relation.to();
                let id = proposal.entity_id();
                let occurrence = Coordinates {
                    partition: id.partition_value(),
                    slot: id.local_slot_value(),
                    generation: id.generation_value(),
                };
                if occurrence != input.reference.occurrence {
                    continue;
                }
                if selected.is_some() {
                    return Ok(Err(Denial::ProposalChanged));
                }
                let name = reader.field(proposal, ProposalName::reference())?;
                let Some(mut next) = reader.field(proposal, ProposalData::reference())? else {
                    return Ok(Err(Denial::ProposalChanged));
                };
                if name != Some(input.reference.key.clone())
                    || next.model != input.reference.model
                    || next.revision != input.reference.revision
                {
                    return Ok(Err(Denial::ProposalChanged));
                }
                if next.format_version != 1 {
                    return Ok(Err(Denial::UnsupportedVersion));
                }
                let Some(revision) = next.revision.checked_add(1) else {
                    return Ok(Err(Denial::RevisionExhausted));
                };
                next.revision = revision;
                next.intents = input.intents.clone();
                selected = Some(Decision {
                    target: reader.mutation_target(proposal)?,
                    next,
                });
            }
            Ok(selected.ok_or(Denial::ProposalChanged))
        })();
        match observed {
            Ok(Ok(decision)) => HandlerResult::Completed(decision),
            Ok(Err(cause)) => HandlerResult::DomainDenied(cause),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &ReviseProposal,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Revised, Denial> {
        let apply = (|| -> Result<_, HandlerExecutionDenial> {
            let target = writer
                .projected_entity(&decision.target)
                .map_err(HandlerExecutionDenial::new)?;
            let revision = decision.next.revision;
            writer
                .write_field(&target, ProposalData::reference(), decision.next)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(Revised { revision })
        })();
        match apply {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = Revise::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, ModelName::reference())
        .operation_read_field(op, ProposalName::reference())
        .operation_read_field(op, ProposalData::reference())
        .operation_read_relation(op, ModelProposal::reference())
        .operation_write(op, ProposalData::reference())
        .application_mutation_binding::<Binding>()
}
