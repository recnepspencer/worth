#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSemanticFocusParticipantObservation {
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticFocusPublicationCause {
    Direct,
    KeyboardTraversal,
    RovingMovement,
    PortalInitial,
    PortalRestoration,
    RebindPreserved,
    RebindFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticFocusPublicationOutcome {
    Moved,
    Unchanged,
    Cleared,
    NoEligibleParticipant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticFocusPhysicalPlacementOutcome {
    Cleared,
    Applied,
    RejectedBeforeEffect,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSemanticFocusPublicationReceipt {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    previous: Option<UiSemanticFocusParticipantObservation>,
    current: Option<UiSemanticFocusParticipantObservation>,
    cause: UiSemanticFocusPublicationCause,
    outcome: UiSemanticFocusPublicationOutcome,
    participants_visited: u32,
    revision: u64,
    host_placement: Option<worth_ui_host_contract::UiHostFocusPlacementAcknowledgement>,
    physical_outcome: UiSemanticFocusPhysicalPlacementOutcome,
}

impl UiSemanticFocusParticipantObservation {
    const fn from_semantic_focus(focus: crate::runtime::focus::UiSemanticKeyboardFocus) -> Self {
        Self {
            mounted_instance: focus.mounted_instance(),
            incarnation: focus.incarnation(),
        }
    }

    pub const fn mounted_instance(self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub const fn incarnation(self) -> worth_ui_host_contract::UiMountIncarnation {
        self.incarnation
    }
}

impl UiSemanticFocusPublicationReceipt {
    pub(super) const fn new(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        transition: crate::runtime::focus::UiFocusTransitionReceipt,
        host_placement: Option<worth_ui_host_contract::UiHostFocusPlacementAcknowledgement>,
    ) -> Self {
        let current = transition.current();
        Self {
            frame,
            previous: match transition.previous() {
                Some(focus) => Some(UiSemanticFocusParticipantObservation::from_semantic_focus(
                    focus,
                )),
                None => None,
            },
            current: match current {
                Some(focus) => Some(UiSemanticFocusParticipantObservation::from_semantic_focus(
                    focus,
                )),
                None => None,
            },
            cause: map_cause(transition.cause()),
            outcome: map_outcome(transition.outcome()),
            participants_visited: transition.participants_visited(),
            revision: transition.revision(),
            host_placement,
            physical_outcome: physical_outcome(current.is_some(), host_placement),
        }
    }

    pub const fn frame(self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }

    pub const fn previous(self) -> Option<UiSemanticFocusParticipantObservation> {
        self.previous
    }

    pub const fn current(self) -> Option<UiSemanticFocusParticipantObservation> {
        self.current
    }

    pub const fn cause(self) -> UiSemanticFocusPublicationCause {
        self.cause
    }

    pub const fn outcome(self) -> UiSemanticFocusPublicationOutcome {
        self.outcome
    }

    pub const fn participants_visited(self) -> u32 {
        self.participants_visited
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }

    pub const fn host_placement(
        self,
    ) -> Option<worth_ui_host_contract::UiHostFocusPlacementAcknowledgement> {
        self.host_placement
    }

    pub const fn physical_outcome(self) -> UiSemanticFocusPhysicalPlacementOutcome {
        self.physical_outcome
    }
}

const fn physical_outcome(
    has_current: bool,
    host_placement: Option<worth_ui_host_contract::UiHostFocusPlacementAcknowledgement>,
) -> UiSemanticFocusPhysicalPlacementOutcome {
    let Some(host_placement) = host_placement else {
        return if has_current {
            UiSemanticFocusPhysicalPlacementOutcome::Indeterminate
        } else {
            UiSemanticFocusPhysicalPlacementOutcome::Cleared
        };
    };
    match host_placement.disposition() {
        worth_ui_host_contract::UiHostFocusPlacementDisposition::Applied => {
            UiSemanticFocusPhysicalPlacementOutcome::Applied
        }
        worth_ui_host_contract::UiHostFocusPlacementDisposition::RejectedBeforeEffect(_) => {
            UiSemanticFocusPhysicalPlacementOutcome::RejectedBeforeEffect
        }
        worth_ui_host_contract::UiHostFocusPlacementDisposition::Indeterminate => {
            UiSemanticFocusPhysicalPlacementOutcome::Indeterminate
        }
    }
}

const fn map_cause(cause: crate::runtime::focus::UiFocusCause) -> UiSemanticFocusPublicationCause {
    match cause {
        #[cfg(any(test, feature = "certification-support"))]
        crate::runtime::focus::UiFocusCause::Direct => UiSemanticFocusPublicationCause::Direct,
        crate::runtime::focus::UiFocusCause::KeyboardTraversal => {
            UiSemanticFocusPublicationCause::KeyboardTraversal
        }
        crate::runtime::focus::UiFocusCause::RovingMovement => {
            UiSemanticFocusPublicationCause::RovingMovement
        }
        crate::runtime::focus::UiFocusCause::PortalInitial => {
            UiSemanticFocusPublicationCause::PortalInitial
        }
        crate::runtime::focus::UiFocusCause::PortalRestoration => {
            UiSemanticFocusPublicationCause::PortalRestoration
        }
        crate::runtime::focus::UiFocusCause::RebindPreserved => {
            UiSemanticFocusPublicationCause::RebindPreserved
        }
        crate::runtime::focus::UiFocusCause::RebindFallback => {
            UiSemanticFocusPublicationCause::RebindFallback
        }
    }
}

const fn map_outcome(
    outcome: crate::runtime::focus::UiFocusOutcome,
) -> UiSemanticFocusPublicationOutcome {
    match outcome {
        crate::runtime::focus::UiFocusOutcome::Moved => UiSemanticFocusPublicationOutcome::Moved,
        crate::runtime::focus::UiFocusOutcome::Unchanged => {
            UiSemanticFocusPublicationOutcome::Unchanged
        }
        crate::runtime::focus::UiFocusOutcome::Cleared => {
            UiSemanticFocusPublicationOutcome::Cleared
        }
        crate::runtime::focus::UiFocusOutcome::NoEligibleParticipant => {
            UiSemanticFocusPublicationOutcome::NoEligibleParticipant
        }
    }
}
