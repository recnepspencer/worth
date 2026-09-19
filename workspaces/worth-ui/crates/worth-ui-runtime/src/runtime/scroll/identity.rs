#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiScrollOwnerIdentity {
    Region {
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        region: crate::graph::UiGraphNodeIdentity,
        repeated_instance_digest: u64,
        plan_region_index: u32,
    },
    Surface(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    Viewport(worth_ui_host_contract::UiSemanticSurfaceIdentity),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiScrollOwnerIncarnation(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollOwnerRegistration {
    identity: UiScrollOwnerIdentity,
    incarnation: UiScrollOwnerIncarnation,
    axes: super::UiScrollAxes,
    bounds: super::UiScrollBounds,
    initial_offset: super::UiScrollOffset,
}

impl UiScrollOwnerIdentity {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn region(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        region: crate::graph::UiGraphNodeIdentity,
        repeated_instance_digest: u64,
    ) -> Self {
        Self::Region {
            surface,
            region,
            repeated_instance_digest,
            plan_region_index: 0,
        }
    }

    pub(crate) const fn declared_region(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        region: crate::graph::UiGraphNodeIdentity,
        repeated_instance_digest: u64,
        plan_region_index: u32,
    ) -> Self {
        Self::Region {
            surface,
            region,
            repeated_instance_digest,
            plan_region_index,
        }
    }

    pub(crate) const fn surface(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        Self::Surface(surface)
    }

    pub(crate) const fn viewport(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        Self::Viewport(surface)
    }

    pub(crate) const fn semantic_surface(
        self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        match self {
            Self::Region { surface, .. } | Self::Surface(surface) | Self::Viewport(surface) => {
                surface
            }
        }
    }

    pub(crate) const fn allocation_graph_node(
        self,
        target: crate::graph::UiGraphNodeIdentity,
    ) -> crate::graph::UiGraphNodeIdentity {
        match self {
            Self::Region { region, .. } => region,
            Self::Surface(_) | Self::Viewport(_) => target,
        }
    }
}

impl UiScrollOwnerIncarnation {
    pub(crate) const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn as_u64(self) -> u64 {
        self.0
    }

    pub(crate) fn from_mount_incarnation(
        incarnation: worth_ui_host_contract::UiMountIncarnation,
    ) -> Self {
        Self(incarnation.diagnostic_value())
    }
}

impl UiScrollOwnerRegistration {
    pub(crate) const fn new(
        identity: UiScrollOwnerIdentity,
        incarnation: UiScrollOwnerIncarnation,
        axes: super::UiScrollAxes,
        bounds: super::UiScrollBounds,
        initial_offset: super::UiScrollOffset,
    ) -> Self {
        Self {
            identity,
            incarnation,
            axes,
            bounds,
            initial_offset,
        }
    }

    pub(super) const fn identity(self) -> UiScrollOwnerIdentity {
        self.identity
    }

    pub(super) const fn incarnation(self) -> UiScrollOwnerIncarnation {
        self.incarnation
    }

    pub(super) const fn axes(self) -> super::UiScrollAxes {
        self.axes
    }

    pub(super) const fn bounds(self) -> super::UiScrollBounds {
        self.bounds
    }

    pub(super) const fn initial_offset(self) -> super::UiScrollOffset {
        self.initial_offset
    }
}
