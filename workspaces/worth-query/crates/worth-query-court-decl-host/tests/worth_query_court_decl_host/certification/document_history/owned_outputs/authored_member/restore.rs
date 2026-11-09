//! Recorded authored inverse can recreate the retired member and its model edge.
use super::super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.authored-member-restore.v1" });
worth_query_operation!(pub RestoreMember for Schema, input Input; access {
    read_entities: [Model]; read_fields: [ModelKey::at("ModelKey")]; creates: [Member]; writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")]; links: [ModelMember];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.authored-member-restore.v1", input Input, operation RestoreMember, result member_edit::ResultBinding,
    idempotency String, identity "owned-outputs.authored-member-restore.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial member_edit::DenialBinding,
    handler identity "owned-outputs.authored-member-restore.v1", program required, outputs Outputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Outputs;
pub struct Restored;
impl WorthQueryApplicationOutputRole for Restored {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Member;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-member-restored";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<Restored as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, member_edit::Denial> {
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
    ) -> HandlerResult<member_edit::Edited, member_edit::Denial> {
        let result = (|| {
            let model = writer.projected_entity(&model)?;
            let key = WorthQueryApplicationEntityKey::new(input.key.clone()).unwrap();
            let member = writer.create_entity_in_context(&model, Member::reference(), key)?;
            writer.initialize_field(&member, MemberKey::at("MemberKey"), input.key.clone())?;
            writer.initialize_field(&member, MemberValue::at("MemberValue"), input.value)?;
            writer.create_output::<Restored>(&member)?;
            writer.link(
                ModelMember::reference(),
                String::from("owned-restored-member"),
                &model,
                &member,
            )
        })();
        match result {
            Ok(()) => HandlerResult::Completed(member_edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<RestoreMember>(
            RestoreMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
