//! Remove a real input association while leaving its producer source alive.
use super::*;

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent;
worth_query_structured_value_binding!(pub Input for Intent { identity: "borrowed-inputs.disconnect" });
worth_query_operation!(pub Disconnect for Schema, input Input; access {
    read_entities: [Model, Member];
    read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey")];
    read_relations: [ModelMember]; unlinks: [ModelMember];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
pub struct Targets {
    owner: WorthQueryInvariantMutationTarget<Schema, Model>,
    member: WorthQueryInvariantMutationTarget<Schema, Member>,
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "borrowed-inputs.disconnect", input Input, operation Disconnect, result member_edit::ResultBinding,
    idempotency String, identity "borrowed-inputs.disconnect", decision Targets, denial member_edit::DenialBinding,
    handler identity "borrowed-inputs.disconnect", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn declared_durable_capabilities(
    ) -> Option<&'static [worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability]> {
        Some(&[worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability::RelationalMutation])
    }

    fn decide(
        &self,
        _: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Targets, member_edit::Denial> {
        let result = (|| {
            let owner = reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
            let member = reader.resolve_entity(MemberKey::at("MemberKey"), "m".to_owned())?;
            reader.entity(&owner, Model::reference())?;
            reader.entity(&member, Member::reference())?;
            let links = reader
                .relations_from(ModelMember::reference(), &owner)
                .map_err(HandlerExecutionDenial::new)?;
            assert!(links
                .iter()
                .any(|link| link.to().entity_id() == member.entity_id()));
            Ok(Targets {
                owner: reader.mutation_target(&owner)?,
                member: reader.mutation_target(&member)?,
            })
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _: &Intent,
        targets: Targets,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<member_edit::Edited, member_edit::Denial> {
        let result = (|| {
            let owner = writer.projected_entity(&targets.owner)?;
            let member = writer.projected_entity(&targets.member)?;
            writer.unlink(ModelMember::reference(), &owner, &member)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(member_edit::Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Disconnect>(
            Disconnect::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
