pub(crate) struct UiIntentInputBasisView<'state> {
    generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    publication_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    mounted: &'state crate::mounting::WorthUiMountedSessionState,
    application_facts: &'state super::super::UiIntentApplicationFactState,
}

impl<'state> UiIntentInputBasisView<'state> {
    pub(crate) fn observe(
        source: &super::super::super::routing::UiIntentProductInputSource,
        generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        mounted: &'state crate::mounting::WorthUiMountedSessionState,
        application_facts: &'state super::super::UiIntentApplicationFactState,
    ) -> Result<Self, super::super::UiIntentPayloadStop> {
        if source.generation() != generation {
            return Err(super::super::UiIntentPayloadStop::ApplicationGenerationChanged);
        }
        if mounted.has_active_presentation_attempt() {
            return Err(super::super::UiIntentPayloadStop::PublicationTransitionInFlight);
        }
        crate::runtime::interaction::targeting::require_current_target(mounted, source.target())
            .map_err(super::super::UiIntentPayloadStop::Targeting)?;
        let publication_frame = mounted
            .view()
            .current_frame()
            .ok_or(super::super::UiIntentPayloadStop::NoCurrentPublication)?;
        Ok(Self {
            generation,
            publication_frame,
            target: source.target(),
            mounted,
            application_facts,
        })
    }

    pub(crate) const fn generation(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        self.generation
    }

    pub(crate) fn projection(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<worth_ui_query_binding::UiProjectionInputFactReference> {
        self.mounted.current_projection_input(slot)
    }

    pub(crate) fn application(
        &self,
        slot: crate::declaration::UiIntentApplicationFactSlot,
    ) -> Option<super::super::UiIntentApplicationInputReference> {
        self.application_facts
            .input_reference(slot, self.generation)
    }

    pub(crate) const fn target(
        &self,
    ) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        self.target
    }

    pub(crate) fn seal(
        self,
        material: super::UiIntentInputBasisMaterial,
    ) -> super::UiIntentInputBasis {
        super::UiIntentInputBasis::seal(super::UiIntentInputBasisInput {
            generation: self.generation.clone(),
            publication_frame: self.publication_frame,
            target: self.target,
            portal_declaration: material.portal_declaration,
            source: material.source,
            query_inputs: material.query_inputs,
            application_inputs: material.application_inputs,
            owner_revisions: material.owner_revisions,
            route_resolution: material.route_resolution,
            cost: material.cost,
            operability: material.operability,
            evidence_reference: material.evidence_reference,
        })
    }
}
