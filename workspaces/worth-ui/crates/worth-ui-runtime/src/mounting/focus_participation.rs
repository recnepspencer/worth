#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedFocusParticipant {
    graph_node: crate::graph::UiGraphNodeIdentity,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    support: crate::capability::ComponentFocusSupport,
    container: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    scope: super::projection::UiMountedFocusScope,
    mounted_order: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedFocusParticipationSnapshot {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    participants: Box<[UiMountedFocusParticipant]>,
    nodes_visited: u32,
    retained_surfaces: Box<[worth_ui_host_contract::UiSemanticSurfaceIdentity]>,
}

impl UiMountedFocusParticipant {
    pub(crate) const fn new(
        graph_node: crate::graph::UiGraphNodeIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        support: crate::capability::ComponentFocusSupport,
        scope: super::projection::UiMountedFocusScope,
        mounted_order: u32,
    ) -> Self {
        Self {
            graph_node,
            semantic_surface,
            mounted_instance,
            incarnation,
            node_receipt,
            support,
            container: None,
            scope,
            mounted_order,
        }
    }

    pub(crate) const fn graph_node(self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
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
    pub(crate) const fn incarnation(self) -> worth_ui_host_contract::UiMountIncarnation {
        self.incarnation
    }
    pub(crate) const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub(crate) const fn support(self) -> crate::capability::ComponentFocusSupport {
        self.support
    }
    pub(crate) const fn container(
        self,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.container
    }
    pub(crate) const fn scope(self) -> super::projection::UiMountedFocusScope {
        self.scope
    }
    pub(crate) const fn mounted_order(self) -> u32 {
        self.mounted_order
    }

    #[cfg(test)]
    pub(crate) const fn with_container(
        mut self,
        container: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Self {
        self.container = Some(container);
        self
    }
}

impl UiMountedFocusParticipationSnapshot {
    pub(in crate::mounting) fn from_projection(
        projection: &super::UiMountedProjectionFrame,
        receipts: &super::UiMountedNodeReceiptBasis,
        requested_surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        retained_surfaces: Vec<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
    ) -> Self {
        let mounted_by_graph = projection
            .semantic_projection()
            .nodes_in_mounted_order()
            .filter(|node| requested_surfaces.contains(&node.receipt().semantic_surface()))
            .map(|node| {
                (
                    node.receipt().graph_node(),
                    node.receipt().mounted_instance(),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut nodes_visited = 0_u32;
        let participants = projection
            .semantic_projection()
            .nodes_in_mounted_order()
            .enumerate()
            .filter_map(|(order, node)| {
                nodes_visited = nodes_visited.checked_add(1)?;
                if !requested_surfaces.contains(&node.receipt().semantic_surface()) {
                    return None;
                }
                if node.focus_support == crate::capability::ComponentFocusSupport::NotFocusable {
                    return None;
                }
                if !projection.participates_in_focus(node.receipt().mounted_instance()) {
                    return None;
                }
                let scope = node.focus_scope?;
                let mounted_instance = node.receipt().mounted_instance();
                let container = match node.focus_container_owner {
                    Some(owner) => Some(*mounted_by_graph.get(&owner)?),
                    None => None,
                };
                let mut participant = UiMountedFocusParticipant::new(
                    node.receipt().graph_node(),
                    node.receipt().semantic_surface(),
                    mounted_instance,
                    node.receipt().incarnation(),
                    receipts.receipt_for(mounted_instance)?,
                    node.focus_support,
                    scope,
                    u32::try_from(order).ok()?,
                );
                participant.container = container;
                Some(participant)
            })
            .collect::<Vec<_>>();
        Self::new(
            receipts.frame(),
            participants,
            nodes_visited,
            retained_surfaces,
        )
    }

    pub(crate) fn new(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        participants: Vec<UiMountedFocusParticipant>,
        nodes_visited: u32,
        retained_surfaces: Vec<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
    ) -> Self {
        Self {
            frame,
            participants: participants.into_boxed_slice(),
            nodes_visited,
            retained_surfaces: retained_surfaces.into_boxed_slice(),
        }
    }

    pub(crate) const fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }
    pub(crate) fn participants(&self) -> &[UiMountedFocusParticipant] {
        &self.participants
    }
    pub(crate) const fn nodes_visited(&self) -> u32 {
        self.nodes_visited
    }

    pub(crate) fn retains_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> bool {
        self.retained_surfaces.contains(&surface)
    }
}
