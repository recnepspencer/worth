use super::{
    UiMountedBackdropIdentity, UiMountedBackdropMechanic, UiMountedOutlineAppearanceMechanic,
    UiMountedPointerAffordanceMechanic, UiMountedPortalSurfaceAppearanceMechanic,
    UiMountedSurfaceAppearanceMechanic, UiMountedTextForegroundAppearanceMechanic,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UiMountedAppearanceMechanicIdentity {
    Surface(crate::UiMountedInstanceIdentity),
    PortalSurface(crate::UiMountedInstanceIdentity),
    Outline(crate::UiMountedInstanceIdentity),
    TextForeground {
        target: crate::UiMountedInstanceIdentity,
        span: crate::UiMountedTextPaintSpanIdentity,
    },
    Pointer {
        pointer: crate::UiHostPointerIdentity,
        surface: crate::UiSemanticSurfaceIdentity,
        target: crate::UiMountedInstanceIdentity,
    },
    Backdrop(UiMountedBackdropIdentity),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiMountedAppearanceMechanic {
    Surface(UiMountedSurfaceAppearanceMechanic),
    PortalSurface(UiMountedPortalSurfaceAppearanceMechanic),
    Outline(UiMountedOutlineAppearanceMechanic),
    TextForeground(UiMountedTextForegroundAppearanceMechanic),
    Pointer(UiMountedPointerAffordanceMechanic),
    Backdrop(UiMountedBackdropMechanic),
}

impl UiMountedAppearanceMechanic {
    #[doc(hidden)]
    pub fn identity(&self) -> UiMountedAppearanceMechanicIdentity {
        match self {
            Self::Surface(mechanic) => UiMountedAppearanceMechanicIdentity::Surface(
                mechanic.node_receipt().mounted_instance(),
            ),
            Self::PortalSurface(mechanic) => {
                UiMountedAppearanceMechanicIdentity::PortalSurface(mechanic.portal_instance())
            }
            Self::Outline(mechanic) => UiMountedAppearanceMechanicIdentity::Outline(
                mechanic.node_receipt().mounted_instance(),
            ),
            Self::TextForeground(mechanic) => UiMountedAppearanceMechanicIdentity::TextForeground {
                target: mechanic.node_receipt().mounted_instance(),
                span: mechanic.paint_span(),
            },
            Self::Pointer(mechanic) => UiMountedAppearanceMechanicIdentity::Pointer {
                pointer: mechanic.pointer(),
                surface: mechanic.surface(),
                target: mechanic.target(),
            },
            Self::Backdrop(mechanic) => {
                UiMountedAppearanceMechanicIdentity::Backdrop(mechanic.identity().clone())
            }
        }
    }

    pub(crate) const fn node_receipt_frame(&self) -> Option<crate::UiMountedFrameIdentity> {
        match self {
            Self::Surface(mechanic) => Some(mechanic.node_receipt().frame()),
            Self::PortalSurface(mechanic) => Some(mechanic.surface().node_receipt().frame()),
            Self::Outline(mechanic) => Some(mechanic.node_receipt().frame()),
            Self::TextForeground(mechanic) => Some(mechanic.node_receipt().frame()),
            Self::Pointer(_) | Self::Backdrop(_) => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiMountedAppearanceMechanicChange {
    Insert(UiMountedAppearanceMechanic),
    Replace {
        predecessor: UiMountedAppearanceMechanicIdentity,
        successor: UiMountedAppearanceMechanic,
    },
    Remove(UiMountedAppearanceMechanicIdentity),
}

impl UiMountedAppearanceMechanicChange {
    #[doc(hidden)]
    pub fn replacement(
        predecessor: UiMountedAppearanceMechanicIdentity,
        successor: UiMountedAppearanceMechanic,
    ) -> Option<Self> {
        (predecessor == successor.identity()).then_some(Self::Replace {
            predecessor,
            successor,
        })
    }

    pub const fn identity(&self) -> Option<&UiMountedAppearanceMechanicIdentity> {
        match self {
            Self::Insert(_) => None,
            Self::Replace { predecessor, .. } | Self::Remove(predecessor) => Some(predecessor),
        }
    }

    pub const fn successor(&self) -> Option<&UiMountedAppearanceMechanic> {
        match self {
            Self::Insert(successor) | Self::Replace { successor, .. } => Some(successor),
            Self::Remove(_) => None,
        }
    }
}
