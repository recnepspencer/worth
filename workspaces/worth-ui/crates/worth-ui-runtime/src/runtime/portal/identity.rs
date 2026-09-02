#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiPortalOwnerIdentity {
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiPortalIdentity {
    owner: UiPortalOwnerIdentity,
    diagnostic_value: u64,
}

impl UiPortalOwnerIdentity {
    #[cfg(any(test, feature = "certification-support"))]
    pub(super) fn for_test(graph_node: u64, _mounted_instance: u64) -> Self {
        Self {
            graph_node: crate::graph::UiGraphNodeIdentity::new(graph_node),
            mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound()
                .expect("test mounted instance identity capacity"),
        }
    }

    pub(crate) fn from_target(
        graph_node: crate::graph::UiGraphNodeIdentity,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    ) -> Self {
        Self::from_mounted_owner(graph_node, target.mounted_instance())
    }

    pub(crate) const fn from_mounted_owner(
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Self {
        Self {
            graph_node,
            mounted_instance,
        }
    }

    pub(crate) const fn graph_node(self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn mounted_instance_identity(
        self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }
}

impl UiPortalIdentity {
    #[cfg(test)]
    pub(crate) fn for_test(graph_node: u64, mounted_instance: u64) -> Self {
        Self::for_owner(UiPortalOwnerIdentity::for_test(
            graph_node,
            mounted_instance,
        ))
    }

    pub(crate) fn for_owner(owner: UiPortalOwnerIdentity) -> Self {
        let diagnostic_value = owner.graph_node.digest().rotate_left(17)
            ^ owner
                .mounted_instance
                .diagnostic_value()
                .wrapping_mul(0x9e37_79b9_7f4a_7c15);
        Self {
            owner,
            diagnostic_value,
        }
    }

    pub(crate) const fn owner(self) -> UiPortalOwnerIdentity {
        self.owner
    }

    pub(crate) const fn diagnostic_value(self) -> u64 {
        self.diagnostic_value
    }
}
