//! An authored stage consumes the child's genuine value before a later edit.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
}
worth_query_structured_value_binding!(pub Input for Intent { identity:"owned-outputs.consume-child.v1" });
worth_query_operation!(pub Consume for Schema, input Input; access {
    read_fields:[ChildKey::at("ChildKey"), ChildValue::at("ChildValue")];
});
fn scope(input: &Intent) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.consume-child.v1", input Input, operation Consume, result fixture::StageResultBinding,
    idempotency String, identity "owned-outputs.consume-child.v1", decision fixture::StageResult, denial super::DenialBinding,
    handler identity "owned-outputs.consume-child.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ChildKey, field ChildKey::at("ChildKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<fixture::StageResult, super::Denial> {
        let read = (|| {
            let row = reader.resolve_entity(ChildKey::at("ChildKey"), input.key.clone())?;
            let value = reader.field(&row, ChildValue::at("ChildValue"))?.unwrap();
            Ok::<_, HandlerExecutionDenial>(fixture::StageResult {
                old_value: value,
                value,
                ordinal: reader.stage_ordinal().unwrap(),
                key: reader.idempotency_key().clone(),
            })
        })();
        match read {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Intent,
        decision: fixture::StageResult,
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<fixture::StageResult, super::Denial> {
        HandlerResult::Completed(decision)
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Consume>(
            Consume::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
