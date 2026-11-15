//! A later authored stage consumes a real produced value and adds a dependent.
use super::*;

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub parent: String,
    pub child: String,
    pub pending_child: bool,
}
worth_query_structured_value_binding!(pub Input for Intent { identity:"owned-outputs.component-attach.v1" });
worth_query_operation!(pub Attach for Schema, input Input; access {
    read_entities:[Model, FirstOutput];
    read_fields:[ModelKey::at("ModelKey"), FirstKey::at("FirstKey"), FirstValue::at("FirstValue")];
    creates:[Member]; writes:[MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
    links:[ModelMember, UpstreamOf, PendingChildOf];
});
fn scope(input: &Intent) -> String {
    input.child.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.component-attach.v1", input Input, operation Attach, result fixture::StageResultBinding,
    idempotency String, identity "owned-outputs.component-attach.v1", decision Decision, denial super::DenialBinding,
    handler identity "owned-outputs.component-attach.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Decision {
    parent: WorthQueryInvariantMutationTarget<Schema, Model>,
    child: WorthQueryInvariantMutationTarget<Schema, Model>,
    output: WorthQueryInvariantMutationTarget<Schema, FirstOutput>,
    result: fixture::StageResult,
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, super::Denial> {
        let capture = (|| {
            let parent = reader.resolve_entity(ModelKey::at("ModelKey"), input.parent.clone())?;
            let child = reader.resolve_entity(ModelKey::at("ModelKey"), input.child.clone())?;
            let output = reader
                .resolve_entity(FirstKey::at("FirstKey"), format!("first:{}", input.parent))?;
            let value = reader
                .field(&output, FirstValue::at("FirstValue"))?
                .unwrap();
            Ok::<_, HandlerExecutionDenial>(Decision {
                parent: reader.mutation_target(&parent)?,
                child: reader.mutation_target(&child)?,
                output: reader.mutation_target(&output)?,
                result: fixture::StageResult {
                    old_value: value,
                    value,
                    ordinal: reader.stage_ordinal().unwrap(),
                    key: reader.idempotency_key().clone(),
                },
            })
        })();
        match capture {
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
        let apply = (|| {
            let parent = writer.projected_entity(&decision.parent)?;
            let child = writer.projected_entity(&decision.child)?;
            let role = if input.pending_child {
                "pending-child"
            } else {
                "previous"
            };
            let key = format!("{}:{role}", input.child);
            let link = writer.create_entity_in_context(
                &child,
                Member::reference(),
                WorthQueryApplicationEntityKey::new(&key).unwrap(),
            )?;
            writer.initialize_field(&link, MemberKey::at("MemberKey"), key.clone())?;
            writer.initialize_field(&link, MemberValue::at("MemberValue"), 0)?;
            writer.link(
                ModelMember::reference(),
                format!("owns:{key}"),
                &child,
                &link,
            )?;
            if input.pending_child {
                let output = writer.projected_entity(&decision.output)?;
                writer.link(
                    PendingChildOf::reference(),
                    format!("uses:{key}"),
                    &link,
                    &output,
                )
            } else {
                writer.link(
                    UpstreamOf::reference(),
                    format!("uses:{key}"),
                    &link,
                    &parent,
                )
            }
        })();
        match apply {
            Ok(()) => HandlerResult::Completed(decision.result),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Attach>(
            Attach::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
