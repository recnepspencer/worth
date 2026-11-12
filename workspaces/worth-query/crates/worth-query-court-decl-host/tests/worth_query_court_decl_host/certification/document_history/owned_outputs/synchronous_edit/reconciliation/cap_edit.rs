//! An cohort-cap edit outside the producer source query.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Denial {
    ValueOutOfRange,
}
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "owned-outputs.cap-edit-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.cap-edit-denial.v1" });
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.edit-cap.v1" });
worth_query_operation!(pub PatchCap for Schema, input Input; access { read_fields: [CapKey::at("CapKey"), CapValue::at("CapValue")]; writes: [CapValue::at("CapValue")]; });
fn scope(input: &Intent) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.edit-cap.v1", input Input, operation PatchCap, result ResultBinding,
    idempotency String, identity "owned-outputs.edit-cap.v1", decision WorthQueryInvariantMutationTarget<Schema, Cap>, denial DenialBinding,
    handler identity "owned-outputs.edit-cap.v1", outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope CapKey, field CapKey::at("CapKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Cap>, Denial> {
        if input.value > 1000 {
            return HandlerResult::DomainDenied(Denial::ValueOutOfRange);
        }
        let result = (|| {
            let entity = reader.resolve_entity(CapKey::at("CapKey"), input.key.clone())?;
            reader.field(&entity, CapValue::at("CapValue"))?;
            reader.mutation_target(&entity)
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Intent,
        target: WorthQueryInvariantMutationTarget<Schema, Cap>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denial> {
        let result = writer
            .projected_entity(&target)
            .and_then(|entity| writer.write_field(&entity, CapValue::at("CapValue"), input.value));
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<PatchCap>(
            PatchCap::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
