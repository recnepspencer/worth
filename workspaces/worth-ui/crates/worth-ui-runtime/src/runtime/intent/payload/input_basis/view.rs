pub(crate) struct UiIntentInputBasisView<'state> {
    generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    publication_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    graph_node: crate::graph::UiGraphNodeIdentity,
    owners: UiIntentInputOwners<'state>,
}

/// The owners an input basis reads, all observed at one generation.
#[derive(Clone, Copy)]
pub(crate) struct UiIntentInputOwners<'state> {
    pub(crate) mounted: &'state crate::mounting::WorthUiMountedSessionState,
    pub(crate) application_facts: &'state super::super::UiIntentApplicationFactState,
    pub(crate) expressions: &'state crate::runtime::expression::UiExpressionRuntimeState,
}

impl<'state> UiIntentInputBasisView<'state> {
    pub(crate) fn observe(
        source: &super::super::super::routing::UiIntentProductInputSource,
        generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        owners: UiIntentInputOwners<'state>,
    ) -> Result<Self, super::super::UiIntentPayloadStop> {
        if source.generation() != generation {
            return Err(super::super::UiIntentPayloadStop::ApplicationGenerationChanged);
        }
        Self::admit_target(source.target(), generation, owners)
    }

    /// Observes dependencies without issuing an interaction or payload source.
    pub(crate) fn observe_target_with<R>(
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
        generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        owners: UiIntentInputOwners<'state>,
        observe: impl FnOnce(&Self) -> R,
    ) -> Result<R, super::super::UiIntentPayloadStop> {
        let view = Self::admit_target(target, generation, owners)?;
        Ok(observe(&view))
    }

    fn admit_target(
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
        generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        owners: UiIntentInputOwners<'state>,
    ) -> Result<Self, super::super::UiIntentPayloadStop> {
        let mounted = owners.mounted;
        if mounted.has_active_presentation_attempt() {
            return Err(super::super::UiIntentPayloadStop::PublicationTransitionInFlight);
        }
        let affinity =
            crate::runtime::interaction::targeting::admit_current_target(mounted, target)
                .map_err(super::super::UiIntentPayloadStop::Targeting)?;
        let publication_frame = mounted
            .view()
            .current_frame()
            .ok_or(super::super::UiIntentPayloadStop::NoCurrentPublication)?;
        Ok(Self {
            generation,
            publication_frame,
            target,
            graph_node: affinity.graph_node(),
            owners,
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
        self.owners.mounted.current_projection_input(slot)
    }

    pub(crate) fn application(
        &self,
        slot: crate::declaration::UiIntentApplicationFactSlot,
    ) -> Option<super::super::UiIntentApplicationInputReference> {
        self.owners
            .application_facts
            .input_reference(slot, self.generation)
    }

    /// The retained result of `slot`, whatever its posture. `None` when the
    /// expression owner does not follow the generation this view observes.
    pub(crate) fn expression(
        &self,
        slot: crate::runtime::expression::UiExpressionSlot,
    ) -> Option<crate::runtime::expression::UiExpressionResultReference> {
        self.owners
            .expressions
            .result(slot)
            .filter(|result| result.generation() == self.generation)
    }

    /// The graph node the admitted target is mounted from.
    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
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
