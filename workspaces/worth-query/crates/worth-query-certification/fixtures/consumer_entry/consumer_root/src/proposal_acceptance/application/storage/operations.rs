//! Actual owning metadata actions retain their native publication and cleanup custody.
use super::super::super::{
    attempts::{claim, data::AttemptInput, recording},
    host::{ClaimAction, Program, ResultAction},
    schema::Schema,
};
use super::*;
use worth_query_decl::facade::application_program::ApplicationSourceCompletion;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution;
pub type ClaimOperation = WorthQuerySourceOnlyOperation<
    Schema,
    AttemptInput,
    Program,
    ClaimAction,
    ApplicationSourceCompletion,
>;
pub type ClaimCompleted = WorthQueryCompletedSourceOperation<
    Schema,
    AttemptInput,
    Program,
    ClaimAction,
    ApplicationSourceCompletion,
>;
pub type ClaimTerminal = WorthQueryTerminalSourceOperation<
    Schema,
    AttemptInput,
    Program,
    ClaimAction,
    ApplicationSourceCompletion,
>;
pub type ClaimCompletion = WorthQuerySourceCompletion<claim::Registered>;
pub type ResultOperation = WorthQuerySourceOnlyOperation<
    Schema,
    recording::RecordResult,
    Program,
    ResultAction,
    ApplicationSourceCompletion,
>;
pub type ResultCompleted = WorthQueryCompletedSourceOperation<
    Schema,
    recording::RecordResult,
    Program,
    ResultAction,
    ApplicationSourceCompletion,
>;
pub type ResultTerminal = WorthQueryTerminalSourceOperation<
    Schema,
    recording::RecordResult,
    Program,
    ResultAction,
    ApplicationSourceCompletion,
>;
pub type ResultCompletion = WorthQuerySourceCompletion<recording::Recorded>;
#[derive(Debug)]
pub enum ConstructionFailure {
    Capture(WorthQueryOperationInputCaptureDenial),
    Action(WorthQueryOperationConstructionDenial),
}
impl Storage<'_> {
    pub(in crate::proposal_acceptance) fn claim_operation(
        &self,
        caller: &Caller<'_>,
        input: &AttemptInput,
        key: u64,
    ) -> Result<ClaimOperation, ConstructionFailure> {
        self.owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .mutate(input.clone())
            .without_source()
            .idempotency_owned(key)
            .map_err(|(_, _, cause)| ConstructionFailure::Capture(cause))?
            .retain_operation::<Program, ClaimAction>(self.owner.host)
            .map_err(|(_, cause)| ConstructionFailure::Action(cause))
    }
    pub(in crate::proposal_acceptance) fn result_operation(
        &self,
        caller: &Caller<'_>,
        input: &recording::RecordResult,
        key: u64,
    ) -> Result<ResultOperation, ConstructionFailure> {
        self.owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .mutate(input.clone())
            .without_source()
            .idempotency_owned(key)
            .map_err(|(_, _, cause)| ConstructionFailure::Capture(cause))?
            .retain_operation::<Program, ResultAction>(self.owner.host)
            .map_err(|(_, cause)| ConstructionFailure::Action(cause))
    }
    pub(in crate::proposal_acceptance::application) fn resolve_claim(
        &self,
        caller: &Caller<'_>,
        input: &AttemptInput,
        key: u64,
    ) -> Result<
        WorthQueryApplicationIdempotencyResolution,
        WorthQueryApplicationRecoveryRequestDenial,
    > {
        self.owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .advanced()
            .mutate(input.clone())
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(self.owner.host)
            .map(|read| read.into_resolution())
    }
}
