#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceConsumerSelection {
    consumers: Box<[crate::graph::UiGraphNodeIdentity]>,
    reconstructible: bool,
}

/// One sealed invalidation transaction for a framework appearance turn.
///
/// The consumed-fact index owns both the basis and the canonical attached
/// consumer relation.  Keeping that proof with the batch prevents a theme
/// value source, an owner snapshot, or a replacement candidate from creating
/// a second consumer-selection lane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceInvalidationBatch {
    basis: crate::graph::UiGraphFactIndexBasis,
    consumers: Box<[crate::graph::UiGraphNodeIdentity]>,
    causes: u8,
    reconstructible: bool,
    revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceInvalidationCause {
    Initial = 1,
    OwnerState = 2,
    RoleReplacement = 4,
    ThemeSlot = 8,
}

impl UiAppearanceInvalidationBatch {
    pub(crate) fn initial(index: &crate::graph::UiGraphConsumedFactIndex) -> Self {
        Self::from_index(
            index,
            UiAppearanceInvalidationCause::Initial,
            index.appearance_attached_consumer_nodes(),
        )
    }

    pub(crate) fn owner_state(
        index: &crate::graph::UiGraphConsumedFactIndex,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> Self {
        Self::from_index(
            index,
            UiAppearanceInvalidationCause::OwnerState,
            index.appearance_state_consumer_nodes(axis),
        )
    }

    pub(crate) fn role_replacement(
        index: &crate::graph::UiGraphConsumedFactIndex,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> Self {
        Self::from_index(
            index,
            UiAppearanceInvalidationCause::RoleReplacement,
            index.appearance_role_consumer_nodes(role),
        )
    }

    pub(crate) fn theme_slot(
        index: &crate::graph::UiGraphConsumedFactIndex,
        capability_identity: &str,
        authored_identity: &str,
    ) -> Result<Self, crate::graph::UiGraphFactLookupDenial> {
        Ok(Self::from_index(
            index,
            UiAppearanceInvalidationCause::ThemeSlot,
            index.select_appearance_slot_consumers(
                index.basis(),
                capability_identity,
                authored_identity,
            )?,
        ))
    }

    pub(crate) fn empty(index: &crate::graph::UiGraphConsumedFactIndex) -> Self {
        Self::from_index(
            index,
            UiAppearanceInvalidationCause::Initial,
            Vec::<crate::graph::UiGraphNodeIdentity>::new().into_boxed_slice(),
        )
    }

    fn from_index(
        index: &crate::graph::UiGraphConsumedFactIndex,
        cause: UiAppearanceInvalidationCause,
        consumers: impl Into<Box<[crate::graph::UiGraphNodeIdentity]>>,
    ) -> Self {
        let mut consumers = consumers.into().into_vec();
        consumers.sort_unstable();
        consumers.dedup();
        Self {
            basis: index.basis(),
            consumers: consumers.into_boxed_slice(),
            causes: cause as u8,
            reconstructible: true,
            revision: 0,
        }
    }

    pub(crate) fn merge(&mut self, other: Self) {
        debug_assert_eq!(self.basis, other.basis);
        if self.basis != other.basis {
            return;
        }
        let mut consumers = self.consumers.to_vec();
        consumers.extend_from_slice(&other.consumers);
        consumers.sort_unstable();
        consumers.dedup();
        self.consumers = consumers.into_boxed_slice();
        self.causes |= other.causes;
        self.reconstructible &= other.reconstructible;
        self.revision = self.revision.max(other.revision);
    }

    pub(crate) fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) const fn basis(&self) -> crate::graph::UiGraphFactIndexBasis {
        self.basis
    }

    pub(crate) fn consumers(&self) -> &[crate::graph::UiGraphNodeIdentity] {
        &self.consumers
    }

    pub(crate) fn selects(&self, graph_node: crate::graph::UiGraphNodeIdentity) -> bool {
        self.consumers.binary_search(&graph_node).is_ok()
    }

    pub(crate) const fn selected_count(&self) -> u32 {
        self.consumers.len() as u32
    }

    pub(crate) const fn is_reconstructible(&self) -> bool {
        self.reconstructible
    }

    #[cfg(test)]
    pub(crate) const fn causes(&self) -> u8 {
        self.causes
    }
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
