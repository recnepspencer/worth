//! #233: a decision read of a linked consumer's output through Query's
//! `output_availability`. Eager derivation answers `Current`; one commit per
//! stage leaves the output underived, and Query answers `Pending`.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryOutputAvailabilityState;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub source: String,
}
worth_query_structured_value_binding!(pub Input for Intent { identity:"owned-outputs.component-output-availability.v1" });
worth_query_operation!(pub ObserveOutputAvailability for Schema, input Input; access {
    read_entities:[Model, FirstOutput];
    read_fields:[ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), FirstKey::at("FirstKey"), FirstValue::at("FirstValue")];
});
fn scope(input: &Intent) -> String {
    input.source.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.component-output-availability.v1", input Input, operation ObserveOutputAvailability, result fixture::StageResultBinding,
    idempotency String, identity "owned-outputs.component-output-availability.v1", decision fixture::StageResult, denial super::DenialBinding,
    handler identity "owned-outputs.component-output-availability.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
/// `value` is 0 for `Pending`, otherwise the current output's `FirstValue`.
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<fixture::StageResult, super::Denial> {
        let read = (|| {
            let producer = reader.resolve_entity(ModelKey::at("ModelKey"), input.source.clone())?;
            let observed = reader.output_availability(
                &producer,
                Demand {
                    source: input.source.clone(),
                    selector: None,
                },
            )?;
            let value = match observed.state() {
                WorthQueryOutputAvailabilityState::Pending => 0,
                WorthQueryOutputAvailabilityState::Current(output) => {
                    reader.field(output, FirstValue::at("FirstValue"))?.unwrap()
                }
            };
            Ok::<_, HandlerExecutionDenial>(fixture::StageResult {
                old_value: value,
                value,
                ordinal: reader.stage_ordinal().unwrap(),
                key: reader.idempotency_key().clone(),
            })
        })();
        match read {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Intent,
        decision: fixture::StageResult,
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<fixture::StageResult, super::Denial> {
        HandlerResult::Completed(decision)
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<ObserveOutputAvailability>(
            ObserveOutputAvailability::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
