use crate::capability::UiSemanticInteractionFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiIntentRouteBinding {
    graph_node: crate::graph::UiGraphNodeIdentity,
    declaration_index: u32,
    interaction: UiSemanticInteractionFamily,
    portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
}

impl UiIntentRouteBinding {
    pub(crate) const fn new(
        graph_node: crate::graph::UiGraphNodeIdentity,
        declaration_index: u32,
        interaction: UiSemanticInteractionFamily,
        portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    ) -> Self {
        Self {
            graph_node,
            declaration_index,
            interaction,
            portal_declaration,
        }
    }

    pub fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub fn interaction(&self) -> UiSemanticInteractionFamily {
        self.interaction
    }

    pub(crate) const fn declaration_index(&self) -> u32 {
        self.declaration_index
    }

    pub(crate) const fn portal_declaration(&self) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.portal_declaration
    }
}
