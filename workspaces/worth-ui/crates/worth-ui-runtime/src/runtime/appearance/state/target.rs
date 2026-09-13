#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceTarget {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    component_reference: Option<crate::capability::ComponentId>,
}

impl UiAppearanceTarget {
    pub(crate) fn new(
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> Result<Self, UiAppearanceTargetDenial> {
        if node_receipt.mounted_instance() != mounted_instance {
            return Err(UiAppearanceTargetDenial::ReceiptInstanceMismatch);
        }
        Ok(Self {
            session,
            surface,
            graph_node,
            mounted_instance,
            incarnation,
            node_receipt,
            component_reference: None,
        })
    }

    pub(crate) fn with_component_reference(
        mut self,
        component: crate::capability::ComponentId,
    ) -> Self {
        self.component_reference = Some(component);
        self
    }

    pub(crate) const fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }

    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(crate) const fn incarnation(&self) -> worth_ui_host_contract::UiMountIncarnation {
        self.incarnation
    }

    pub(crate) const fn node_receipt(
        &self,
    ) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) fn component_reference(&self) -> Option<&crate::capability::ComponentId> {
        self.component_reference.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceTargetDenial {
    ReceiptInstanceMismatch,
}
