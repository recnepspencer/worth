//! An ordinary declared mutation writes the selected domain field.
use super::{
    super::{
        edit::{DenialBinding, Edited, MissingValue, ResultBinding},
        schema::*,
    },
    Binary, Canonical, CanonicalCounter, Delegated,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub enum Encoding {
    Binary,
    Canonical,
    Delegated,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Write {
    pub key: String,
    pub value: u64,
    pub encoding: Encoding,
}
worth_query_structured_value_binding!(pub InputBinding for Write { identity: "domain-write.input.v1" });
worth_query_operation!(pub WriteOperation for NumericSchema, input InputBinding);
worth_query_operation_reads!(WriteOperation => [Key, Binary, Canonical, Delegated]);
worth_query_operation_writes!(WriteOperation => [Binary, Canonical, Delegated]);
pub fn key(input: &Write) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(
    pub Binding for Write, schema NumericSchema,
    identity "domain-write.binding.v1", input InputBinding,
    operation WriteOperation, result ResultBinding,
    idempotency String, identity "domain-write.command.v1",
    decision WorthQueryInvariantMutationTarget<NumericSchema, Number>, denial DenialBinding,
    handler identity "domain-write.handler.v1",
    outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,
);
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Write,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        let decision = (|| {
            let root = reader.resolve_entity(Key::reference(), input.key.clone())?;
            match input.encoding {
                Encoding::Binary => {
                    reader.field(&root, Binary::reference())?;
                }
                Encoding::Canonical => {
                    reader.field(&root, Canonical::reference())?;
                }
                Encoding::Delegated => {
                    reader.field(&root, Delegated::reference())?;
                }
            }
            reader.mutation_target(&root)
        })();
        match decision {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Write,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let result = writer
            .projected_entity(&target)
            .and_then(|target| match input.encoding {
                Encoding::Binary => writer.write_field(&target, Binary::reference(), input.value),
                Encoding::Canonical => writer.write_field(
                    &target,
                    Canonical::reference(),
                    CanonicalCounter(input.value),
                ),
                Encoding::Delegated => {
                    writer.write_field(&target, Delegated::reference(), input.value)
                }
            });
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
