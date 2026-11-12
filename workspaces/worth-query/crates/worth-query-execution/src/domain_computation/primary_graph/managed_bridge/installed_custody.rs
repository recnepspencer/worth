//! One original Bridge payload advances through exclusive retirement phases.
use super::conditional_owner::ConditionalOwner;
use worth_runtime_bridge::facade::{RetiredRuntimeBridge, RetiringRuntimeBridge, RuntimeBridge};

pub(super) enum InstalledBridgeCustody {
    Serving {
        ordinary: RuntimeBridge,
        conditional: ConditionalOwner,
    },
    // Field order preserves the original conditional-before-task destruction.
    Retiring {
        conditional: ConditionalOwner,
        ordinary: RetiringRuntimeBridge,
    },
    Retired {
        conditional: ConditionalOwner,
        ordinary: RetiredRuntimeBridge,
    },
    Released {
        ordinary: RetiredRuntimeBridge,
    },
}
impl InstalledBridgeCustody {
    pub(super) fn ordinary(&self) -> Option<&RuntimeBridge> {
        match self {
            Self::Serving { ordinary, .. } => Some(ordinary),
            Self::Retiring { .. } | Self::Retired { .. } | Self::Released { .. } => None,
        }
    }
    pub(super) fn conditional(&self) -> Option<&ConditionalOwner> {
        match self {
            Self::Serving { conditional, .. }
            | Self::Retiring { conditional, .. }
            | Self::Retired { conditional, .. } => Some(conditional),
            Self::Released { .. } => None,
        }
    }
}
