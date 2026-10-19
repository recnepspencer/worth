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
    pub(super) values: Vec<super::value::InstalledBodyValue>,
    pub(super) external_emission: Option<std::sync::Arc<worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalEffectRecipe>>,
    pub(super) marker: PhantomData<fn() -> (Schema, Binding)>,
    pub(super) input_bindings:
        Vec<worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding>,
    pub(super) outputs: Vec<super::output::InstalledBodyOutput>,
    pub(super) selection_reads: Vec<
        worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget,
    >,
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
                | ApplicationOperationBodyScopeField::AssignValue { .. }
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
        let values = match reader.body_values(&self.values) {
            Ok(values) => values,
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
            if let ApplicationOperationBodyScopeField::AssignValue { field, value, .. } = step
                && let Err(cause) = writer.body_field(
                    &target,
                    field,
                    decision
                        .values
                        .as_ref()
                        .expect("installed computed value")
                        .value(value),
                )
            {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
            if let ApplicationOperationBodyScopeField::Assign { field, value, .. } = step
                && let Err(cause) = writer.body_field(&target, field, value)
            {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
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
            if let ApplicationOperationBodyScopeField::ClearOptional { field, .. } = step
                && let Err(cause) = writer.body_optional_field(&target, field, None)
            {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
        }
        for output in &self.outputs {
            if matches!(output.target, worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Scope)
                && let Err(cause) = writer.body_output(&target, output.role.clone()) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Related { relation, from, to } = &output.target {
                let selected = decision.selections.selected(relation, from, to).as_ref().expect("required output validated at decision");
                let preserved = match writer.projected_entity(&selected.target) {
                    Ok(target) => target,
                    Err(cause) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                };
                if let Err(cause) = writer.body_output(&preserved, output.role.clone()) {
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
                    ApplicationOperationBodyCreatedField::Computed { value, .. } => {
                        std::borrow::Cow::Borrowed(
                            decision
                                .values
                                .as_ref()
                                .expect("installed computed value")
                                .value(value),
                        )
                    }
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
                    | ApplicationOperationBodyCreatedField::CapturedPathField { .. }
                    | ApplicationOperationBodyCreatedField::RelatedField { .. }
                    | ApplicationOperationBodyCreatedField::PrincipalIdentity { .. } => {
                        let Some(source) = source_values
                            .next()
                            .expect("one captured source per installed initializer")
                        else {
                            continue;
                        };
                        let (source, reservation) = source.into_parts();
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
                if matches!(output.target, worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Creation { .. })
                    && let Err(cause) = writer.body_output(&created, output.role.clone()) {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
            }
            for link in creation.links() {
                if link.posture == worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkPosture::ReplaceSingle
                    && let Some(predecessor) = decision.selections.selected(&link.relation, &link.from, &link.to) {
                        let predecessor = match writer.projected_entity(&predecessor.target) {
                            Ok(target) => target,
                            Err(cause) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                        };
                        if let Err(cause) = writer.body_creation_unlink(&target, &predecessor, link) {
                            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                        }
                    }
                if let Err(cause) = writer.body_creation_link(&target, &created, link) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            if let Some(signing) = creation.signing()
                && let Err(cause) = writer.body_signing_slot(&created, signing)
            {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
        }
        if let Some(emission) = decision.emission
            && let Err(cause) = writer.body_external_emission(emission)
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        HandlerResult::Completed(())
    }
}
