//! Lower installed literal guards/writes through the native handler phase owners.
use super::super::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use super::{WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField,
};
use worth_query_installation::facade::ApplicationSchema;

type Scope<Schema, Binding> = <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding
    as ApplicationMutationScopeBinding<Schema>>::Scope;

pub(in crate::domain_computation::primary_graph) struct OperationBodyHandler<Schema, Binding> {
    pub(super) body: ApplicationOperationBodyProgram,
    pub(super) marker: PhantomData<fn() -> (Schema, Binding)>,
    pub(super) input_bindings:
        Vec<worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding>,
}

impl<Schema, Binding> OperationBodyHandler<Schema, Binding> {
    pub(in crate::domain_computation::primary_graph) fn body(
        &self,
    ) -> &ApplicationOperationBodyProgram {
        &self.body
    }
}

impl<Schema, Binding> OperationHandler<Schema, Binding> for OperationBodyHandler<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<
        Schema,
        Result = (),
        Denial = WorthQueryOperationBodyDenial,
        Decision = WorthQueryOperationBodyDecision<Schema, Scope<Schema, Binding>>,
    >,
{
    fn decide(
        &self,
        _: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Binding::Decision, Binding::Denial> {
        for step in self.body.scope_fields() {
            let expected = match step {
                ApplicationOperationBodyScopeField::Expect { value, .. } => Some(value),
                ApplicationOperationBodyScopeField::Assign { .. }
                | ApplicationOperationBodyScopeField::AssignInput { .. } => None,
            };
            let matches = match reader.body_scope_field(step.field(), expected) {
                Ok(value) => value,
                Err(cause) => return HandlerResult::ExecutionDenied(cause),
            };
            if !matches {
                return HandlerResult::DomainDenied(
                    WorthQueryOperationBodyDenial::SourceMismatch {
                        entity: step.field().entity().to_owned(),
                        aspect: step.field().aspect().to_owned(),
                        field: step.field().field().to_owned(),
                    },
                );
            }
        }
        let inputs = match reader.body_inputs(&self.body, &self.input_bindings) {
            HandlerResult::Completed(inputs) => inputs,
            HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
            HandlerResult::ExecutionDenied(cause) => return HandlerResult::ExecutionDenied(cause),
            HandlerResult::Cancelled => return HandlerResult::Cancelled,
            HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
        };
        match reader.body_scope_target() {
            Ok(target) => {
                HandlerResult::Completed(WorthQueryOperationBodyDecision { target, inputs })
            }
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn candidate_requirements(
        &self,
        _: &Binding::Input,
        _: &Binding::Decision,
    ) -> ApplicationCandidateRequirements {
        Binding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &Binding::Input,
        decision: Binding::Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<(), Binding::Denial> {
        let target = match writer.projected_entity(&decision.target) {
            Ok(target) => target,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let mut prepared_inputs = decision.inputs;
        let mut input_values = prepared_inputs
            .as_mut()
            .map(|inputs| inputs.values.drain(..));
        for step in self.body.scope_fields() {
            if let ApplicationOperationBodyScopeField::Assign { field, value, .. } = step {
                if let Err(cause) = writer.body_field(&target, field, value) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            if let ApplicationOperationBodyScopeField::AssignInput { field, .. } = step {
                let value = input_values
                    .as_mut()
                    .and_then(Iterator::next)
                    .expect("one prepared value per installed input assignment");
                if let Err(cause) = writer.body_input_field(&target, field, value) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
        }
        if let Some(creation) = self.body.creation() {
            let created =
                match writer.body_create_in_scope(&target, creation.entity(), creation.key()) {
                    Ok(created) => created,
                    Err(cause) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
                    }
                };
            for step in creation.fields() {
                let value = match step.literal_value() {
                    Some(value) => std::borrow::Cow::Borrowed(value),
                    None => std::borrow::Cow::Owned(
                        input_values
                            .as_mut()
                            .and_then(Iterator::next)
                            .expect("one prepared value per installed input initializer"),
                    ),
                };
                if let Err(cause) = writer.body_initialize_field(&created, step.field(), value) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
        }
        HandlerResult::Completed(())
    }
}
