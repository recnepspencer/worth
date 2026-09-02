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
        let fact = crate::fact_contract::UiProducedFact::AuthoredSource(
            crate::fact_contract::UiAuthoredChangedFact::new(
                crate::fact_contract::UiAuthoredFactSelector::node(declaration),
                crate::fact_contract::UiAuthoredFactKind::SemanticsChanged,
            ),
        );
        let index = prepared.consumed_fact_index();
        let mut consumers = match index.lookup_retained(&fact) {
            Ok(receipt) => receipt
                .entries()
                .iter()
                .filter_map(|entry| match entry.consumer() {
                    crate::graph::UiGraphFactConsumerIdentity::GraphNode(node) => Some(node),
                    crate::graph::UiGraphFactConsumerIdentity::MountEligibilitySlot(_) => None,
                })
                .collect::<Vec<_>>(),
            Err(crate::graph::UiGraphFactLookupDenial::UnknownAuthoredDeclaration { .. }) => {
                Vec::new()
            }
            Err(crate::graph::UiGraphFactLookupDenial::BasisMismatch { .. }) => {
                unreachable!("an index always accepts its own retained basis")
            }
        };
        consumers.sort();
        consumers.dedup();
        consumers.into_boxed_slice()
    }
}
