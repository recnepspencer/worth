//! An authored deletion retires one named member through the ordinary lane.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "persistent-naming.delete-member.v1" });
worth_query_operation!(pub DeleteMember for Schema, input Input; access {
    read_entities: [Model, Member]; read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey")]; deletes: [Member];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "persistent-naming.delete-member.v1", input Input, operation DeleteMember, result edit::ResultBinding,
    idempotency String, identity "persistent-naming.delete-member.v1", decision WorthQueryInvariantMutationTarget<Schema, Member>, denial edit::DenialBinding,
    handler identity "persistent-naming.delete-member.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Member>, edit::Denial> {
        let result = reader
            .resolve_entity(MemberKey::at("MemberKey"), input.key.clone())
            .and_then(|member| {
                reader.entity(&member, Member::reference())?;
                reader.mutation_target(&member)
            });
        match result {
            Ok(member) => HandlerResult::Completed(member),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Intent,
        member: WorthQueryInvariantMutationTarget<Schema, Member>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = writer
            .projected_entity(&member)
            .and_then(|member| writer.delete_entity(Member::reference(), &member));
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
        .declared_operation::<DeleteMember>(
            DeleteMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
