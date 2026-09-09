use super::super::appearance::UiMountedAppearanceSidecar;
use super::appearance_state_membership::{
    UiMountedAppearanceLocalNodeKey, UiMountedAppearanceStateEntry,
    UiMountedAppearanceStateMembership,
};

#[derive(Clone)]
pub(super) struct UiMountedAppearancePhysicalPredecessor {
    pub(super) key: UiMountedAppearanceLocalNodeKey,
    pub(super) session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(super) sidecar: UiMountedAppearanceSidecar,
}

#[derive(Clone)]
pub(super) enum UiMountedAppearanceStatePredecessor {
    Resolved(UiMountedAppearanceStateEntry),
    PhysicalOnly(UiMountedAppearancePhysicalPredecessor),
}

impl UiMountedAppearanceStatePredecessor {
    pub(super) fn sidecar(&self) -> &UiMountedAppearanceSidecar {
        match self {
            Self::Resolved(entry) => &entry.sidecar,
            Self::PhysicalOnly(physical) => &physical.sidecar,
        }
    }

    pub(super) fn semantic(&self) -> Option<&UiMountedAppearanceStateEntry> {
        match self {
            Self::Resolved(entry) => Some(entry),
            Self::PhysicalOnly(_) => None,
        }
    }

    pub(super) fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        match self {
            Self::Resolved(entry) => entry.key.session,
            Self::PhysicalOnly(physical) => physical.session,
        }
    }

    pub(super) fn into_physical(self) -> Option<UiMountedAppearancePhysicalPredecessor> {
        if self.sidecar().current_node_receipt().is_none() {
            return None;
        }
        Some(match self {
            Self::Resolved(entry) => UiMountedAppearancePhysicalPredecessor {
                key: entry.key.local_node,
                session: entry.key.session,
                sidecar: entry.sidecar,
            },
            Self::PhysicalOnly(physical) => physical,
        })
    }
}

impl UiMountedAppearanceStateMembership {
    pub(super) fn into_predecessor(self) -> Option<UiMountedAppearanceStatePredecessor> {
        match self {
            Self::Retained(entry) => Some(UiMountedAppearanceStatePredecessor::Resolved(entry)),
            Self::PhysicalOnly(physical) => {
                Some(UiMountedAppearanceStatePredecessor::PhysicalOnly(physical))
            }
            Self::Staged { predecessor, .. } => predecessor,
            Self::Reserved => None,
        }
    }
}
