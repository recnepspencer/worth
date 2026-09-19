use std::sync::Arc;

use crate::capability::{
    UiIntentPayloadFieldDescriptor, UiIntentPayloadFieldKind, UiIntentProjectedValue,
    UiIntentSelectionValue,
};
use crate::declaration::{UiResolvedIntentApplicationSource, UiResolvedIntentProjectionSource};
use crate::declaration::{UiResolvedIntentPayloadBinding, UiResolvedIntentPayloadSource};

use super::{
    UiIntentApplicationFactState, UiIntentApplicationInputReference, UiIntentInputBasisMaterial,
    UiIntentInputBasisView, UiIntentInputOwnerRevision, UiIntentPayloadProjectionCost,
    UiIntentPayloadStop, UiPreparedIntentPayload,
};

pub(crate) fn prepare_intent_payload(
    route: super::super::UiResolvedProductIntentRoute,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
    execution_bindings: &crate::runtime::intent_execution::FrozenIntentExecutionBindings,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    application_facts: &UiIntentApplicationFactState,
    occupancy: &super::super::operability::UiIntentOccupancyState,
) -> Result<UiPreparedIntentPayload, UiIntentPayloadStop> {
    let (graph_node, portal_declaration, declaration, source, route_resolution, evidence_reference) =
        route.into_parts();
    let basis_view =
        UiIntentInputBasisView::observe(&source, generation, mounted, application_facts)?;
    let definition = definitions.definition_at(declaration.definition());
    let binding_support = execution_bindings.support_at(declaration.definition());
    let mut projection = PayloadProjection::new(&source, definition.payload_schema(), &basis_view);
    for binding in declaration.payload() {
        projection.project(binding)?;
    }
    let (values, query_inputs, application_inputs, owner_revisions, cost) = projection.finish();
    let execution = execution_bindings
        .project_at(declaration.definition(), values)
        .map_err(UiIntentPayloadStop::PayloadProjection)?;
    let operability = super::super::operability::observe_operability_basis(
        &basis_view,
        &declaration,
        definition,
        binding_support,
        occupancy,
    );
    let basis = basis_view.seal(UiIntentInputBasisMaterial {
        source,
        query_inputs,
        application_inputs,
        owner_revisions,
        route_resolution,
        portal_declaration,
        cost,
        operability,
        evidence_reference,
    });
    Ok(UiPreparedIntentPayload::new(
        graph_node,
        definition.id(),
        declaration,
        basis,
        execution,
    ))
}

struct PayloadProjection<'basis> {
    source: &'basis super::super::routing::UiIntentProductInputSource,
    payload_schema: crate::capability::UiIntentSchema,
    basis: &'basis UiIntentInputBasisView<'basis>,
    values: Vec<UiIntentProjectedValue>,
    query_inputs: Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
    application_inputs: Vec<UiIntentApplicationInputReference>,
    owner_revisions: Vec<UiIntentInputOwnerRevision>,
    cost: UiIntentPayloadProjectionCost,
}

impl<'basis> PayloadProjection<'basis> {
    fn new(
        source: &'basis super::super::routing::UiIntentProductInputSource,
        payload_schema: crate::capability::UiIntentSchema,
        basis: &'basis UiIntentInputBasisView<'basis>,
    ) -> Self {
        Self {
            source,
            payload_schema,
            basis,
            values: Vec::new(),
            query_inputs: Vec::new(),
            application_inputs: Vec::new(),
            owner_revisions: Vec::new(),
            cost: Default::default(),
        }
    }

