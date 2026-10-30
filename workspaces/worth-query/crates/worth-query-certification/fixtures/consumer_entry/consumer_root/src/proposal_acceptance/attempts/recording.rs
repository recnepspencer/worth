//! Only actual sealed adoption custody constructs the ordinary result-recording command.
use super::super::{application::candidate::adoption::Adopted, schema::*};
use super::{data::*, schema::*};
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::{application_entry::WorthQuerySourceCompletion, primary_graph::*};
mod handler;
pub use handler::Handler;

#[derive(Clone, Debug, serde::Serialize)]
pub struct RecordResult {
    data: StoredResult,
}
#[derive(Debug)]
pub enum CorrespondenceFailure {
    AdoptionMismatch,
    Reservation(std::collections::TryReserveError),
}
impl RecordResult {
    pub fn from_adopted(
        claim: &ClaimReference,
        adopted: &Adopted<'_>,
    ) -> Result<Self, CorrespondenceFailure> {
        if adopted.metadata().attempt != claim.data.input.attempt
            || adopted.metadata().proposal != claim.data.input.reference
            || adopted.proposal() != &claim.data.input.proposal
            || adopted.completed().len() != claim.data.input.proposal.intents.len()
        {
            return Err(CorrespondenceFailure::AdoptionMismatch);
        }
        let mut steps = Vec::new();
        steps
            .try_reserve_exact(adopted.completed().len())
            .map_err(CorrespondenceFailure::Reservation)?;
        for completion in adopted.completed() {
            steps.push(match completion {
                WorthQuerySourceCompletion::Fresh { result, .. } => {
                    StoredStepOutcome::Fresh { days: result.days }
                }
                WorthQuerySourceCompletion::Recovered { .. } => StoredStepOutcome::Recovered,
            });
        }
        Ok(Self {
            data: StoredResult {
                format_version: 1,
                claim: claim.clone(),
                steps,
            },
        })
    }
    pub fn data(&self) -> &StoredResult {
        &self.data
    }
}
#[derive(Debug)]
pub struct Recorded;
#[derive(Debug)]
pub enum Denial {
    ModelMismatch,
    ClaimMismatch,
    PrincipalMismatch,
    AlreadyRecorded,
    ConflictingResult,
}
worth_query_structured_value_binding!(pub InputBinding for RecordResult {identity:"proposal-application.record-result.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Recorded {identity:"proposal-application.record-result.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denial {identity:"proposal-application.record-result.denial.v1"});
worth_query_operation!(pub SaveResult for Schema,input InputBinding);
worth_query_operation_reads!(SaveResult=>[ModelName,ModelClaim,ClaimData,ModelResult,ResultData]);
worth_query_operation_creates!(SaveResult=>[AcceptanceResult]);
worth_query_operation_writes!(SaveResult=>[ResultName,ResultData]);
worth_query_operation_links!(SaveResult=>[ModelResult]);
fn model(input: &RecordResult) -> String {
    input.data.claim.data.input.proposal.model.label.clone()
}
worth_query_mutation_binding!(pub Binding for RecordResult,schema Schema,
    identity "proposal-application.record-result.binding.v1",input InputBinding,operation SaveResult,
    result ResultBinding,idempotency u64,identity "proposal-application.record-result.key.v1",
    decision WorthQueryInvariantMutationTarget<Schema,Model>,denial DenialBinding,
    handler identity "proposal-application.record-result.handler.v1",program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,field ModelName::reference(),value model,
);
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = SaveResult::reference();
    builder
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, ModelName::reference())
        .operation_read_field(op, ClaimData::reference())
        .operation_read_field(op, ResultData::reference())
        .operation_read_relation(op, ModelClaim::reference())
        .operation_read_relation(op, ModelResult::reference())
        .operation_create(op, AcceptanceResult::reference())
        .operation_write(op, ResultName::reference())
        .operation_write(op, ResultData::reference())
        .operation_link(op, ModelResult::reference())
        .application_mutation_binding::<Binding>()
}
