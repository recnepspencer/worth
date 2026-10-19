//! Native candidate assembly consumes one sealed body decision; no graph rereads.
use super::super::{path_value, retirement, reuse};
use super::*;
impl<Schema, Binding> OperationBodyHandler<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<
        Schema,
        Result = (),
        Denial = WorthQueryOperationBodyDenial,
        Decision = WorthQueryOperationBodyDecision<Schema, Scope<Schema, Binding>>,
    >,
{
    pub(super) fn build_body_candidate(
        &self,
        decision: Binding::Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<(), Binding::Denial> {
        let target = match writer.projected_entity(&decision.target) {
            Ok(target) => target,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
        };
        if let Err(cause) = writer.body_references(&target, &self.body, decision.references) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        let mut prepared_inputs = decision.inputs;
        let mut children = decision.children;
        let mut input_values = prepared_inputs
            .as_mut()
            .map(|inputs| inputs.values.drain(..));
        let mut source_values = decision.sources.into_iter();
        for step in self.body.scope_fields() {
            if matches!(step, ApplicationOperationBodyScopeField::ExpectInput { .. }) {
                let _ = input_values
                    .as_mut()
                    .and_then(Iterator::next)
                    .expect("one prepared value per admitted input guard");
                continue;
            }
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
        match retirement::apply(decision.retirements, &self.body, writer) {
            HandlerResult::Completed(()) => {}
            HandlerResult::DomainDenied(cause) => return HandlerResult::DomainDenied(cause),
            HandlerResult::ExecutionDenied(cause) => return HandlerResult::ExecutionDenied(cause),
            HandlerResult::Cancelled => return HandlerResult::Cancelled,
            HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
        }
        if let Err(cause) = path_value::apply(
            decision.path_writes,
            &self.body,
            decision.values.as_ref(),
            writer,
        ) {
            return HandlerResult::ExecutionDenied(cause);
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
        if let Some(creation) = self.body.creation()
            && matches!(
                decision.creation_disposition,
                reuse::CreationDisposition::Reuse
            )
        {
            for _ in creation
                .fields()
                .iter()
                .filter(|field| field.input_member().is_some())
            {
                let _ = input_values
                    .as_mut()
                    .and_then(Iterator::next)
                    .expect("validated reused initializer");
            }
        }
        if let Some(creation) = self.body.creation()
            && matches!(
                decision.creation_disposition,
                reuse::CreationDisposition::Create
            )
        {
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
                if link.posture.replaces_selection()
                    && let Some(predecessor) =
                        decision
                            .selections
                            .selected(&link.relation, &link.from, &link.to)
                {
                    let predecessor = match writer.projected_entity(&predecessor.target) {
                        Ok(target) => target,
                        Err(cause) => {
                            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                                cause,
                            ))
                        }
                    };
                    if let Err(cause) = writer.body_creation_unlink(&target, &predecessor, link) {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
                    if link.posture.retires_predecessor()
                        && let Err(cause) =
                            writer.body_creation_retire_predecessor(&predecessor, link)
                    {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
                }
                if let Err(cause) = writer.body_creation_link(&target, &created, link) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            for (index, child) in creation.children().iter().enumerate() {
                let site = &child.creation;
                // Creation partition remains the original admitted Existing scope,
                // not the new parent. The parent is used only as a retained link endpoint.
                let nested = match writer.body_create_in_scope(&target, site.entity(), site.key()) {
                    Ok(created) => created,
                    Err(cause) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
                    }
                };
                for step in site.fields() {
                    let value = match step {
                        ApplicationOperationBodyCreatedField::Literal { value, .. } => {
                            std::borrow::Cow::Borrowed(value)
                        }
                        ApplicationOperationBodyCreatedField::Input { .. }
                        | ApplicationOperationBodyCreatedField::OptionalInput { .. } => {
                            let Some(value) = input_values
                                .as_mut()
                                .and_then(Iterator::next)
                                .expect("validated child initializer")
                            else {
                                continue;
                            };
                            std::borrow::Cow::Owned(value)
                        }
                        _ => unreachable!("installed child scalar profile"),
                    };
                    if let Err(cause) = writer.body_initialize_field(&nested, step.field(), value) {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
                }
                let custody = children.as_mut().expect("funded child sites");
                let parent_index = self.child_parents[index];
                let parent = if parent_index == 0 {
                    &created
                } else {
                    &custody.targets[parent_index - 1]
                };
                if let Err(cause) = writer.body_creation_link(parent, &nested, &site.links()[0]) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
                custody.targets.push(nested);
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
