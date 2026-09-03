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

    pub(crate) fn try_for_slot(
        index: &crate::graph::UiGraphConsumedFactIndex,
        capability_identity: &str,
        authored_identity: &str,
    ) -> Result<Self, crate::graph::UiGraphFactLookupDenial> {
        let consumers = index.select_appearance_slot_consumers(
            index.basis(),
            capability_identity,
            authored_identity,
        )?;
        Ok(Self::from_nodes(consumers))
    }

    pub(crate) fn empty() -> Self {
        Self::from_nodes(Box::new([]))
    }

    pub(crate) fn merge(&mut self, other: Self) {
        let mut consumers = self.consumers.to_vec();
        consumers.extend_from_slice(&other.consumers);
        consumers.sort_unstable();
        consumers.dedup();
        self.consumers = consumers.into_boxed_slice();
        self.reconstructible &= other.reconstructible;
    }

    fn from_nodes(consumers: Box<[crate::graph::UiGraphNodeIdentity]>) -> Self {
        let mut consumers = consumers.into_vec();
        consumers.sort_unstable();
        consumers.dedup();
        Self {
            consumers: consumers.into_boxed_slice(),
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

    #[cfg(test)]
    pub(crate) fn for_test(
        consumers: impl IntoIterator<Item = crate::graph::UiGraphNodeIdentity>,
    ) -> Self {
        Self::from_nodes(consumers.into_iter().collect::<Vec<_>>().into_boxed_slice())
    }
}
