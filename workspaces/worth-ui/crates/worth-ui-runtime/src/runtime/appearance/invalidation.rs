/// One sealed invalidation transaction for a framework appearance turn.
///
/// The consumed-fact index owns both the basis and the canonical attached
/// consumer relation.  Keeping that proof with the batch prevents a theme
/// value source, an owner snapshot, or a replacement candidate from creating
/// a second consumer-selection lane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceInvalidationBatch {
    basis: crate::graph::UiGraphFactIndexBasis,
    graph_consumers: Box<[crate::graph::UiGraphNodeIdentity]>,
    mounted_consumers: Box<
        [(
            crate::graph::UiGraphNodeIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
        )],
    >,
    semantic_graph_consumers: Box<[crate::graph::UiGraphNodeIdentity]>,
    semantic_mounted_consumers: Box<
        [(
            crate::graph::UiGraphNodeIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
        )],
    >,
    causes: u8,
    revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceInvalidationCause {
    Initial = 1,
    OwnerState = 2,
    RoleReplacement = 4,
    ThemeSlot = 8,
    ProjectionInput = 16,
    Mount = 32,
    TextContent = 64,
    Geometry = 128,
}

impl UiAppearanceInvalidationBatch {
    pub(crate) fn is_physical_input_only(&self) -> bool {
        let physical = UiAppearanceInvalidationCause::TextContent as u8
            | UiAppearanceInvalidationCause::Geometry as u8;
        self.causes != 0 && self.causes & !physical == 0
    }

    pub(crate) fn mounted_geometry(
        index: &crate::graph::UiGraphConsumedFactIndex,
        candidates: impl Iterator<
            Item = (
                crate::graph::UiGraphNodeIdentity,
                worth_ui_host_contract::UiMountedInstanceIdentity,
            ),
        >,
    ) -> Self {
        let mut batch = Self::empty(index);
        batch.causes = UiAppearanceInvalidationCause::Geometry as u8;
        batch.mounted_consumers = candidates
            .filter(|(node, _)| index.has_appearance_attachment(*node))
            .collect();
        batch.canonicalize_mounted_consumers();
        batch
    }

