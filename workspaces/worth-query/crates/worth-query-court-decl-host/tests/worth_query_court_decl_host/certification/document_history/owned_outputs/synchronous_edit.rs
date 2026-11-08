//! A declared synchronous edit over the generic two-family fixture.
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
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "owned-outputs.synchronous-edit-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.synchronous-edit-denial.v1" });
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.synchronous-edit-model.v1" });
worth_query_operation!(pub SynchronousPatchModel for Schema, input Input; access { read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue")]; writes: [ModelValue::at("ModelValue")]; });
fn scope(input: &Intent) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.synchronous-edit-model.v1", input Input, operation SynchronousPatchModel, result ResultBinding,
    idempotency String, identity "owned-outputs.synchronous-edit-model.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial DenialBinding,
    handler identity "owned-outputs.synchronous-edit-model.v1", program required,
    coupled_outputs |visitor| { visitor.required::<program::StartFirst>(); visitor.required::<program::StartSecond>(); },
    outputs NoApplicationMutationOutputs,
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
        let result = writer.projected_entity(&target).and_then(|entity| {
            writer.write_field(&entity, ModelValue::at("ModelValue"), input.value)
        });
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    graph::unstable::query::declare(schema)
        .declared_operation::<SynchronousPatchModel>(
            SynchronousPatchModel::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}

macro_rules! demand {
    ($connection:ident, $family:ident) => {
        impl WorthQueryApplicationRequiredOutputSource<Schema, program::$connection> for Binding {
            fn demand_from_source(
                input: &Intent,
            ) -> Result<$family::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok($family::Demand(input.key.clone()))
            }
        }
    };
}
demand!(StartFirst, first);
demand!(StartSecond, second);
mod cold;
mod corruption;
mod courts;
mod failure;
mod initial;

pub(super) mod new_source;

pub(super) mod graph;
