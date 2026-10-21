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
        let mut anchors = decision.path_links.targets.into_iter();
        let mut child_index = 0;
        let mut signing_target = None;
        for (root_index, creation) in self
            .body
            .creation()
            .into_iter()
            .flat_map(|c| c.creation_roots())
            .enumerate()
        {
            if root_index == 0
                && matches!(
                    decision.creation_disposition,
                    reuse::CreationDisposition::Reuse
                )
            {
                for _ in creation
                    .fields()
                    .iter()
                    .filter(|f| f.input_member().is_some())
                {
                    let _ = input_values
                        .as_mut()
                        .and_then(Iterator::next)
                        .expect("validated reused initializer");
                }
                continue;
            }
            let created =
                match writer.body_create_in_scope(&target, creation.entity(), creation.key()) {
                    Ok(created) => created,
                    Err(cause) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
                };
            if let Err(cause) = super::creation_initializers::initialize(
                creation,
                &created,
                &mut input_values,
                &mut source_values,
                decision.values.as_ref(),
                writer,
            ) {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
            for output in &self.outputs {
                if root_index == 0 && matches!(output.target, worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Creation { .. })
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
                let linked = if matches!(link.source, worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkSource::Principal {..}) {
                    let principal = match writer.projected_entity(decision.principal_link.as_ref().expect("exact admitted principal target")) {
                        Ok(target) => target,
                        Err(cause) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                    };
                    writer.body_creation_link(&principal, &created, link)
                } else { writer.body_creation_link(&target, &created, link) };
                if let Err(cause) = linked {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            for path in creation.path_links() {
                let anchor = match writer
                    .projected_entity(&anchors.next().expect("prepared exact existing anchor"))
                {
                    Ok(anchor) => anchor,
                    Err(cause) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
                    }
                };
                if let Err(cause) = writer.body_creation_link(&anchor, &created, &path.link) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
            }
            for child in creation.children() {
                let site = &child.creation;
                // Creation partition remains the original admitted Existing scope,
                // not the new parent. The parent is used only as a retained link endpoint.
                let nested = match writer.body_create_in_scope(&target, site.entity(), site.key()) {
                    Ok(created) => created,
                    Err(cause) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
                    }
                };
                if let Err(cause) = super::creation_initializers::initialize(
                    site,
                    &nested,
                    &mut input_values,
                    &mut source_values,
                    decision.values.as_ref(),
                    writer,
                ) {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                }
                let custody = children.as_mut().expect("funded child sites");
                let parent_index = self.child_parents[child_index];
                let parent = if parent_index == 0 {
                    &created
                } else {
                    &custody.targets[parent_index - 1]
                };
                for link in site.links() {
                    if let Err(cause) = writer.body_creation_link(parent, &nested, link) {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
                    }
                }
                custody.targets.push(nested);
                child_index += 1;
            }
            if creation.signing().is_some() {
                signing_target = Some(created);
            }
        }
        if let Some(emission) = decision.emission
            && let Err(cause) = writer.body_external_emission(emission)
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        // The deferred provenance slot owns the remaining representation budget.
        // Reserve it only after every independent ordinary effect is assembled.
        if let Some(created) = signing_target
            && let Err(cause) = writer.body_signing_slot(
                &created,
                self.body
                    .creation()
                    .and_then(|root| root.signing())
                    .expect("installed signed root"),
            )
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        HandlerResult::Completed(())
    }
}
