//! The one recorded source edit: an ordinary completion writing the model value.
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
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "owned-outputs.edit-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.edit-denial.v1" });
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.edit-model.v1" });
worth_query_operation!(pub PatchModel for Schema, input Input; access { read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), predecessor::content::Values::at("Payload")]; writes: [ModelValue::at("ModelValue"), predecessor::content::Values::at("Payload")]; });
fn scope(input: &Intent) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.edit-model.v1", input Input, operation PatchModel, result ResultBinding,
    idempotency String, identity "owned-outputs.edit-model.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial DenialBinding,
    handler identity "owned-outputs.edit-model.v1", program required, inspection_disclosure predecessor::inspection::Disclosure, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, Denial> {
        if input.value > 1000 {
            return HandlerResult::DomainDenied(Denial::ValueOutOfRange);
        }
        let result = (|| {
            let entity = reader.resolve_entity(ModelKey::at("ModelKey"), input.key.clone())?;
            reader.field(&entity, ModelValue::at("ModelValue"))?;
            predecessor::content::prepare(reader, &entity)?;
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
        target: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denial> {
        let result = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|entity| {
                writer
                    .write_field(&entity, ModelValue::at("ModelValue"), input.value)
                    .map_err(HandlerExecutionDenial::new)?;
                predecessor::content::write(writer, &entity, input.value)
                    .map_err(HandlerExecutionDenial::new)
            });
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<PatchModel>(
            PatchModel::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