    fn project(
        &mut self,
        binding: &UiResolvedIntentPayloadBinding,
    ) -> Result<(), UiIntentPayloadStop> {
        let field = binding.field();
        let value = match binding.source() {
            UiResolvedIntentPayloadSource::ProjectionText(projection) => {
                self.projection_text(field, projection)?
            }
            UiResolvedIntentPayloadSource::ProjectionSelection(projection) => {
                self.projection_selection(field, projection)?
            }
            UiResolvedIntentPayloadSource::CommittedDraft => self.committed_draft(field)?,
            UiResolvedIntentPayloadSource::ConstantText(value) => {
                self.text(field, Arc::clone(value))?
            }
            UiResolvedIntentPayloadSource::ConstantBoolean(value) => {
                UiIntentProjectedValue::boolean(*value)
            }
            UiResolvedIntentPayloadSource::ConstantUnsigned64(value) => {
                UiIntentProjectedValue::unsigned64(*value)
            }
            UiResolvedIntentPayloadSource::ApplicationText(fact) => {
                self.application_text(field, fact)?
            }
            UiResolvedIntentPayloadSource::ApplicationBoolean(fact) => {
                self.application_boolean(field, fact)?
            }
            UiResolvedIntentPayloadSource::ApplicationUnsigned64(fact) => {
                self.application_unsigned64(field, fact)?
            }
        };
        self.cost.record_field();
        self.values.push(value);
        Ok(())
    }

    fn projection_text(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        projection: &UiResolvedIntentProjectionSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let input = self.current_projection(field, projection)?;
        let value = match &input {
            worth_ui_query_binding::UiProjectionInputFactReference::Scalar(fact) => fact
                .value_reference()
                .ok_or(UiIntentPayloadStop::ProjectionValueMissing {
                    field: field.stable_name(),
                })?,
            worth_ui_query_binding::UiProjectionInputFactReference::Collection(_) => {
                return Err(UiIntentPayloadStop::ProjectionShapeMismatch {
                    field: field.stable_name(),
                })
            }
        };
        self.cost.record_query_input();
        self.owner_revisions.push(UiIntentInputOwnerRevision::query(
            field,
            input.revision().clone(),
        ));
        self.query_inputs.push(input);
        self.text(field, value)
    }

    fn projection_selection(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        projection: &UiResolvedIntentProjectionSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let Some(crate::runtime::interaction::UiSemanticInteraction::SelectionCommit(selection)) =
            self.source.mounted_interaction()
        else {
            return Err(UiIntentPayloadStop::SelectionInteractionRequired {
                field: field.stable_name(),
            });
        };
        let option = selection.option();
        if option.owner_revision().projection_identity() != projection.identity()
            || option.owner_revision().slot() != projection.slot()
        {
            return Err(UiIntentPayloadStop::SelectionProjectionMismatch {
                field: field.stable_name(),
            });
        }
        let input = self.current_projection(field, projection)?;
        let worth_ui_query_binding::UiProjectionInputFactReference::Collection(collection) = &input
        else {
            return Err(UiIntentPayloadStop::ProjectionShapeMismatch {
                field: field.stable_name(),
            });
        };
        if collection.revision() != option.owner_revision() {
            return Err(UiIntentPayloadStop::SelectionRevisionChanged {
                field: field.stable_name(),
            });
        }
        self.cost.record_query_input();
        self.owner_revisions.push(UiIntentInputOwnerRevision::query(
            field,
            input.revision().clone(),
        ));
        self.query_inputs.push(input);
        Ok(UiIntentProjectedValue::selection(
            UiIntentSelectionValue::admitted(option.clone()),
        ))
    }

    fn current_projection(
        &self,
        field: UiIntentPayloadFieldDescriptor,
        projection: &UiResolvedIntentProjectionSource,
    ) -> Result<worth_ui_query_binding::UiProjectionInputFactReference, UiIntentPayloadStop> {
        let input = self.basis.projection(projection.slot()).ok_or_else(|| {
            UiIntentPayloadStop::ProjectionUnavailable {
                field: field.stable_name(),
                projection: projection.identity().clone(),
            }
        })?;
        if input.revision().projection_identity() != projection.identity()
            || input.revision().slot() != projection.slot()
        {
            return Err(UiIntentPayloadStop::ProjectionIdentityMismatch {
                field: field.stable_name(),
                expected: projection.identity().clone(),
                observed: input.revision().projection_identity().clone(),
            });
        }
        if input.posture() != worth_ui_query_binding::UiProjectionInputPosture::Current {
            return Err(UiIntentPayloadStop::ProjectionNotCurrent {
                field: field.stable_name(),
                posture: input.posture(),
            });
        }
        Ok(input)
    }

