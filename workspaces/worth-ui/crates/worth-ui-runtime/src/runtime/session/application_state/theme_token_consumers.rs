use super::WorthUiApplicationSessionState;

impl WorthUiApplicationSessionState {
    pub(crate) fn theme_token_graph_consumers(
        &self,
        token: &crate::capability::ThemeTokenId,
    ) -> Box<[crate::graph::UiGraphNodeIdentity]> {
        let prepared = self.app.prepared_authority();
        let declarations = prepared.authored_declaration_lookup();
        let declaration = declarations
            .theme_token_declaration_identity(token.as_str())
            .unwrap_or(token.as_str());
        let index = prepared.consumed_fact_index();
        index
            .select_appearance_slot_consumers(index.basis(), token.as_str(), declaration)
            .unwrap_or_else(|_| Box::new([]))
    }
}
