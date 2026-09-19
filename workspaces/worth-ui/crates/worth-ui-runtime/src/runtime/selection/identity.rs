#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiSelectionStableKey(crate::runtime::UiApplicationItemKey);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiSelectionOwnerIdentity {
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    item_key_family: crate::runtime::UiApplicationItemKeyFamily,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiSelectionOwnerIncarnation(u64);

impl UiSelectionStableKey {
    pub(crate) const fn new(key: crate::runtime::UiApplicationItemKey) -> Self {
        Self(key)
    }

    pub(super) const fn family(self) -> crate::runtime::UiApplicationItemKeyFamily {
        self.0.family()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) const fn application_value(self) -> core::num::NonZeroU64 {
        self.0.value()
    }

    pub(in crate::runtime) const fn application_key(self) -> crate::runtime::UiApplicationItemKey {
        self.0
    }
}

impl UiSelectionOwnerIdentity {
    pub(crate) const fn new(
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        item_key_family: crate::runtime::UiApplicationItemKeyFamily,
    ) -> Self {
        Self {
            semantic_surface,
            graph_node,
            item_key_family,
        }
    }

    pub(super) const fn item_key_family(self) -> crate::runtime::UiApplicationItemKeyFamily {
        self.item_key_family
    }

    pub(crate) const fn semantic_surface(
        self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub(crate) const fn graph_node(self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn key_family(self) -> crate::runtime::UiApplicationItemKeyFamily {
        self.item_key_family
    }
}

impl UiSelectionOwnerIncarnation {
    pub(crate) const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub(crate) const fn from_mount_incarnation(
        incarnation: worth_ui_host_contract::UiMountIncarnation,
    ) -> Self {
        Self(incarnation.diagnostic_value())
    }
}
