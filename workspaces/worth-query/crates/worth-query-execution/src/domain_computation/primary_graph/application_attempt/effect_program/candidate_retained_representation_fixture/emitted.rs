//! An installed ordinary emission carries its original measured typed payload.

use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    RetainedStatusNotice, RetainedStatusNoticeBinding,
};
use worth_query_declaration::facade::{application_operation::*, application_schema::*};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct QuotedEmissionInput {
    pub value: String,
    pub underquote: bool,
}
worth_query_declaration::worth_query_portable_type!(QuotedEmissionInput => "worth.query.test.quoted-emission-input.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub QuotedEmissionInputBinding for QuotedEmissionInput {
    identity: "worth.query.test.quoted-emission-input.v1"
});
worth_query_declaration::worth_query_operation!(pub QuotedEmissionOperation for ReservationSchema, input QuotedEmissionInputBinding);
worth_query_declaration::worth_query_effect!(pub QuotedRetainedEffect for ReservationSchema, payload RetainedStatusNoticeBinding);
worth_query_declaration::worth_query_operation_emits!(QuotedEmissionOperation => [QuotedRetainedEffect]);

pub struct QuotedEmissionDecision {
    payload: RetainedStatusNotice,
    demand: ApplicationCandidateRequirements,
}
worth_query_declaration::worth_query_mutation_binding!(
    pub QuotedEmissionBinding for QuotedEmissionInput, schema ReservationSchema,
    identity "worth.query.test.quoted-emission.v1",
    input QuotedEmissionInputBinding, operation QuotedEmissionOperation, result QuotedEmissionInputBinding,
    idempotency String, identity "worth.query.test.quoted-emission-key.v1",
    decision QuotedEmissionDecision, denial QuotedEmissionInputBinding,
    handler identity "worth.query.test.quoted-emission-handler.v1",
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(), value scope,
    candidates policy ApplicationCandidatePolicy::new(None, None)
);

fn scope(_: &QuotedEmissionInput) -> String {
    "open".to_owned()
}

pub(super) struct QuotedEmissionHandler;
impl OperationHandler<ReservationSchema, QuotedEmissionBinding> for QuotedEmissionHandler {
    fn decide(
        &self,
        input: &QuotedEmissionInput,
        reader: &mut DecisionReader<'_, '_, '_, ReservationSchema, QuotedEmissionBinding>,
    ) -> HandlerResult<QuotedEmissionDecision, QuotedEmissionInput> {
        let payload = RetainedStatusNotice(input.value.clone());
        let bytes = match reader.candidate_emit_bytes(QuotedRetainedEffect::reference(), &payload) {
            Ok(bytes) => bytes,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
        };
        let bytes = if input.underquote {
            bytes
                .checked_sub(1)
                .expect("valid notice retains at least one byte")
        } else {
            bytes
        };
        HandlerResult::Completed(QuotedEmissionDecision {
            payload,
            demand: ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 1),
                ApplicationCandidateResourceCeiling::representation_bytes(bytes),
            ),
        })
    }

    fn candidate_requirements(
        &self,
        _: &QuotedEmissionInput,
        decision: &QuotedEmissionDecision,
    ) -> ApplicationCandidateRequirements {
        decision.demand
    }

    fn build_candidate(
        &self,
        input: &QuotedEmissionInput,
        decision: QuotedEmissionDecision,
        writer: &mut CandidateWriter<'_, ReservationSchema, QuotedEmissionBinding>,
    ) -> HandlerResult<QuotedEmissionInput, QuotedEmissionInput> {
        match writer.emit(QuotedRetainedEffect::reference(), decision.payload) {
            Ok(()) => HandlerResult::Completed(input.clone()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
