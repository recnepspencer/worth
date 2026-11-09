//! Ordinary relation authoring supplies a live incoming edge from outside a roster.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Connect {
    pub from: String,
    pub to: String,
}
worth_query_structured_value_binding!(pub Input for Connect { identity: "source-subgraph.connect-input.v1" });
worth_query_operation!(pub ConnectMembers for Schema, input Input; access {
    read_fields: [SourceKey::at("SourceKey"), MemberKey::at("MemberKey")]; read_relations: [Next]; links: [Next];
});
fn scope(_: &Connect) -> String {
    "source".into()
}
worth_query_mutation_binding!(pub Binding for Connect, schema Schema,
    identity "source-subgraph.connect.v1", input Input, operation ConnectMembers, result edit::ResultBinding,
    idempotency String, identity "source-subgraph.connect-key.v1", decision (MemberTarget, MemberTarget), denial edit::DenialBinding,
    handler identity "source-subgraph.connect-handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope SourceKey, field SourceKey::at("SourceKey"), value scope,
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<ConnectMembers>(
            ConnectMembers::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Connect,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(MemberTarget, MemberTarget), edit::Denial> {
        let result = (|| {
            let from = reader.resolve_entity(MemberKey::at("MemberKey"), input.from.clone())?;
            let to = reader.resolve_entity(MemberKey::at("MemberKey"), input.to.clone())?;
            reader
                .relations_from(Next::reference(), &from)
                .map_err(HandlerExecutionDenial::new)?;
            Ok((reader.mutation_target(&from)?, reader.mutation_target(&to)?))
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Connect,
        target: (MemberTarget, MemberTarget),
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let from = writer.projected_entity(&target.0)?;
            let to = writer.projected_entity(&target.1)?;
            writer.link(Next::reference(), "outside-incoming", &from, &to)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
