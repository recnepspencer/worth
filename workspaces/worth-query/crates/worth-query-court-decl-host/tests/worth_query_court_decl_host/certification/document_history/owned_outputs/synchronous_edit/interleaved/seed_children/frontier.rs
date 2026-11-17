//! A final authored stage that demands no output and reads no producer: it
//! only moves the progress frontier, as a CAD correction's document stage
//! follows its source stage.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub frontier: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Denial;
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.interleaved.seed-frontier" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.interleaved.seed-frontier-denial" });
worth_query_structured_value_binding!(pub ResultBinding for u64 { identity: "owned-outputs.interleaved.seed-frontier-result" });
worth_query_operation!(pub Advance for Schema, input Input; access {
    read_fields: [MemberKey::at("MemberKey"), EnvelopeFrontier::at("Frontier")];
    writes: [EnvelopeFrontier::at("Frontier")];
});
fn scope(_: &Intent) -> String {
    "progress:envelope-calibration".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.interleaved.seed-frontier", input Input, operation Advance, result ResultBinding,
    idempotency String, identity "owned-outputs.interleaved.seed-frontier", decision Decision, denial DenialBinding,
    handler identity "owned-outputs.interleaved.seed-frontier", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope MemberKey, field MemberKey::at("MemberKey"), value scope,
);
pub struct Decision(WorthQueryInvariantMutationTarget<Schema, Member>);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        // The platform refuses a blind write, so the stage observes the field.
        let target = (reader.resolve_entity(MemberKey::at("MemberKey"), scope(input)))
            .and_then(|target| {
                reader.field(&target, EnvelopeFrontier::at("Frontier"))?;
                reader.mutation_target(&target)
            });
        match target {
            Ok(target) => HandlerResult::Completed(Decision(target)),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Intent,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<u64, Denial> {
        let write = (writer.projected_entity(&decision.0)).and_then(|target| {
            writer.write_field(&target, EnvelopeFrontier::at("Frontier"), input.frontier)
        });
        match write {
            Ok(()) => HandlerResult::Completed(input.frontier),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Advance>(
            Advance::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