    fn committed_draft(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let Some(crate::runtime::interaction::UiSemanticInteraction::EditCommit(draft)) =
            self.source.mounted_interaction()
        else {
            return Err(UiIntentPayloadStop::DraftInteractionRequired {
                field: field.stable_name(),
            });
        };
        if draft.field().schema() != self.payload_schema || draft.field().field() != field {
            return Err(UiIntentPayloadStop::DraftFieldMismatch {
                field: field.stable_name(),
            });
        }
        self.owner_revisions.push(UiIntentInputOwnerRevision::draft(
            field,
            draft.session(),
            draft.input_revision(),
            draft.draft_revision(),
        ));
        self.text(field, draft.committed_text_reference())
    }

    fn application_text(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        fact: &UiResolvedIntentApplicationSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let input = self.application_input(field, fact, UiIntentPayloadFieldKind::Text)?;
        let value = input
            .text_value()
            .expect("validated application text fact has text shape");
        self.application_inputs.push(input);
        self.text(field, value)
    }

    fn application_boolean(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        fact: &UiResolvedIntentApplicationSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let input = self.application_input(field, fact, UiIntentPayloadFieldKind::Boolean)?;
        let value = input
            .boolean_value()
            .expect("validated application boolean fact has boolean shape");
        self.application_inputs.push(input);
        Ok(UiIntentProjectedValue::boolean(value))
    }

    fn application_unsigned64(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        fact: &UiResolvedIntentApplicationSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let input = self.application_input(field, fact, UiIntentPayloadFieldKind::Unsigned64)?;
        let value = input
            .unsigned64_value()
            .expect("validated application unsigned fact has unsigned shape");
        self.application_inputs.push(input);
        Ok(UiIntentProjectedValue::unsigned64(value))
    }

    fn application_input(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        fact: &UiResolvedIntentApplicationSource,
        expected: UiIntentPayloadFieldKind,
    ) -> Result<UiIntentApplicationInputReference, UiIntentPayloadStop> {
        let input = self.basis.application(fact.slot()).ok_or_else(|| {
            UiIntentPayloadStop::ApplicationFactUnavailable {
                field: field.stable_name(),
                fact: fact.identity().into(),
            }
        })?;
        if input.revision().identity() != fact.identity() {
            return Err(UiIntentPayloadStop::ApplicationFactIdentityChanged {
                field: field.stable_name(),
                expected: fact.identity().into(),
                observed: input.revision().identity().into(),
            });
        }
        if input.revision().generation() != self.basis.generation() {
            return Err(UiIntentPayloadStop::ApplicationFactGenerationChanged {
                field: field.stable_name(),
                fact: fact.identity().into(),
            });
        }
        if input.kind() != expected {
            return Err(UiIntentPayloadStop::ApplicationFactKindMismatch {
                field: field.stable_name(),
                fact: fact.identity().into(),
                observed: input.kind(),
            });
        }
        self.cost.record_application_input();
        self.owner_revisions
            .push(UiIntentInputOwnerRevision::application(
                field,
                input.revision().identity(),
                input.revision().revision(),
            ));
        Ok(input)
    }

    fn text(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        value: Arc<str>,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        if value.len() > field.byte_budget() {
            return Err(UiIntentPayloadStop::TextByteBudgetExceeded {
                field: field.stable_name(),
                observed: value.len(),
                maximum: field.byte_budget(),
            });
        }
        self.cost.record_utf8_bytes(value.len());
        Ok(UiIntentProjectedValue::text(value))
    }

    fn finish(
        self,
    ) -> (
        Vec<UiIntentProjectedValue>,
        Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
        Vec<UiIntentApplicationInputReference>,
        Vec<UiIntentInputOwnerRevision>,
        UiIntentPayloadProjectionCost,
    ) {
        (
            self.values,
            self.query_inputs,
            self.application_inputs,
            self.owner_revisions,
            self.cost,
        )
    }
}
