//! A positioned stage that declares a read of the second family and links a
//! new input member to model `a`, which the first family reads.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.interleaved.input-arrival" });
worth_query_operation!(pub ArriveInput for Schema, input Input; access {
    read_entities: [Model]; read_fields: [ModelKey::at("ModelKey"), SecondKey::at("SecondKey"), SecondValue::at("SecondValue")];
    creates: [InputMember]; writes: [InputKey::at("InputKey"), InputValue::at("InputValue")]; links: [InputOwner];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.interleaved.input-arrival", input Input, operation ArriveInput, result fixture::StageResultBinding,
    idempotency String, identity "owned-outputs.interleaved.input-arrival", decision Decision, denial super::DenialBinding,
    handler identity "owned-outputs.interleaved.input-arrival", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Decision {
    owner: WorthQueryInvariantMutationTarget<Schema, Model>,
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
            let owner = reader.resolve_entity(ModelKey::at("ModelKey"), scope(input))?;
            Ok::<_, HandlerExecutionDenial>(Decision {
                owner: reader.mutation_target(&owner)?,
                result: fixture::StageResult {
                    old_value: 0,
                    value: input.value,
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
        input: &Intent,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<fixture::StageResult, super::Denial> {
        let build = (|| {
            let owner = writer.projected_entity(&decision.owner)?;
            let member = writer.create_entity_in_context(
                &owner,
                InputMember::reference(),
                WorthQueryApplicationEntityKey::new(format!("input:{}", input.key)).unwrap(),
            )?;
            writer.initialize_field(&member, InputKey::at("InputKey"), input.key.clone())?;
            writer.initialize_field(&member, InputValue::at("InputValue"), input.value)?;
            writer.link(
                InputOwner::reference(),
                format!("input-owner:{}", input.key),
                &member,
                &owner,
            )
        })();
        match build {
            Ok(()) => HandlerResult::Completed(decision.result),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<ArriveInput>(
            ArriveInput::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
