//! The two committed endpoints a Motion transition is built from: the state
//! the owner has already published, and the state it has prepared next.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiCommittedMotionPredecessorReceipt {
    geometry: Option<crate::mounting::presentation::UiPublishedRect>,
    visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiPreparedMotionSuccessorReceipt {
    target: super::UiMotionTargetIdentity,
    owner_revision: u64,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    geometry: Option<crate::mounting::presentation::UiPublishedRect>,
    visible: bool,
}

impl UiCommittedMotionPredecessorReceipt {
    pub(super) const fn committed(
        geometry: Option<crate::mounting::presentation::UiPublishedRect>,
        visible: bool,
    ) -> Self {
        Self { geometry, visible }
    }

    pub(in crate::runtime) const fn geometry(
        self,
    ) -> Option<crate::mounting::presentation::UiPublishedRect> {
        self.geometry
    }

    pub(in crate::runtime) const fn visible(self) -> bool {
        self.visible
    }
}

impl UiPreparedMotionSuccessorReceipt {
    pub(super) const fn prepared(
        target: super::UiMotionTargetIdentity,
        owner_revision: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        geometry: Option<crate::mounting::presentation::UiPublishedRect>,
        visible: bool,
    ) -> Self {
        Self {
            target,
            owner_revision,
            presentation,
            geometry,
            visible,
        }
    }

    /// Republish this prepared successor against a later host presentation
    /// basis. Whether that basis is a lawful continuation or a rebind is the
    /// transition request's decision, not this receipt's.
    pub(super) const fn republished_at(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        self.presentation = presentation;
        self
    }

    pub(in crate::runtime) const fn target(self) -> super::UiMotionTargetIdentity {
        self.target
    }

    pub(in crate::runtime) const fn owner_revision(self) -> u64 {
        self.owner_revision
    }

    pub(in crate::runtime) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }

    pub(in crate::runtime) const fn geometry(
        self,
    ) -> Option<crate::mounting::presentation::UiPublishedRect> {
        self.geometry
    }

    pub(in crate::runtime) const fn visible(self) -> bool {
        self.visible
    }
}
