use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use crate::domain_computation::primary_graph::tests::fixture::MutationFreeNotice;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_query_declaration::facade::{application_operation::*, application_schema::*};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct MeasuredInput(pub Vec<u8>);
worth_query_declaration::worth_query_portable_type!(MeasuredInput => "worth.query.test.measured-candidate-input.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub MeasuredInputBinding for MeasuredInput {
    identity: "worth.query.test.measured-candidate-input.v1"
});
worth_query_declaration::worth_query_operation!(pub MeasuredEmitOperation for ReservationSchema, input MeasuredInputBinding);
worth_query_declaration::worth_query_operation_emits!(MeasuredEmitOperation => [MutationFreeExternalEffect]);
worth_query_declaration::worth_query_mutation_binding!(
    pub MeasuredBinding for MeasuredInput, schema ReservationSchema,
    identity "worth.query.test.measured-candidate.v1",
    input MeasuredInputBinding, operation MeasuredEmitOperation, result MeasuredInputBinding,
    idempotency String, identity "worth.query.test.measured-candidate-key.v1",
    decision ApplicationCandidateRequirements, denial MeasuredInputBinding,
    handler identity "worth.query.test.measured-candidate-handler.v1",
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(), value scope,
    candidates policy ApplicationCandidatePolicy::new(None, None)
);

fn scope(_: &MeasuredInput) -> String {
    "open".to_owned()
}

pub(super) struct MeasuredHandler {
    pub(super) contacts: std::sync::Arc<AtomicUsize>,
    pub(super) cancel_on_requirements: std::sync::Arc<std::sync::Mutex<Option<worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource>>>,
}

impl OperationHandler<ReservationSchema, MeasuredBinding> for MeasuredHandler {
    fn decide(
        &self,
        input: &MeasuredInput,
        _: &mut DecisionReader<'_, '_, '_, ReservationSchema, MeasuredBinding>,
    ) -> HandlerResult<ApplicationCandidateRequirements, MeasuredInput> {
        // MutationFreeNotice(1) retains its u8 payload and one encoded byte.
        let Some(bytes) = input.0.len().checked_mul(2) else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
                    "measured representation overflow",
                ),
            ));
        };
        HandlerResult::Completed(ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, input.0.len()),
            ApplicationCandidateResourceCeiling::representation_bytes(bytes),
        ))
    }

    fn candidate_requirements(
        &self,
        _: &MeasuredInput,
        decision: &ApplicationCandidateRequirements,
    ) -> ApplicationCandidateRequirements {
        if let Some(source) = self.cancel_on_requirements.lock().unwrap().as_ref() {
            source.cancel();
        }
        *decision
    }

    fn build_candidate(
        &self,
        input: &MeasuredInput,
        _: ApplicationCandidateRequirements,
        writer: &mut CandidateWriter<'_, ReservationSchema, MeasuredBinding>,
    ) -> HandlerResult<MeasuredInput, MeasuredInput> {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        for _ in &input.0 {
            if let Err(cause) = writer.emit_external(
                MutationFreeExternalEffect::reference(),
                MutationFreeNotice(1),
            ) {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
        }
        HandlerResult::Completed(input.clone())
    }
}