    pub(crate) fn text_content(
        index: &crate::graph::UiGraphConsumedFactIndex,
        content: &crate::mounting::UiMountedSemanticContentInput,
    ) -> Self {
        Self::from_physical_index(
            index,
            UiAppearanceInvalidationCause::TextContent,
            content
                .graph_nodes()
                .filter(|node| index.has_appearance_attachment(*node))
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn mounted_initial(
        index: &crate::graph::UiGraphConsumedFactIndex,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> Self {
        let mut batch = Self::empty(index);
        batch.causes = UiAppearanceInvalidationCause::Mount as u8;
        batch.mounted_consumers = instances
            .iter()
            .filter_map(|instance| {
                let basis = mounted.current_mounted_identity_basis(*instance)?;
                let node = basis.graph_node_identity();
                index
                    .has_appearance_attachment(node)
                    .then_some((node, *instance))
            })
            .collect();
        batch.canonicalize_mounted_consumers();
        batch.semantic_mounted_consumers = batch.mounted_consumers.clone();
        batch
    }

    pub(crate) fn mounted_selection_input(
        index: &crate::graph::UiGraphConsumedFactIndex,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> Self {
        let mut batch = Self::mounted_owner_state(
            index,
            mounted,
            worth_ui_dsl::UiAppearanceStateAxis::Selection,
            instances,
        );
        batch.causes = UiAppearanceInvalidationCause::ProjectionInput as u8;
        batch
    }

    pub(crate) fn mounted_owner_state(
        index: &crate::graph::UiGraphConsumedFactIndex,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
        instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> Self {
        let mut batch = Self::empty(index);
        batch.causes = UiAppearanceInvalidationCause::OwnerState as u8;
        batch.mounted_consumers = instances
            .iter()
            .filter_map(|instance| {
                let basis = mounted.current_mounted_identity_basis(*instance)?;
                let graph_node = basis.graph_node_identity();
                index
                    .consumes_appearance_state(axis, graph_node)
                    .then_some((graph_node, *instance))
            })
            .collect();
        batch.canonicalize_mounted_consumers();
        batch.semantic_mounted_consumers = batch.mounted_consumers.clone();
        batch
    }

    pub(crate) fn initial(index: &crate::graph::UiGraphConsumedFactIndex) -> Self {
        Self::from_semantic_index(
            index,
            UiAppearanceInvalidationCause::Initial,
            index.appearance_attached_consumer_nodes(),
        )
    }

    #[cfg(test)]
    pub(crate) fn owner_state(
        index: &crate::graph::UiGraphConsumedFactIndex,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> Self {
        Self::from_semantic_index(
            index,
            UiAppearanceInvalidationCause::OwnerState,
            index.appearance_state_consumer_nodes(axis),
        )
    }

    pub(crate) fn role_replacement(
        index: &crate::graph::UiGraphConsumedFactIndex,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> Self {
        Self::from_semantic_index(
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
        Ok(Self::from_semantic_index(
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
        Self::from_semantic_index(
            index,
            UiAppearanceInvalidationCause::Initial,
            Vec::<crate::graph::UiGraphNodeIdentity>::new().into_boxed_slice(),
        )
    }

    fn from_semantic_index(
        index: &crate::graph::UiGraphConsumedFactIndex,
        cause: UiAppearanceInvalidationCause,
        graph_consumers: impl Into<Box<[crate::graph::UiGraphNodeIdentity]>>,
    ) -> Self {
        let mut graph_consumers = graph_consumers.into().into_vec();
        graph_consumers.sort_unstable();
        graph_consumers.dedup();
        Self {
            basis: index.basis(),
            semantic_graph_consumers: graph_consumers.clone().into_boxed_slice(),
            graph_consumers: graph_consumers.into_boxed_slice(),
            mounted_consumers: Box::default(),
            semantic_mounted_consumers: Box::default(),
            causes: cause as u8,
            revision: 0,
        }
    }

    fn from_physical_index(
        index: &crate::graph::UiGraphConsumedFactIndex,
        cause: UiAppearanceInvalidationCause,
        graph_consumers: impl Into<Box<[crate::graph::UiGraphNodeIdentity]>>,
    ) -> Self {
        let mut batch = Self::from_semantic_index(index, cause, graph_consumers);
        batch.semantic_graph_consumers = Box::default();
        batch
    }

    pub(crate) fn merge(&mut self, other: Self) {
        debug_assert_eq!(self.basis, other.basis);
        if self.basis != other.basis {
            return;
        }
        let mut graph_consumers = self.graph_consumers.to_vec();
        graph_consumers.extend_from_slice(&other.graph_consumers);
        graph_consumers.sort_unstable();
        graph_consumers.dedup();
        self.graph_consumers = graph_consumers.into_boxed_slice();
        let mut mounted = self.mounted_consumers.to_vec();
        mounted.extend_from_slice(&other.mounted_consumers);
        self.mounted_consumers = mounted.into();
        self.canonicalize_mounted_consumers();
        let mut semantic_graph = self.semantic_graph_consumers.to_vec();
        semantic_graph.extend_from_slice(&other.semantic_graph_consumers);
        semantic_graph.sort_unstable();
        semantic_graph.dedup();
        self.semantic_graph_consumers = semantic_graph.into_boxed_slice();
        let mut semantic_mounted = self.semantic_mounted_consumers.to_vec();
        semantic_mounted.extend_from_slice(&other.semantic_mounted_consumers);
        semantic_mounted.sort_unstable();
        semantic_mounted.dedup();
        semantic_mounted
            .retain(|(node, _)| self.semantic_graph_consumers.binary_search(node).is_err());
        self.semantic_mounted_consumers = semantic_mounted.into_boxed_slice();
        self.causes |= other.causes;
        self.revision = self.revision.max(other.revision);
    }

    fn canonicalize_mounted_consumers(&mut self) {
        let mut mounted = self.mounted_consumers.to_vec();
        mounted.sort_unstable();
        mounted.dedup();
        mounted.retain(|(node, _)| self.graph_consumers.binary_search(node).is_err());
        self.mounted_consumers = mounted.into();
    }

    pub(crate) fn mounted_consumers(
        &self,
    ) -> &[(
        crate::graph::UiGraphNodeIdentity,
        worth_ui_host_contract::UiMountedInstanceIdentity,
    )] {
        &self.mounted_consumers
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

    pub(crate) fn graph_consumers(&self) -> &[crate::graph::UiGraphNodeIdentity] {
        &self.graph_consumers
    }

    pub(crate) fn selects_graph(&self, graph_node: crate::graph::UiGraphNodeIdentity) -> bool {
        self.graph_consumers.binary_search(&graph_node).is_ok()
    }

    pub(crate) fn requires_semantic_resolution(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        self.semantic_graph_consumers
            .binary_search(&graph_node)
            .is_ok()
            || self
                .semantic_mounted_consumers
                .binary_search(&(graph_node, mounted_instance))
                .is_ok()
    }

    pub(crate) const fn selected_count(&self) -> u32 {
        (self.graph_consumers.len() + self.mounted_consumers.len()) as u32
    }

    #[cfg(test)]
    pub(crate) const fn causes(&self) -> u8 {
        self.causes
    }
}
