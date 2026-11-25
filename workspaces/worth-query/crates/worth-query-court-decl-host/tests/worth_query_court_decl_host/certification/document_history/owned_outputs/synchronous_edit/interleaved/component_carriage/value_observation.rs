//! An authored observation selects its Model or a genuine produced child value.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum ValueSource {
    Model,
    Child(String),
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub model: String,
    pub source: ValueSource,
}
worth_query_structured_value_binding!(pub Input for Intent { identity:"owned-outputs.component-value-observation.v1" });
worth_query_operation!(pub ObserveComponentValue for Schema, input Input; access {
    read_fields:[ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), ChildKey::at("ChildKey"), ChildValue::at("ChildValue")];
});
fn scope(input: &Intent) -> String {
    input.model.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.component-value-observation.v1", input Input, operation ObserveComponentValue, result fixture::StageResultBinding,
    idempotency String, identity "owned-outputs.component-value-observation.v1", decision fixture::StageResult, denial super::DenialBinding,
    handler identity "owned-outputs.component-value-observation.v1", program required, outputs NoApplicationMutationOutputs,
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
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<fixture::StageResult, super::Denial> {
        let read = (|| {
            let value = match &input.source {
                ValueSource::Model => {
                    let row =
                        reader.resolve_entity(ModelKey::at("ModelKey"), input.model.clone())?;
                    reader.field(&row, ModelValue::at("ModelValue"))?.unwrap()
                }
                ValueSource::Child(key) => {
                    let row = reader.resolve_entity(ChildKey::at("ChildKey"), key.clone())?;
                    reader.field(&row, ChildValue::at("ChildValue"))?.unwrap()
                }
            };
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
        .declared_operation::<ObserveComponentValue>(
            ObserveComponentValue::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
