//! Two positioned Creates share a seeded owner; the second reads the first at D.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub previous: Option<String>,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity:"owned-outputs.interleaved.created.v1" });
worth_query_operation!(pub Create for Schema, input Input; access {
    read_entities:[Model, Member]; read_fields:[ModelKey::at("ModelKey"),MemberKey::at("MemberKey"),MemberValue::at("MemberValue")];
    creates:[Member]; writes:[MemberKey::at("MemberKey"),MemberValue::at("MemberValue")]; links:[ModelMember];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.interleaved.created.v1", input Input, operation Create, result fixture::StageResultBinding,
    idempotency String, identity "owned-outputs.interleaved.created.v1", decision Decision, denial super::DenialBinding,
    handler identity "owned-outputs.interleaved.created.v1", program required, outputs NoApplicationMutationOutputs,
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
        if input.value > 1000 {
            return HandlerResult::DomainDenied(super::Denial::ValueOutOfRange);
        }
        let decide = (|| {
            let owner = reader.resolve_entity(ModelKey::at("ModelKey"), scope(input))?;
            let prior = match &input.previous {
                Some(key) => {
                    let earlier = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
                    reader
                        .field(&earlier, MemberValue::at("MemberValue"))?
                        .expect("earlier created value")
                }
                None => 0,
            };
            Ok::<_, HandlerExecutionDenial>(Decision {
                owner: reader.mutation_target(&owner)?,
                result: fixture::StageResult {
                    old_value: prior,
                    value: prior + input.value,
                    ordinal: reader.stage_ordinal().unwrap(),
                    key: reader.idempotency_key().clone(),
                },
            })
        })();
        match decide {
            Ok(d) => HandlerResult::Completed(d),
            Err(e) => HandlerResult::ExecutionDenied(e),
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
                Member::reference(),
                WorthQueryApplicationEntityKey::new(&input.key).unwrap(),
            )?;
            writer.initialize_field(&member, MemberKey::at("MemberKey"), input.key.clone())?;
            writer.initialize_field(
                &member,
                MemberValue::at("MemberValue"),
                decision.result.value,
            )?;
            writer.link(ModelMember::reference(), input.key.clone(), &owner, &member)
        })();
        match build {
            Ok(()) => HandlerResult::Completed(decision.result),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Create>(
            Create::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
