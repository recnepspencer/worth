//! A model-scoped operation creates a member and links the model as its owner.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-model.member-birth.v1" });
worth_query_operation!(pub AddMember for Schema, input Input; access {
    read_entities: [Model]; read_fields: [ModelKey::at("ModelKey")]; creates: [Member]; writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")]; links: [ModelMember];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-model.member-birth.v1", input Input, operation AddMember, result edit::ResultBinding,
    idempotency String, identity "owned-model.member-birth-key.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial edit::DenialBinding,
    handler identity "owned-model.member-birth-handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, edit::Denial> {
        match reader
            .resolve_entity(ModelKey::at("ModelKey"), "a".into())
            .and_then(|model| reader.mutation_target(&model))
        {
            Ok(model) => HandlerResult::Completed(model),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Intent,
        model: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let model = writer.projected_entity(&model)?;
            let key = WorthQueryApplicationEntityKey::new(input.key.clone()).unwrap();
            let member = writer.create_entity_in_context(&model, Member::reference(), key)?;
            writer.initialize_field(&member, MemberKey::at("MemberKey"), input.key.clone())?;
            writer.initialize_field(&member, MemberValue::at("MemberValue"), input.value)?;
            writer.link(
                ModelMember::reference(),
                format!("born:{}", input.key),
                &model,
                &member,
            )
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<AddMember>(
            AddMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
