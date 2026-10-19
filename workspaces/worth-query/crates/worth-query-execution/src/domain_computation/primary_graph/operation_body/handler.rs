//! Lower installed literal guards/writes through the native handler phase owners.
use super::super::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use super::{WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial};
#[path = "handler/candidate.rs"]
mod candidate;
#[path = "handler/creation_initializers.rs"]
mod creation_initializers;
#[path = "handler/scope_expectations.rs"]
mod scope_expectations;
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
    pub(super) values: Vec<super::value::InstalledBodyValue>,
    pub(super) external_emission: Option<std::sync::Arc<worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalEffectRecipe>>,
    pub(super) marker: PhantomData<fn() -> (Schema, Binding)>,
    pub(super) input_bindings:
        Vec<worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding>,
    pub(super) outputs: Vec<super::output::InstalledBodyOutput>,
    pub(super) selection_reads: Vec<
        worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget,
    >,
    pub(super) retirement_inputs:Vec<usize>,
    pub(super) child_parents: Vec<usize>,
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
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Binding::Decision, Binding::Denial> {
        match scope_expectations::evaluate(&self.body, reader) {
            HandlerResult::Completed(()) => {}
            HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
            HandlerResult::ExecutionDenied(cause) => return HandlerResult::ExecutionDenied(cause),
            HandlerResult::Cancelled => return HandlerResult::Cancelled,
            HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
        }
        let findings = match reader.body_scope_requirements(&self.body) {
            Ok(findings) => findings,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Some(unmet) = findings.findings().iter().find(|finding| {
            finding.status() == super::WorthQueryOperationBodyRequirementStatus::Unmet
        }) {
            return HandlerResult::DomainDenied(match unmet.identity() {
                Some(identity) => WorthQueryOperationBodyDenial::DeclaredRequirementUnmet {
                    identity: identity.to_owned(),
                },
                None => {
                    let field = unmet.field().expect("intrinsic presence targets a field");
                    WorthQueryOperationBodyDenial::RequirementUnmet {
                        entity: field.entity().to_owned(),
                        aspect: field.aspect().to_owned(),
                        field: field.field().to_owned(),
                    }
                }
            });
        }
        drop(findings);
        let selections = match reader.body_selections(&self.selection_reads) {
            Ok(selections) => selections,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Err(cause) = super::output::require_related_outputs(&self.outputs, &selections) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        let sources = match reader.body_created_sources(&self.body, &selections) {
            HandlerResult::Completed(sources) => sources,
            HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
            HandlerResult::ExecutionDenied(cause) => return HandlerResult::ExecutionDenied(cause),
            HandlerResult::Cancelled => return HandlerResult::Cancelled,
            HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
        };
        let inputs = match reader.body_inputs(&self.body, &self.input_bindings) {
            HandlerResult::Completed(inputs) => inputs,
            HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
            HandlerResult::ExecutionDenied(cause) => return HandlerResult::ExecutionDenied(cause),
            HandlerResult::Cancelled => return HandlerResult::Cancelled,
            HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
        };
        match super::input_expectation::evaluate(&self.body, inputs.as_ref(), reader) {
            HandlerResult::Completed(()) => {}
            HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
            HandlerResult::ExecutionDenied(cause) => return HandlerResult::ExecutionDenied(cause),
            HandlerResult::Cancelled => return HandlerResult::Cancelled,
            HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
        }
        let creation_disposition =
            match reader.body_creation_disposition(&self.body, inputs.as_ref(), &selections) {
                Ok(disposition) => disposition,
                Err(cause) => return HandlerResult::ExecutionDenied(cause),
            };
        let retirements =
            match reader.body_retirements(&self.body, &self.retirement_inputs, inputs.as_ref()) {
                HandlerResult::Completed(value) => value,
                HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
                HandlerResult::ExecutionDenied(cause) => {
                    return HandlerResult::ExecutionDenied(cause)
                }
                HandlerResult::Cancelled => return HandlerResult::Cancelled,
                HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
            };
        let values = match reader.body_values(&self.values) {
            Ok(values) => values,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let path_writes = match reader.body_path_writes(&self.body) {
            Ok(targets) => targets,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let path_links = match reader.body_path_links(&self.body) {
            Ok(anchors) => anchors,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let principal_link = match reader.body_principal_link(&self.body) {
            Ok(target) => target,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let children = match reader.body_child_custody(&self.body) {
            Ok(children) => children,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let references = match reader.body_references(&self.body, inputs.as_ref()) {
            Ok(references) => references,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let emission = match self
            .external_emission
            .as_ref()
            .map(|recipe| {
                reader.body_external_emission(std::sync::Arc::clone(recipe), input, values.as_ref())
            })
            .transpose()
        {
            Ok(emission) => emission,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.body_scope_target(&self.body) {
            Ok(target) => HandlerResult::Completed(WorthQueryOperationBodyDecision {
                target,
                inputs,
                sources,
                selections,
                emission,
                values,
                retirements,
                path_writes,
                path_links,
                principal_link,
                references,
                creation_disposition,
                children,
            }),
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
        self.build_body_candidate(decision, writer)
    }
}
