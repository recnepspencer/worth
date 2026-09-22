#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFocusPlacementExecutionDenial {
    OwnerUnavailable,
    IdentityExhausted,
    MountedFrameUnavailable,
    UnpublishedScrollGeometry,
    ForeignPublishedFrame,
    TargetReceiptMismatch,
    SurfaceUnavailable,
    Request(worth_ui_host_contract::UiHostFocusPlacementRequestDenial),
    Settlement(crate::mounting::UiFocusHostPlacementSettlementDenial),
    MissingInteractionTarget,
    InputRecipient(crate::runtime::interaction::UiLocalInputRecipientBindingStopReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFocusPlacementReconciliationExecutionDenial {
    OwnerUnavailable,
    Host(crate::mounting::UiFocusHostPlacementReconciliationDenial),
    RecipientInstallation,
}

impl super::WorthUiActiveApplicationSession {
    pub(super) fn place_committed_semantic_focus(
        &mut self,
        transition: crate::runtime::focus::UiFocusTransitionReceipt,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Result<UiSemanticFocusPublicationReceipt, UiFocusPlacementExecutionDenial> {
        let active_generation = self.active_generation_identity();
        let Some(focus) = self.focus.as_mut() else {
            return Err(UiFocusPlacementExecutionDenial::OwnerUnavailable);
        };
        let host_placement = ports::UiFocusPlacementPorts::new(
            &mut self.mounted,
            focus,
            &mut self.interaction,
            &self.host_session,
            active_generation,
        )
        .place(transition, publication)?;
        Ok(UiSemanticFocusPublicationReceipt::new(
            publication.frame(),
            transition,
            host_placement,
        ))
    }

    pub fn reconcile_indeterminate_focus_placement(
        &mut self,
        observation: worth_ui_host_contract::UiHostFocusPlacementObservation,
    ) -> Result<
        crate::mounting::UiFocusHostPlacementReconciliationReceipt,
        UiFocusPlacementReconciliationExecutionDenial,
    > {
        let active_generation = self.active_generation_identity();
        let Some(focus) = self.focus.as_mut() else {
            return Err(UiFocusPlacementReconciliationExecutionDenial::OwnerUnavailable);
        };
        ports::UiFocusPlacementPorts::new(
            &mut self.mounted,
            focus,
            &mut self.interaction,
            &self.host_session,
            active_generation,
        )
        .reconcile(observation)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiFocusInputRecipientDisposition {
    Preserve,
    BindCurrent,
    ClearPrevious,
}

fn focus_input_recipient_disposition(
    transition: crate::runtime::focus::UiFocusTransitionReceipt,
    requires_focused_submit: bool,
) -> UiFocusInputRecipientDisposition {
    if transition.current().is_none() {
        return UiFocusInputRecipientDisposition::ClearPrevious;
    }
    if !matches!(
        transition.cause(),
        crate::runtime::focus::UiFocusCause::PortalInitial
            | crate::runtime::focus::UiFocusCause::PortalRestoration
            | crate::runtime::focus::UiFocusCause::KeyboardTraversal
            | crate::runtime::focus::UiFocusCause::RovingMovement
            | crate::runtime::focus::UiFocusCause::RebindPreserved
            | crate::runtime::focus::UiFocusCause::RebindFallback
    ) {
        return UiFocusInputRecipientDisposition::Preserve;
    }
    if requires_focused_submit {
        UiFocusInputRecipientDisposition::BindCurrent
    } else {
        UiFocusInputRecipientDisposition::ClearPrevious
    }
}
#[path = "focus_placement/ports.rs"]
pub(super) mod ports;
#[path = "focus_placement/publication.rs"]
mod publication;
pub use publication::{
    UiSemanticFocusParticipantObservation, UiSemanticFocusPhysicalPlacementOutcome,
    UiSemanticFocusPublicationCause, UiSemanticFocusPublicationOutcome,
    UiSemanticFocusPublicationReceipt,
};
