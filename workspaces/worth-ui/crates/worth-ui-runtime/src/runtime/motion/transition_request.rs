//! The admitted request to move one Motion target from its committed
//! predecessor to its prepared successor, and the typed refusals that keep an
//! inadmissible one from reaching a track.

use super::receipt::{UiCommittedMotionPredecessorReceipt, UiPreparedMotionSuccessorReceipt};

/// One end of a Motion transition as its owner states it: the owner revision
/// the endpoint stands at, the presentation it is published against, and the
/// geometry and visibility it holds there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionTransitionEndpoint {
    revision: u64,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    geometry: Option<crate::mounting::presentation::UiPublishedRect>,
    visible: bool,
}

impl UiMotionTransitionEndpoint {
    pub(crate) const fn new(
        revision: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        geometry: Option<crate::mounting::presentation::UiPublishedRect>,
        visible: bool,
    ) -> Self {
        Self {
            revision,
            presentation,
            geometry,
            visible,
        }
    }

    pub(crate) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionTransitionRequest {
    predecessor: UiCommittedMotionPredecessorReceipt,
    successor: UiPreparedMotionSuccessorReceipt,
    declaration: super::UiMotionDeclaration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionTransitionRequestDenial {
    RevisionDidNotAdvance,
    SurfaceChanged,
    BindingChangedWithoutRebind,
    GeometryCoordinateSpaceChanged,
    /// This target's retained exit has issued physical work that has not
    /// settled. A successor transition would displace that retention while its
    /// terminal is still in flight, so the request is refused before effect.
    ExitRetentionAwaitingPhysicalSettlement,
}

impl UiMotionTransitionRequest {
    pub(super) const fn with_policy(mut self, policy: crate::declaration::UiMotionPolicy) -> Self {
        self.declaration = self.declaration.with_policy(policy);
        self
    }

    /// A transition within one presentation family: the successor must keep
    /// the predecessor's host surface and binding.
    pub(crate) fn from_family_transition(
        target: super::UiMotionTargetIdentity,
        predecessor: UiMotionTransitionEndpoint,
        successor: UiMotionTransitionEndpoint,
        declaration: super::UiMotionDeclaration,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        Self::from_transition(target, predecessor, successor, declaration, false)
    }

    /// A transition that rebinds to a successor presentation, so a changed
    /// host surface or binding is the point rather than a refusal.
    pub(crate) fn from_rebind_transition(
        target: super::UiMotionTargetIdentity,
        predecessor: UiMotionTransitionEndpoint,
        successor: UiMotionTransitionEndpoint,
        declaration: super::UiMotionDeclaration,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        Self::from_transition(target, predecessor, successor, declaration, true)
    }

    fn from_transition(
        target: super::UiMotionTargetIdentity,
        predecessor: UiMotionTransitionEndpoint,
        successor: UiMotionTransitionEndpoint,
        declaration: super::UiMotionDeclaration,
        rebind: bool,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        if successor.revision <= predecessor.revision {
            return Err(UiMotionTransitionRequestDenial::RevisionDidNotAdvance);
        }
        if !rebind
            && predecessor.presentation.host_surface() != successor.presentation.host_surface()
        {
            return Err(UiMotionTransitionRequestDenial::SurfaceChanged);
        }
        if !rebind && predecessor.presentation.binding() != successor.presentation.binding() {
            return Err(UiMotionTransitionRequestDenial::BindingChangedWithoutRebind);
        }
        if predecessor
            .geometry
            .zip(successor.geometry)
            .is_some_and(|(predecessor, successor)| {
                predecessor.coordinate_space() != successor.coordinate_space()
            })
        {
            return Err(UiMotionTransitionRequestDenial::GeometryCoordinateSpaceChanged);
        }
        Ok(Self {
            predecessor: UiCommittedMotionPredecessorReceipt::committed(
                predecessor.geometry,
                predecessor.visible,
            ),
            successor: UiPreparedMotionSuccessorReceipt::prepared(
                target,
                successor.revision,
                successor.presentation,
                successor.geometry,
                successor.visible,
            ),
            declaration,
        })
    }

    pub(in crate::runtime) const fn predecessor(self) -> UiCommittedMotionPredecessorReceipt {
        self.predecessor
    }

    pub(in crate::runtime) const fn successor(self) -> UiPreparedMotionSuccessorReceipt {
        self.successor
    }

    pub(in crate::runtime) const fn declaration(self) -> super::UiMotionDeclaration {
        self.declaration
    }

    pub(in crate::runtime) fn bind_published_successor(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<Self, UiMotionTransitionRequestDenial> {
        if self.successor.presentation().host_surface() != presentation.host_surface() {
            return Err(UiMotionTransitionRequestDenial::SurfaceChanged);
        }
        if self.successor.presentation().binding() != presentation.binding() {
            return Err(UiMotionTransitionRequestDenial::BindingChangedWithoutRebind);
        }
        self.successor = self.successor.republished_at(presentation);
        Ok(self)
    }

    pub(in crate::runtime) fn rebind_published_successor(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        self.successor = self.successor.republished_at(presentation);
        self
    }
}
