//! Ordinary scalar mutation captures the original binding/input/publication.
use super::super::{
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
    schema::*,
};
use super::{queries::inspection, Value};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Edit {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "numeric-scalar.edit-input.v1" });
worth_query_operation!(pub SetScalar for NumericSchema, input InputBinding);
worth_query_operation_reads!(SetScalar => [Key, Value]);
worth_query_operation_writes!(SetScalar => [Value]);
fn key(input: &Edit) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Edit, schema NumericSchema,
    identity "numeric-scalar.edit-binding.v1", input InputBinding, operation SetScalar, result ResultBinding,
    idempotency String, identity "numeric-scalar.edit-key.v1",
    decision WorthQueryInvariantMutationTarget<NumericSchema, Number>, denial DenialBinding,
    handler identity "numeric-scalar.edit-handler.v1", program required, inspection_disclosure Disclosure,
    outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,);
pub struct Disclosure;
impl ApplicationSourceDisclosureBinding<NumericSchema> for Disclosure {
    const IDENTITY: &'static str = "numeric-scalar.original-source.v1";
    type Source = Binding;
    type Disclosure = inspection::Binding;
    fn disclosure_parameters(
        _: &Edit,
    ) -> Result<
        ApplicationQueryParameterSet<inspection::Query>,
        ApplicationSourceDisclosureParameterDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
}
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        let target = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(target) => target,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Err(cause) = reader.field(&target, Value::reference()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&target) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let result = writer
            .projected_entity(&target)
            .and_then(|target| writer.write_field(&target, Value::reference(), input.value));
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            SetScalar::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(SetScalar::reference(), 8)
        .operation_read_field(SetScalar::reference(), Key::reference())
        .operation_read_field(SetScalar::reference(), Value::reference())
        .operation_write(SetScalar::reference(), Value::reference())
        .application_mutation_binding::<Binding>()
}
