//! Ordinary optional-operand authoring with its own original source disclosure.
use super::{
    causal_inspection::query,
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct OperandEdit {
    pub key: String,
    pub value: Option<u64>,
}
worth_query_structured_value_binding!(pub InputBinding for OperandEdit { identity: "numeric-correction.operand-input.v1" });
worth_query_operation!(pub EditOperand for NumericSchema, input InputBinding);
worth_query_operation_reads!(EditOperand => [Key, Ceiling, Operand]);
worth_query_operation_writes!(EditOperand => [Operand]);
fn key(input: &OperandEdit) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(
    pub OperandBinding for OperandEdit, schema NumericSchema,
    identity "numeric-correction.operand-binding.v1", input InputBinding,
    operation EditOperand, result ResultBinding,
    idempotency String, identity "numeric-correction.operand-key.v1",
    decision WorthQueryInvariantMutationTarget<NumericSchema, Number>, denial DenialBinding,
    handler identity "numeric-correction.operand-handler.v1",
    program required,
    inspection_disclosure Disclosure,
    outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,
);

pub struct Disclosure;
impl ApplicationSourceDisclosureBinding<NumericSchema> for Disclosure {
    const IDENTITY: &'static str = "numeric-correction.operand-source.v1";
    type Source = OperandBinding;
    type Disclosure = query::Binding;
    fn disclosure_parameters(
        _: &OperandEdit,
    ) -> Result<
        ApplicationQueryParameterSet<query::Query>,
        ApplicationSourceDisclosureParameterDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            EditOperand::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(EditOperand::reference(), 8)
        .operation_read_field(EditOperand::reference(), Key::reference())
        .operation_read_field(EditOperand::reference(), Ceiling::reference())
        .operation_read_field(EditOperand::reference(), Operand::reference())
        .operation_write(EditOperand::reference(), Operand::reference())
        .application_mutation_binding::<OperandBinding>()
}

pub struct Handler;
impl OperationHandler<NumericSchema, OperandBinding> for Handler {
    fn decide(
        &self,
        input: &OperandEdit,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, OperandBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        let root = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(root) => root,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&root, Ceiling::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        // Complete the actual optional read before writing; propagate unavailable reads.
        if let Err(error) = reader.field(&root, Operand::reference()) {
            return HandlerResult::ExecutionDenied(error);
        }
        match reader.mutation_target(&root) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &OperandEdit,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, OperandBinding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let target = match writer.projected_entity(&target) {
            Ok(target) => target,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.write_optional_field(&target, Operand::reference(), input.value) {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
