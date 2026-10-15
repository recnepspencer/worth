//! Lower installed literal guards/writes through the native handler phase owners.
use super::super::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use super::{BodyFieldExpectation, WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyProgram,
    ApplicationOperationBodyScopeField,
};
use worth_query_installation::facade::ApplicationSchema;

type Scope<Schema, Binding> = <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding
    as ApplicationMutationScopeBinding<Schema>>::Scope;

pub(in crate::domain_computation::primary_graph) struct OperationBodyHandler<Schema, Binding> {
    pub(super) body: ApplicationOperationBodyProgram,
    pub(super) marker: PhantomData<fn() -> (Schema, Binding)>,
    pub(super) input_bindings:
        Vec<worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding>,
    pub(super) outputs: Vec<super::output::InstalledBodyOutput>,
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
            if matches!(
                step,
                ApplicationOperationBodyScopeField::RequirePresent { .. }
            ) {
                continue;
            }
            let expected = match step {
                ApplicationOperationBodyScopeField::Expect { value, .. } => {
                    BodyFieldExpectation::Equals(value)
                }
                ApplicationOperationBodyScopeField::RequirePresent { .. } => {
                    BodyFieldExpectation::Present
                }
                ApplicationOperationBodyScopeField::Assign { .. }
                | ApplicationOperationBodyScopeField::AssignInput { .. }
                | ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                | ApplicationOperationBodyScopeField::AssignOptionalInput { .. }
                | ApplicationOperationBodyScopeField::ClearOptional { .. } => {
                    BodyFieldExpectation::Any
                }
            };
            let matches = match reader.body_scope_field(step.field(), expected) {
                Ok(value) => value,
                Err(cause) => return HandlerResult::ExecutionDenied(cause),
            };
            if !matches {
                let field = step.field();
                return HandlerResult::DomainDenied(
                    if matches!(
                        step,
                        ApplicationOperationBodyScopeField::RequirePresent { .. }
                    ) {
                        WorthQueryOperationBodyDenial::RequirementUnmet {
                            entity: field.entity().to_owned(),
                            aspect: field.aspect().to_owned(),
                            field: field.field().to_owned(),
                        }
                    } else {
                        WorthQueryOperationBodyDenial::SourceMismatch {
                            entity: field.entity().to_owned(),
                            aspect: field.aspect().to_owned(),
                            field: field.field().to_owned(),
                        }
                    },
                );
            }
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
        let mut sources = Vec::new();
        if let Some(creation) = self.body.creation() {
            for step in creation.fields() {
                let (source, captured) = if let Some(source) = step.source_field() {
                    (source, reader.body_scope_value(source))
                } else if let Some(source) = step.principal_source() {
                    (source, reader.body_principal_value(source))
                } else {
                    continue;
                };
                match captured {
                    Ok(Some(value)) => sources.push(value),
                    Ok(None) => {
                        return HandlerResult::DomainDenied(
                            WorthQueryOperationBodyDenial::SourceMismatch {
                                entity: source.entity().to_owned(),
                                aspect: source.aspect().to_owned(),
                                field: source.field().to_owned(),
                            },
                        );
                    }
                    Err(cause) => return HandlerResult::ExecutionDenied(cause),
                }
            }
        }
        let inputs = match reader.body_inputs(&self.body, &self.input_bindings) {
            HandlerResult::Completed(inputs) => inputs,
            HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
            HandlerResult::ExecutionDenied(cause) => return HandlerResult::ExecutionDenied(cause),
            HandlerResult::Cancelled => return HandlerResult::Cancelled,
            HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
        };
        match reader.body_scope_target(&self.body) {
            Ok(target) => HandlerResult::Completed(WorthQueryOperationBodyDecision {
                target,
                inputs,
                sources,
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
        let target = match writer.projected_entity(&decision.target) {
            Ok(target) => target,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
        };
        let mut prepared_inputs = decision.inputs;
        let mut input_values = prepared_inputs
            .as_mut()
            .map(|inputs| inputs.values.drain(..));
        let mut source_values = decision.sources.into_iter();
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
                    .expect("one prepared value per installed input assignment")
                    .expect("required input assignment is present");
                if let Err(cause) = writer.body_input_field(&target, field, value) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            if let ApplicationOperationBodyScopeField::SetOptionalInput { field, .. } = step {
                let value = input_values
                    .as_mut()
                    .and_then(Iterator::next)
                    .expect("one prepared value per installed optional set")
                    .expect("flat optional Set input is present");
                if let Err(cause) = writer.body_optional_field(&target, field, Some(value)) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            if let ApplicationOperationBodyScopeField::AssignOptionalInput { field, .. } = step {
                let value = input_values
                    .as_mut()
                    .and_then(Iterator::next)
                    .expect("one prepared option per installed optional assignment");
                if let Err(cause) = writer.body_optional_field(&target, field, value) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            if let ApplicationOperationBodyScopeField::ClearOptional { field, .. } = step {
                if let Err(cause) = writer.body_optional_field(&target, field, None) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
        }
        for output in &self.outputs {
            if matches!(output.target, worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Scope) {
                if let Err(cause) = writer.body_output(&target, output.role.clone()) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
        }
        if let Some(creation) = self.body.creation() {
            let created =
                match writer.body_create_in_scope(&target, creation.entity(), creation.key()) {
                    Ok(created) => created,
                    Err(cause) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
                };
            for step in creation.fields() {
                let mut source_reservation = None;
                let value = match step {
                    ApplicationOperationBodyCreatedField::Literal { value, .. } => {
                        std::borrow::Cow::Borrowed(value)
                    }
                    ApplicationOperationBodyCreatedField::Input { .. } => std::borrow::Cow::Owned(
                        input_values
                            .as_mut()
                            .and_then(Iterator::next)
                            .expect("one prepared value per installed input initializer")
                            .expect("required initializer input is present"),
                    ),
                    ApplicationOperationBodyCreatedField::OptionalInput { .. } => {
                        let Some(value) = input_values
                            .as_mut()
                            .and_then(Iterator::next)
                            .expect("one prepared option per installed optional initializer")
                        else {
                            continue;
                        };
                        std::borrow::Cow::Owned(value)
                    }
                    ApplicationOperationBodyCreatedField::ScopeField { .. }
                    | ApplicationOperationBodyCreatedField::PresentScopeField { .. }
                    | ApplicationOperationBodyCreatedField::CapturedScopeField { .. }
                    | ApplicationOperationBodyCreatedField::PrincipalIdentity { .. } => {
                        let (source, reservation) = source_values
                            .next()
                            .expect("one captured source per installed initializer")
                            .into_parts();
                        source_reservation = Some(reservation);
                        std::borrow::Cow::Owned(source)
                    }
                    ApplicationOperationBodyCreatedField::SigningProvenance { .. } => continue,
                };
                if let Err(cause) = writer.body_initialize_field(&created, step.field(), value) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
                drop(source_reservation);
            }
            for output in &self.outputs {
                if matches!(output.target, worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Creation { .. }) {
                    if let Err(cause) = writer.body_output(&created, output.role.clone()) {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
                }
            }
            for link in creation.links() {
                if let Err(cause) = writer.body_creation_link(&target, &created, link) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            if let Some(signing) = creation.signing() {
                if let Err(cause) = writer.body_signing_slot(&created, signing) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
        }
        HandlerResult::Completed(())
    }
}
