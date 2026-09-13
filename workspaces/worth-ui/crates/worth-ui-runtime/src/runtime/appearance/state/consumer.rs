use worth_ui_dsl::{
    UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity, UiAppearanceRoleRevision,
    UiAppearanceStateAxis,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct UiAppearanceSelectionSelector {
    owner: crate::runtime::selection::UiSelectionOwnerIdentity,
    key: crate::runtime::selection::UiSelectionStableKey,
    incarnation: crate::runtime::selection::UiSelectionOwnerIncarnation,
}

impl UiAppearanceSelectionSelector {
    pub(crate) const fn new(
        owner: crate::runtime::selection::UiSelectionOwnerIdentity,
        key: crate::runtime::selection::UiSelectionStableKey,
        incarnation: crate::runtime::selection::UiSelectionOwnerIncarnation,
    ) -> Self {
        Self {
            owner,
            key,
            incarnation,
        }
    }

    pub(crate) const fn owner(self) -> crate::runtime::selection::UiSelectionOwnerIdentity {
        self.owner
    }

    pub(crate) const fn key(self) -> crate::runtime::selection::UiSelectionStableKey {
        self.key
    }

    pub(crate) const fn incarnation(
        self,
    ) -> crate::runtime::selection::UiSelectionOwnerIncarnation {
        self.incarnation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceStateConsumer {
    graph_node: crate::graph::UiGraphNodeIdentity,
    role: UiAppearanceRoleIdentity,
    role_revision: UiAppearanceRoleRevision,
    axes: super::UiAppearanceStateAxisDemand,
}

impl UiAppearanceStateConsumer {
    pub(crate) fn from_role(
        graph_node: crate::graph::UiGraphNodeIdentity,
        role: &UiAppearanceRoleDeclaration,
    ) -> Self {
        let mut axes = super::UiAppearanceStateAxisDemand::default();
        for (_, partition) in role.partitions() {
            for axis in partition.axes() {
                axes.include(axis.axis());
            }
        }
        Self {
            graph_node,
            role: role.role().clone(),
            role_revision: role.revision(),
            axes,
        }
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn role(&self) -> &UiAppearanceRoleIdentity {
        &self.role
    }

    pub(crate) const fn consumes(&self, axis: UiAppearanceStateAxis) -> bool {
        self.axes.contains(axis)
    }

    #[cfg(test)]
    pub(crate) fn all_axes_for_test(graph_node: crate::graph::UiGraphNodeIdentity) -> Self {
        let mut axes = super::UiAppearanceStateAxisDemand::default();
        for axis in [
            UiAppearanceStateAxis::Operability,
            UiAppearanceStateAxis::Focus,
            UiAppearanceStateAxis::Validation,
            UiAppearanceStateAxis::Selection,
            UiAppearanceStateAxis::Hover,
            UiAppearanceStateAxis::Pressed,
        ] {
            axes.include(axis);
        }
        Self {
            graph_node,
            role: UiAppearanceRoleIdentity::new("appearance-state-test-role")
                .expect("test appearance role identity"),
            role_revision: UiAppearanceRoleRevision::new(1).expect("test role revision"),
            axes,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_demand_for_test(
        graph_node: crate::graph::UiGraphNodeIdentity,
        axes: super::UiAppearanceStateAxisDemand,
    ) -> Self {
        Self {
            graph_node,
            role: UiAppearanceRoleIdentity::new("appearance-state-test-role")
                .expect("test appearance role identity"),
            role_revision: UiAppearanceRoleRevision::new(1).expect("test role revision"),
            axes,
        }
    }
}
