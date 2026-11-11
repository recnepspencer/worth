//! One original Bridge payload advances through exclusive retirement phases.
use std::sync::{Arc, RwLock};
use worth_runtime_bridge::facade::{
    BridgeSealedRuntimeAssembly, RetiredRuntimeBridge, RetiringRuntimeBridge, RuntimeBridge,
};

pub(super) enum InstalledBridgeCustody {
    Serving {
        ordinary: RuntimeBridge,
        conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    },
    // Field order preserves the original conditional-before-task destruction.
    Retiring {
        conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
        ordinary: RetiringRuntimeBridge,
    },
    Retired {
        conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
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
    pub(super) fn conditional(&self) -> Option<&Arc<RwLock<BridgeSealedRuntimeAssembly>>> {
        match self {
            Self::Serving { conditional, .. }
            | Self::Retiring { conditional, .. }
            | Self::Retired { conditional, .. } => Some(conditional),
            Self::Released { .. } => None,
        }
    }
}
