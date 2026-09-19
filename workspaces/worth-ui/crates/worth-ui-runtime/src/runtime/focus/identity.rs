#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiFocusScopeIdentity {
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    mosaic_owner: Option<crate::graph::UiGraphNodeIdentity>,
    kind: crate::capability::MosaicFocusScopeKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::runtime) struct UiFocusParticipantIdentity(
    worth_ui_host_contract::UiMountedInstanceIdentity,
);

impl UiFocusScopeIdentity {
    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) const fn for_surface(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        Self {
            semantic_surface: surface,
            mosaic_owner: None,
            kind: crate::capability::MosaicFocusScopeKind::ActiveSurfaceScope,
        }
    }

    pub(in crate::runtime) const fn from_mounted(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        scope: crate::mounting::UiMountedFocusScope,
    ) -> Self {
        Self {
            semantic_surface: surface,
            mosaic_owner: scope.mosaic_owner(),
            kind: scope.kind(),
        }
    }

    pub(crate) const fn semantic_surface(
        self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub(crate) const fn kind(self) -> crate::capability::MosaicFocusScopeKind {
        self.kind
    }
}

impl UiFocusParticipantIdentity {
    pub(in crate::runtime) const fn for_mounted_instance(
        mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Self {
        Self(mounted)
    }

    pub(crate) const fn mounted_instance(
        self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.0
    }
}
