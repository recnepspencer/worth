use worth_ui_host_contract::{
    UiHostObservationFamily, UiHostPointerDeviceKind, UiHostPointerIdentity,
};

/// The pointer-presence owner borrows the already sealed local observation
/// bound. It may degrade hover state at saturation, but it never evicts an
/// admitted record or silently changes its identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPointerPresenceCapacity {
    limit: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerPresenceAdmissionDenial {
    Targeting {
        pointer: UiHostPointerIdentity,
        denial: crate::runtime::interaction::targeting::UiInteractionTargetingDenial,
    },
    MissingDeviceKind {
        pointer: UiHostPointerIdentity,
        family: UiHostObservationFamily,
    },
    CapacityExceeded {
        pointer: UiHostPointerIdentity,
        limit: usize,
    },
    PointerKindChanged {
        pointer: UiHostPointerIdentity,
        prior: UiHostPointerDeviceKind,
        observed: UiHostPointerDeviceKind,
    },
}

impl UiPointerPresenceCapacity {
    pub(crate) const fn from_host_observation(
        capacity: crate::host_exchange::observation_report_validation::UiHostObservationCapacity,
    ) -> Self {
        Self {
            limit: capacity.local_reports(),
        }
    }

    #[cfg(test)]
    pub(crate) const fn for_test(limit: usize) -> Self {
        Self { limit }
    }

    pub(crate) const fn limit(self) -> usize {
        self.limit
    }
}

impl UiPointerPresenceAdmissionDenial {
    pub(crate) const fn pointer(self) -> UiHostPointerIdentity {
        match self {
            Self::Targeting { pointer, .. }
            | Self::MissingDeviceKind { pointer, .. }
            | Self::CapacityExceeded { pointer, .. }
            | Self::PointerKindChanged { pointer, .. } => pointer,
        }
    }
}
