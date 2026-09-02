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

    pub(crate) const fn role_revision(&self) -> UiAppearanceRoleRevision {
        self.role_revision
    }

    pub(crate) const fn axes(&self) -> super::UiAppearanceStateAxisDemand {
        self.axes
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceStateConsumerSelection {
    basis: crate::graph::UiGraphFactIndexBasis,
    axis: UiAppearanceStateAxis,
    consumers: Box<[UiAppearanceStateConsumer]>,
    cost: UiAppearanceStateConsumerSelectionCost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceStateConsumerSelectionCost {
    index_probes: usize,
    selected_consumers: usize,
    unrelated_neighborhoods_touched: usize,
}

impl UiAppearanceStateConsumerSelection {
    pub(crate) fn new(
        basis: crate::graph::UiGraphFactIndexBasis,
        axis: UiAppearanceStateAxis,
        consumers: Box<[UiAppearanceStateConsumer]>,
    ) -> Self {
        let selected_consumers = consumers.len();
        Self {
            basis,
            axis,
            consumers,
            cost: UiAppearanceStateConsumerSelectionCost {
                index_probes: 1,
                selected_consumers,
                unrelated_neighborhoods_touched: 0,
            },
        }
    }

    pub(crate) const fn basis(&self) -> crate::graph::UiGraphFactIndexBasis {
        self.basis
    }

    pub(crate) const fn axis(&self) -> UiAppearanceStateAxis {
        self.axis
    }

    pub(crate) fn consumers(&self) -> &[UiAppearanceStateConsumer] {
        &self.consumers
    }

    pub(crate) const fn cost(&self) -> UiAppearanceStateConsumerSelectionCost {
        self.cost
    }
}

impl UiAppearanceStateConsumerSelectionCost {
    pub(crate) const fn index_probes(self) -> usize {
        self.index_probes
    }

    pub(crate) const fn selected_consumers(self) -> usize {
        self.selected_consumers
    }

    pub(crate) const fn unrelated_neighborhoods_touched(self) -> usize {
        self.unrelated_neighborhoods_touched
    }
}
