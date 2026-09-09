use super::{
    focus_input_recipient_disposition, UiFocusInputRecipientDisposition,
    UiFocusPlacementExecutionDenial, UiFocusPlacementReconciliationExecutionDenial,
};

pub(in crate::facade::entry) struct UiFocusPlacementPorts<'a> {
    mounted: &'a mut crate::mounting::WorthUiMountedSessionState,
    pub(super) focus: &'a mut crate::runtime::focus::UiFocusRuntimeState,
    pub(super) interaction: &'a mut crate::runtime::interaction::UiInteractionRuntimeState,
    pub(super) host_session: &'a crate::facade::WorthUiHostSessionAuthority,
    active_generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
}

impl UiFocusPlacementPorts<'_> {
    pub(in crate::facade::entry) fn new<'a>(
        mounted: &'a mut crate::mounting::WorthUiMountedSessionState,
        focus: &'a mut crate::runtime::focus::UiFocusRuntimeState,
        interaction: &'a mut crate::runtime::interaction::UiInteractionRuntimeState,
        host_session: &'a crate::facade::WorthUiHostSessionAuthority,
        active_generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> UiFocusPlacementPorts<'a> {
        UiFocusPlacementPorts {
            mounted,
            focus,
            interaction,
            host_session,
            active_generation,
        }
    }

    pub(in crate::facade::entry) fn place(
        &mut self,
        transition: crate::runtime::focus::UiFocusTransitionReceipt,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Result<
        Option<worth_ui_host_contract::UiHostFocusPlacementAcknowledgement>,
        UiFocusPlacementExecutionDenial,
    > {
        let Some(current) = transition.current() else {
            self.clear_previous(transition);
            return Ok(None);
        };
        let target = current.mounted_target();
        self.inspect_target(target, publication)?;
        let presentation = self
            .mounted
            .current_presentation_for_surface(current.scope().semantic_surface())
            .ok_or(UiFocusPlacementExecutionDenial::SurfaceUnavailable)?;
        let supported = self
            .host_session
            .capability_report()
            .supports(worth_ui_host_contract::WorthUiHostCapability::SemanticFocusPlacement);
        let acknowledgement = self
            .mounted
            .place_semantic_focus(
                crate::mounting::UiMountedFocusPlacementRequestBasis {
                    protocol: self.host_session.protocol(),
                    host_session: self.host_session.identity().as_u64(),
                    host_surface: presentation.host_surface(),
                    binding: presentation.binding(),
                    presentation,
                    target,
                },
                supported,
                self.host_session.effect_port(),
            )
            .map_err(map_mounted_denial)?;
        match focus_input_recipient_disposition(transition, self.focus.requires_focused_submit()) {
            UiFocusInputRecipientDisposition::BindCurrent
                if acknowledgement.disposition()
                    == worth_ui_host_contract::UiHostFocusPlacementDisposition::Applied =>
            {
                self.bind_current(target, presentation)?;
            }
            UiFocusInputRecipientDisposition::BindCurrent
            | UiFocusInputRecipientDisposition::ClearPrevious => self.clear_previous(transition),
            UiFocusInputRecipientDisposition::Preserve => {}
        }
        Ok(Some(acknowledgement))
    }

    pub(in crate::facade::entry) fn reconcile(
        &mut self,
        observation: worth_ui_host_contract::UiHostFocusPlacementObservation,
    ) -> Result<
        crate::mounting::UiFocusHostPlacementReconciliationReceipt,
        UiFocusPlacementReconciliationExecutionDenial,
    > {
        let receipt = self
            .mounted
            .reconcile_focus_placement(observation)
            .map_err(UiFocusPlacementReconciliationExecutionDenial::Host)?;
        if receipt.outcome()
            == crate::mounting::UiFocusHostPlacementReconciliationOutcome::RequestedTargetObserved
            && self.focus.requires_focused_submit()
        {
            if let Some(current) = self.focus.current_semantic_focus() {
                self.bind_current(current.mounted_target(), observation.presentation())
                    .map_err(|_| {
                        UiFocusPlacementReconciliationExecutionDenial::RecipientInstallation
                    })?;
            }
        }
        Ok(receipt)
    }

    fn inspect_target(
        &self,
        target: worth_ui_host_contract::UiHostFocusPlacementTarget,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Result<
        Box<crate::inspection::mounted_frame::UiMountedInspectedFrame>,
        UiFocusPlacementExecutionDenial,
    > {
        let inspected = match self.mounted.inspect_frame(
            crate::inspection::mounted_frame::UiMountedInspectionRequest::current()
                .for_instance(target.mounted_instance()),
        ) {
            crate::inspection::mounted_frame::UiMountedInspectionReceipt::Available(frame) => frame,
            crate::inspection::mounted_frame::UiMountedInspectionReceipt::Omitted(_) => {
                return Err(UiFocusPlacementExecutionDenial::MountedFrameUnavailable);
            }
        };
        if inspected.frame() != publication.frame() {
            return Err(UiFocusPlacementExecutionDenial::ForeignPublishedFrame);
        }
        if inspected.selected_node_receipt() != Some(target.node_receipt()) {
            return Err(UiFocusPlacementExecutionDenial::TargetReceiptMismatch);
        }
        Ok(inspected)
    }

    fn bind_current(
        &mut self,
        target: worth_ui_host_contract::UiHostFocusPlacementTarget,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<(), UiFocusPlacementExecutionDenial> {
        let target = crate::runtime::interaction::targeting::resolve_presented_focus_target(
            self.mounted,
            presentation,
            target,
        )
        .map_err(|_| UiFocusPlacementExecutionDenial::MissingInteractionTarget)?
        .ok_or(UiFocusPlacementExecutionDenial::MissingInteractionTarget)?;
        let previous = self.interaction.active_input_binding();
        let context = crate::runtime::interaction::draft::UiLocalInputRecipientBindingContext::new(
            self.host_session.identity().as_u64(),
            self.interaction.application_generation(),
            &self.active_generation,
            self.mounted,
        );
        self.interaction
            .bind_focused_submit(target, context, |binding| {
                self.host_session.install_input_recipient(binding)
            })
            .map_err(UiFocusPlacementExecutionDenial::InputRecipient)?;
        self.clear_displaced(previous);
        Ok(())
    }

    fn clear_previous(&mut self, transition: crate::runtime::focus::UiFocusTransitionReceipt) {
        let previous = self.interaction.active_input_binding();
        if let Some(previous_focus) = transition.previous() {
            self.interaction
                .clear_focused_recipient(previous_focus.mounted_target().mounted_instance());
        }
        self.clear_displaced(previous);
    }

    fn clear_displaced(
        &self,
        previous: Option<worth_ui_host_contract::UiHostInputRecipientBindingReceipt>,
    ) {
        if let Some(previous) = previous {
            if self.interaction.active_input_binding() != Some(previous) {
                let _ = self.host_session.clear_input_recipient(previous);
            }
        }
    }
}

fn map_mounted_denial(
    denial: crate::mounting::UiMountedFocusPlacementDenial,
) -> UiFocusPlacementExecutionDenial {
    match denial {
        crate::mounting::UiMountedFocusPlacementDenial::IdentityExhausted => {
            UiFocusPlacementExecutionDenial::IdentityExhausted
        }
        crate::mounting::UiMountedFocusPlacementDenial::Request(denial) => {
            UiFocusPlacementExecutionDenial::Request(denial)
        }
        crate::mounting::UiMountedFocusPlacementDenial::Settlement(denial) => {
            UiFocusPlacementExecutionDenial::Settlement(denial)
        }
    }
}
