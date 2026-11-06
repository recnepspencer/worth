//! Ordinary optional-operand authoring with its own original source disclosure.
use super::super::{
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
    schema::*,
};
use super::{query, Bias, InputFacts, Sibling};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct AspectEdit {
    pub key: String,
    pub action: Action,
}
#[derive(Clone, Copy, Debug, worth_foundational::facade::WorthValue)]
pub enum Action {
    Set { bias: u64, sibling: u64 },
    Remove,
    IncompleteRemoval,
    ClearThenWrite,
    WriteThenClear,
    WrongAspect,
}
worth_query_structured_value_binding!(pub InputBinding for AspectEdit { identity: "numeric-aspect.edit-input.v1" });
worth_query_operation!(pub EditInputAspect for NumericSchema, input InputBinding);

worth_query_decl::facade::// Deliberately overclaims a Rust capability: the installed program still grants only InputFacts.
worth_query_operation_removes_aspects!(EditInputAspect => [InputFacts, NumberFacts]);
fn key(input: &AspectEdit) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(
    pub Binding for AspectEdit, schema NumericSchema,
    identity "numeric-aspect.edit-binding.v1", input InputBinding,
    operation EditInputAspect, result ResultBinding,
    idempotency String, identity "numeric-aspect.edit-key.v1",
    decision WorthQueryInvariantMutationTarget<NumericSchema, Number>, denial DenialBinding,
    handler identity "numeric-aspect.edit-handler.v1",
    program required,
    inspection_disclosure Disclosure,
    outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key,
    field Key::at("Key"), value key,
);

pub struct Disclosure;
impl ApplicationSourceDisclosureBinding<NumericSchema> for Disclosure {
    const IDENTITY: &'static str = "numeric-aspect.edit-source.v1";
    type Source = Binding;
    type Disclosure = query::Binding;
    fn disclosure_parameters(
        _: &AspectEdit,
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
            EditInputAspect::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(EditInputAspect::reference(), 8)
        .operation_read_field(EditInputAspect::reference(), Key::at("Key"))
        .operation_read_field(EditInputAspect::reference(), Ceiling::at("Ceiling"))
        .operation_read_field(EditInputAspect::reference(), Bias::at("Bias"))
        .operation_read_field(EditInputAspect::reference(), Sibling::at("Sibling"))
        .operation_write(EditInputAspect::reference(), Bias::at("Bias"))
        .operation_write(EditInputAspect::reference(), Sibling::at("Sibling"))
        .operation_remove_aspect(EditInputAspect::reference(), InputFacts::reference())
        .application_mutation_binding::<Binding>()
}

pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &AspectEdit,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        let root = match reader.resolve_entity(Key::at("Key"), input.key.clone()) {
            Ok(root) => root,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&root, Ceiling::at("Ceiling")) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        // Complete the actual optional read before writing; propagate unavailable reads.
        if let Err(error) = reader.field(&root, Bias::at("Bias")) {
            return HandlerResult::ExecutionDenied(error);
        }
        if !matches!(input.action, Action::IncompleteRemoval)
            && let Err(error) = reader.field(&root, Sibling::at("Sibling"))
        {
            return HandlerResult::ExecutionDenied(error);
        }
        match reader.mutation_target(&root) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &AspectEdit,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let target = match writer.projected_entity(&target) {
            Ok(target) => target,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let effect = match input.action {
            Action::Set { bias, sibling } => writer
                .write_field(&target, Bias::at("Bias"), bias)
                .and_then(|()| writer.write_field(&target, Sibling::at("Sibling"), sibling)),
            Action::Remove | Action::IncompleteRemoval => {
                writer.remove_aspect(&target, InputFacts::reference())
            }
            Action::ClearThenWrite => writer
                .remove_aspect(&target, InputFacts::reference())
                .and_then(|()| writer.write_field(&target, Bias::at("Bias"), 99)),
            Action::WrongAspect => writer.remove_aspect(&target, NumberFacts::reference()),
            Action::WriteThenClear => writer
                .write_field(&target, Bias::at("Bias"), 99)
                .and_then(|()| writer.remove_aspect(&target, InputFacts::reference())),
        };
        match effect {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
