/// Which of an owner's structurally distinct animatable groups a Motion target
/// names. A mounted component, its separately placed Portal content, and the
/// scrolled content of a Scroll region it owns are three different moving
/// things bound to the same mounted instance, so they are three different
/// targets and never collapse into one another.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiMotionTargetScope {
    Ordinary,
    PortalContents,
    ScrollContents,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiMotionTargetIdentity {
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    owner_key: u64,
    scope: UiMotionTargetScope,
}

impl UiMotionTargetIdentity {
    /// An ordinary mounted owner: the test shape of a target that is neither
    /// Portal content nor a Scroll region's scrolled content.
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn from_mounted_owner(
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        owner_key: u64,
    ) -> Self {
        Self {
            semantic_surface,
            mounted_instance,
            owner_key,
            scope: UiMotionTargetScope::Ordinary,
        }
    }

    /// The owner's separately placed Portal content, not its ordinary mounted surface.
    pub(crate) const fn from_portal_owner(
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        portal_identity: u64,
    ) -> Self {
        Self {
            semantic_surface,
            mounted_instance,
            owner_key: portal_identity,
            scope: UiMotionTargetScope::PortalContents,
        }
    }

    /// The scrolled content group of one exact Scroll region occurrence, not
    /// the region's own mounted surface and not a Portal placed above it. The
    /// stationary viewport clip belongs to the ordinary target and never moves
    /// with this one.
    pub(crate) const fn from_scroll_region_owner(
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        owner_key: u64,
    ) -> Self {
        Self {
            semantic_surface,
            mounted_instance,
            owner_key,
            scope: UiMotionTargetScope::ScrollContents,
        }
    }

    pub(crate) const fn scope(self) -> UiMotionTargetScope {
        self.scope
    }

    pub(crate) const fn is_portal_contents(self) -> bool {
        matches!(self.scope, UiMotionTargetScope::PortalContents)
    }

    pub(crate) const fn semantic_surface(
        self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub(crate) const fn mounted_instance(
        self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(crate) const fn owner_key(self) -> u64 {
        self.owner_key
    }
}
