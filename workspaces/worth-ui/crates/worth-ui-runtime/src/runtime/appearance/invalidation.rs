#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceConsumerSelection {
    consumers: Box<[crate::graph::UiGraphNodeIdentity]>,
    reconstructible: bool,
}

impl UiAppearanceConsumerSelection {
    pub(crate) fn for_state(
        index: &crate::graph::UiGraphConsumedFactIndex,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> Self {
        Self::from_nodes(index.appearance_state_consumer_nodes(axis))
    }

    pub(crate) fn for_role(
        index: &crate::graph::UiGraphConsumedFactIndex,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> Self {
        Self::from_nodes(index.appearance_role_consumer_nodes(role))
    }

    pub(crate) fn for_slot(
        index: &crate::graph::UiGraphConsumedFactIndex,
        authored_identity: &str,
    ) -> Self {
        let fact = crate::fact_contract::UiProducedFact::AuthoredSource(
            crate::fact_contract::UiAuthoredChangedFact::new(
                crate::fact_contract::UiAuthoredFactSelector::node(authored_identity),
                crate::fact_contract::UiAuthoredFactKind::SemanticsChanged,
            ),
        );
        let consumers = index
            .lookup_retained(&fact)
            .map(|receipt| {
                receipt
                    .entries()
                    .iter()
                    .filter_map(|entry| match entry.consumer() {
                        crate::graph::UiGraphFactConsumerIdentity::GraphNode(node) => Some(node),
                        crate::graph::UiGraphFactConsumerIdentity::MountEligibilitySlot(_) => None,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Self::from_nodes(consumers.into_boxed_slice())
    }

    fn from_nodes(consumers: Box<[crate::graph::UiGraphNodeIdentity]>) -> Self {
        Self {
            consumers,
            reconstructible: true,
        }
    }

    pub(crate) fn consumers(&self) -> &[crate::graph::UiGraphNodeIdentity] {
        &self.consumers
    }

    pub(crate) const fn is_reconstructible(&self) -> bool {
        self.reconstructible
    }

    pub(crate) const fn selected_count(&self) -> u32 {
        self.consumers.len() as u32
    }
}
