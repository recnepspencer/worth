#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiFocusOutcome {
    Moved,
    Unchanged,
    Cleared,
    NoEligibleParticipant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiFocusTransitionReceipt {
    previous: Option<super::UiSemanticKeyboardFocus>,
    current: Option<super::UiSemanticKeyboardFocus>,
    cause: super::UiFocusCause,
    outcome: UiFocusOutcome,
    participants_visited: u32,
    revision: u64,
}

impl UiFocusTransitionReceipt {
    pub(super) const fn new(
        previous: Option<super::UiSemanticKeyboardFocus>,
        current: Option<super::UiSemanticKeyboardFocus>,
        cause: super::UiFocusCause,
        outcome: UiFocusOutcome,
        participants_visited: u32,
        revision: u64,
    ) -> Self {
        Self {
            previous,
            current,
            cause,
            outcome,
            participants_visited,
            revision,
        }
    }

    pub(crate) const fn previous(self) -> Option<super::UiSemanticKeyboardFocus> {
        self.previous
    }
    pub(crate) const fn current(self) -> Option<super::UiSemanticKeyboardFocus> {
        self.current
    }
    pub(crate) const fn cause(self) -> super::UiFocusCause {
        self.cause
    }
    pub(crate) const fn outcome(self) -> UiFocusOutcome {
        self.outcome
    }
    pub(crate) const fn participants_visited(self) -> u32 {
        self.participants_visited
    }
    pub(crate) const fn revision(self) -> u64 {
        self.revision
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiFocusReconciliationReceipt {
    transition: Option<UiFocusTransitionReceipt>,
    mounted_nodes_visited: u32,
    participants_installed: u32,
}

impl UiFocusReconciliationReceipt {
    pub(super) const fn new(
        transition: Option<UiFocusTransitionReceipt>,
        mounted_nodes_visited: u32,
        participants_installed: u32,
    ) -> Self {
        Self {
            transition,
            mounted_nodes_visited,
            participants_installed,
        }
    }

    pub(crate) const fn transition(self) -> Option<UiFocusTransitionReceipt> {
        self.transition
    }
    #[cfg(test)]
    pub(crate) const fn mounted_nodes_visited(self) -> u32 {
        self.mounted_nodes_visited
    }
    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) const fn participants_installed(self) -> u32 {
        self.participants_installed
    }
}
