#[derive(Debug)]
pub struct UiSelectionCommitInteraction {
    activation: super::UiActivateInteraction,
    option: worth_ui_query_binding::UiProjectionOptionReference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSelectionCommitStopReason {
    ApplicationGenerationChanged,
    TargetNoLongerCurrent(super::super::UiInteractionTargetingDenial),
    ProjectionUnavailable,
    ProjectionNotCurrent(worth_ui_query_binding::UiProjectionInputPosture),
    ProjectionShapeMismatch,
    ProjectionRevisionChanged,
    MountedSelectionBindingUnavailable,
}

#[derive(Debug)]
pub struct UiSelectionCommitStop {
    activation: Box<super::UiActivateInteraction>,
    option: worth_ui_query_binding::UiProjectionOptionReference,
    reason: UiSelectionCommitStopReason,
}

pub(crate) fn commit_selection(
    activation: super::UiActivateInteraction,
    option: worth_ui_query_binding::UiProjectionOptionReference,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
) -> Result<UiSelectionCommitInteraction, UiSelectionCommitStop> {
    let reason = selection_stop_reason(&activation, &option, generation, mounted);
    match reason {
        Some(reason) => Err(UiSelectionCommitStop {
            activation: Box::new(activation),
            option,
            reason,
        }),
        None => Ok(UiSelectionCommitInteraction { activation, option }),
    }
}

fn selection_stop_reason(
    activation: &super::UiActivateInteraction,
    option: &worth_ui_query_binding::UiProjectionOptionReference,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
) -> Option<UiSelectionCommitStopReason> {
    if activation.generation() != generation {
        return Some(UiSelectionCommitStopReason::ApplicationGenerationChanged);
    }
    if let Err(denial) =
        super::super::targeting::require_current_target(mounted, activation.target())
    {
        return Some(UiSelectionCommitStopReason::TargetNoLongerCurrent(denial));
    }
    let Some(current) = mounted.current_projection_input(option.owner_revision().slot()) else {
        return Some(UiSelectionCommitStopReason::ProjectionUnavailable);
    };
    if current.revision().projection_identity() != option.owner_revision().projection_identity() {
        return Some(UiSelectionCommitStopReason::ProjectionUnavailable);
    }
    let worth_ui_query_binding::UiProjectionInputFactReference::Collection(current) = current
    else {
        return Some(UiSelectionCommitStopReason::ProjectionShapeMismatch);
    };
    if current.posture() != worth_ui_query_binding::UiProjectionInputPosture::Current {
        return Some(UiSelectionCommitStopReason::ProjectionNotCurrent(
            current.posture(),
        ));
    }
    if current.revision() != option.owner_revision() {
        return Some(UiSelectionCommitStopReason::ProjectionRevisionChanged);
    }
    (!mounted.selection_item_matches_option(activation.target().mounted_instance(), option))
        .then_some(UiSelectionCommitStopReason::MountedSelectionBindingUnavailable)
}

impl UiSelectionCommitInteraction {
    pub const fn activation(&self) -> &super::UiActivateInteraction {
        &self.activation
    }

    pub const fn target(&self) -> super::super::UiPresentedInteractionTargetView {
        self.activation.target()
    }

    pub const fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        self.activation.generation()
    }

    pub const fn option(&self) -> &worth_ui_query_binding::UiProjectionOptionReference {
        &self.option
    }
}

impl UiSelectionCommitStop {
    pub const fn activation(&self) -> &super::UiActivateInteraction {
        &self.activation
    }

    pub const fn option(&self) -> &worth_ui_query_binding::UiProjectionOptionReference {
        &self.option
    }

    pub const fn reason(&self) -> UiSelectionCommitStopReason {
        self.reason
    }

    pub fn into_parts(
        self,
    ) -> (
        super::UiActivateInteraction,
        worth_ui_query_binding::UiProjectionOptionReference,
    ) {
        (*self.activation, self.option)
    }
}
