use super::{UiMountedSemanticContentInput, UiMountedSemanticTextContent};

impl UiMountedSemanticContentInput {
    pub(crate) fn merge_application_presentation(
        &mut self,
        presentation: Self,
    ) -> Result<(), crate::mounting::UiMountedProjectionDenial> {
        if !matches!(
            presentation.projection_inputs,
            super::UiMountedProjectionInputTransition::Retain
        ) || !presentation.schema_transitions.is_empty()
        {
            return Err(crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution);
        }
        if presentation.by_graph_node.keys().any(|graph_node| {
            self.by_graph_node.contains_key(graph_node)
                || self.application_text_source.contains_key(graph_node)
        }) {
            return Err(crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution);
        }
        let mut carried = Vec::new();
        for (graph, content) in presentation.application_text_source.iter() {
            if self.application_text_source.contains_key(graph) {
                return Err(crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution);
            }
            match self.by_graph_node.get(graph) {
                // A preserving row keeps the published value, so the retained
                // application source it would restore is not a competing owner.
                Some(row) if row.preserves_published_value() => {}
                Some(_) => {
                    return Err(
                        crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution,
                    )
                }
                None => carried.push((*graph, content.clone())),
            }
        }
        self.by_graph_node.extend(presentation.by_graph_node);
        std::sync::Arc::make_mut(&mut self.application_text_source).extend(carried);
        Ok(())
    }

    pub(crate) fn retain_application_text_source(
        &mut self,
        graph: crate::graph::UiGraphNodeIdentity,
    ) {
        let content = self
            .by_graph_node
            .remove(&graph)
            .expect("application source was materialized from the captured owner row");
        std::sync::Arc::make_mut(&mut self.application_text_source).insert(graph, content);
    }

    pub(in crate::mounting) fn text_for_lowering(
        &self,
        graph: crate::graph::UiGraphNodeIdentity,
        predecessor_available: bool,
    ) -> (Option<&UiMountedSemanticTextContent>, usize) {
        if let Some(content) = self.get(graph) {
            return (Some(content), 0);
        }
        if predecessor_available || self.application_text_source.is_empty() {
            return (None, 0);
        }
        (self.application_text_source.get(&graph), 1)
    }
}
