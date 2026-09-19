#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiFocusTraversalDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiFocusCause {
    #[cfg(any(test, feature = "certification-support"))]
    Direct,
    KeyboardTraversal,
    RovingMovement,
    PortalInitial,
    PortalRestoration,
    RebindPreserved,
    RebindFallback,
}

#[cfg(any(test, feature = "certification-support"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiFocusRequest {
    Direct {
        scope: super::UiFocusScopeIdentity,
        participant: super::UiFocusParticipantIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        cause: UiFocusCause,
    },
    Traverse {
        scope: super::UiFocusScopeIdentity,
        direction: UiFocusTraversalDirection,
        wrap: bool,
    },
    #[cfg(test)]
    First {
        scope: super::UiFocusScopeIdentity,
        cause: UiFocusCause,
    },
    #[cfg(test)]
    Restore(super::UiFocusRestorationToken),
}
