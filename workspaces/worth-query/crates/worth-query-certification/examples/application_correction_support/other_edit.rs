//! The irreversible inverse and independent edits use an ordinary installed handler.
use super::{
    edit::{DenialBinding, Edited, MissingValue, ResultBinding, Value},
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct OtherEdit {
    pub key: String,
    pub value: Value,
}
worth_query_structured_value_binding!(pub OtherInputBinding for OtherEdit { identity: "numeric-correction.other-input.v1" });
worth_query_operation!(pub EditOther for NumericSchema, input OtherInputBinding);
worth_query_operation_reads!(EditOther => [Key, Primary, Secondary, ReadEnabled, Ceiling]);
worth_query_operation_writes!(EditOther => [Primary, Secondary, ReadEnabled, Ceiling]);
fn key(input: &OtherEdit) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(
    pub OtherBinding for OtherEdit, schema NumericSchema,
    identity "numeric-correction.other-binding.v1", input OtherInputBinding,
    operation EditOther, result ResultBinding,
    idempotency String, identity "numeric-correction.other-key.v1",
    decision WorthQueryInvariantMutationTarget<NumericSchema, Number>, denial DenialBinding,
    handler identity "numeric-correction.other-handler.v1", outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,
);
pub struct Handler;
impl OperationHandler<NumericSchema, OtherBinding> for Handler {
    fn decide(
        &self,
        input: &OtherEdit,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, OtherBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        let root = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(root) => root,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let present = match input.value {
            Value::Primary(value) => {
                match reader.field(&root, Ceiling::reference()) {
                    Ok(Some(ceiling)) if value <= ceiling => {}
                    Ok(_) => return HandlerResult::DomainDenied(MissingValue),
                    Err(cause) => return HandlerResult::ExecutionDenied(cause),
                }
                reader
                    .field(&root, Primary::reference())
                    .map(|value| value.is_some())
            }
            Value::Ceiling(_) => reader
                .field(&root, Ceiling::reference())
                .map(|value| value.is_some()),
            Value::Secondary(_) => reader
                .field(&root, Secondary::reference())
                .map(|value| value.is_some()),
            Value::ReadEnabled(_) => reader
                .field(&root, ReadEnabled::reference())
                .map(|value| value.is_some()),
            _ => return HandlerResult::DomainDenied(MissingValue),
        };
        match present {
            Ok(true) => {}
            Ok(false) => return HandlerResult::DomainDenied(MissingValue),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&root) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &OtherEdit,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, OtherBinding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let target = match writer.projected_entity(&target) {
            Ok(target) => target,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let written = match input.value {
            Value::Primary(value) => writer.write_field(&target, Primary::reference(), value),
            Value::Ceiling(value) => writer.write_field(&target, Ceiling::reference(), value),
            Value::Secondary(value) => writer.write_field(&target, Secondary::reference(), value),
            Value::ReadEnabled(value) => {
                writer.write_field(&target, ReadEnabled::reference(), value)
            }
            _ => return HandlerResult::DomainDenied(MissingValue),
        };
        match written {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
