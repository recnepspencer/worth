//! This authored stage consumes the new root without changing producer inputs.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub add: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.interleaved.dependent" });
worth_query_operation!(pub PatchDependent for Schema, input Input; access {
    read_fields: [MemberKey::at("MemberKey"), EnvelopeDependent::at("Dependent"), FirstKey::at("FirstKey"), FirstValue::at("FirstValue")];
    writes: [EnvelopeDependent::at("Dependent")];
});
fn scope(_: &Intent) -> String {
    "progress:m".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.interleaved.dependent", input Input, operation PatchDependent, result fixture::StageResultBinding,
    idempotency String, identity "owned-outputs.interleaved.dependent", decision Decision, denial super::DenialBinding,
    handler identity "owned-outputs.interleaved.dependent", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope MemberKey, field MemberKey::at("MemberKey"), value scope,
);
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<Schema, Member>,
    result: fixture::StageResult,
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, super::Denial> {
        let decide = (|| {
            let target = reader.resolve_entity(MemberKey::at("MemberKey"), scope(input))?;
            let produced = reader.resolve_entity(FirstKey::at("FirstKey"), "first:a".to_owned())?;
            let value = reader
                .field(&produced, FirstValue::at("FirstValue"))?
                .expect("new root has its declared value");
            let old_value = reader
                .field(&target, EnvelopeDependent::at("Dependent"))?
                .unwrap();
            Ok::<_, HandlerExecutionDenial>(Decision {
                target: reader.mutation_target(&target)?,
                result: fixture::StageResult {
                    old_value,
                    value: value + input.add,
                    ordinal: reader.stage_ordinal().unwrap(),
                    key: reader.idempotency_key().clone(),
                },
            })
        })();
        match decide {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Intent,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<fixture::StageResult, super::Denial> {
        match writer
            .projected_entity(&decision.target)
            .and_then(|target| {
                writer.write_field(
                    &target,
                    EnvelopeDependent::at("Dependent"),
                    decision.result.value,
                )
            }) {
            Ok(()) => HandlerResult::Completed(decision.result),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<PatchDependent>(
            PatchDependent::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
