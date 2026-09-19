#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiProducedFactOwner {
    SourceIngress,
    HostViewport,
    HostDeviceScale,
    PointerPresenceRuntimeState,
    MeasurementExchange,
    QueryBinding,
    IntentRuntime,
    ScrollRuntimeState,
    PortalRuntimeState,
    FocusRuntimeState,
    SelectionRuntimeState,
    MotionRuntimeState,
    CommandRoutingRuntimeState,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiProducedFactFamily {
    AuthoredSource,
    HostViewport,
    HostDeviceScale,
    PointerPresenceTarget,
    Measurement,
    Query,
    IntentPosture,
    CommittedScrollExtent,
    CommittedPortalAnchor,
    CommittedFocus,
    CommittedSelection,
    CommittedMotionTrack,
    CommittedCommandRoute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiProducedFactResetPosture {
    NoReset,
    OwnerIssuedReset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiProducedFactContract {
    owner: UiProducedFactOwner,
    family: UiProducedFactFamily,
    reset: UiProducedFactResetPosture,
}

impl UiProducedFactContract {
    pub const fn for_owner(owner: UiProducedFactOwner) -> Self {
        match owner {
            UiProducedFactOwner::SourceIngress => {
                Self::new(owner, UiProducedFactFamily::AuthoredSource, false)
            }
            UiProducedFactOwner::HostViewport => {
                Self::new(owner, UiProducedFactFamily::HostViewport, false)
            }
            UiProducedFactOwner::HostDeviceScale => {
                Self::new(owner, UiProducedFactFamily::HostDeviceScale, false)
            }
            UiProducedFactOwner::PointerPresenceRuntimeState => {
                Self::new(owner, UiProducedFactFamily::PointerPresenceTarget, false)
            }
            UiProducedFactOwner::MeasurementExchange => {
                Self::new(owner, UiProducedFactFamily::Measurement, false)
            }
            UiProducedFactOwner::QueryBinding => {
                Self::new(owner, UiProducedFactFamily::Query, true)
            }
            UiProducedFactOwner::IntentRuntime => {
                Self::new(owner, UiProducedFactFamily::IntentPosture, false)
            }
            UiProducedFactOwner::ScrollRuntimeState => {
                Self::new(owner, UiProducedFactFamily::CommittedScrollExtent, false)
            }
            UiProducedFactOwner::PortalRuntimeState => {
                Self::new(owner, UiProducedFactFamily::CommittedPortalAnchor, false)
            }
            UiProducedFactOwner::FocusRuntimeState => {
                Self::new(owner, UiProducedFactFamily::CommittedFocus, false)
            }
            UiProducedFactOwner::SelectionRuntimeState => {
                Self::new(owner, UiProducedFactFamily::CommittedSelection, false)
            }
            UiProducedFactOwner::MotionRuntimeState => {
                Self::new(owner, UiProducedFactFamily::CommittedMotionTrack, false)
            }
            UiProducedFactOwner::CommandRoutingRuntimeState => {
                Self::new(owner, UiProducedFactFamily::CommittedCommandRoute, false)
            }
        }
    }

    const fn new(
        owner: UiProducedFactOwner,
        family: UiProducedFactFamily,
        owner_issued_reset: bool,
    ) -> Self {
        Self {
            owner,
            family,
            reset: if owner_issued_reset {
                UiProducedFactResetPosture::OwnerIssuedReset
            } else {
                UiProducedFactResetPosture::NoReset
            },
        }
    }

    pub const fn owner(self) -> UiProducedFactOwner {
        self.owner
    }

    pub const fn family(self) -> UiProducedFactFamily {
        self.family
    }

    pub const fn reset_posture(self) -> UiProducedFactResetPosture {
        self.reset
    }
}
