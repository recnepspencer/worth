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
                ApplicationOperationBodyScopeField::Assign { .. } => None,
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
        match reader.body_scope_target() {
            Ok(target) => HandlerResult::Completed(WorthQueryOperationBodyDecision { target }),
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
        for step in self.body.scope_fields() {
            if let ApplicationOperationBodyScopeField::Assign { field, value, .. } = step {
                if let Err(cause) = writer.body_field(&target, field, value) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
        }
        HandlerResult::Completed(())
    }
}
