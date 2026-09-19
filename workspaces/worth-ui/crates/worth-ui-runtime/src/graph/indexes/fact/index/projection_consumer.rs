impl super::UiGraphConsumedFactIndex {
    pub(crate) fn consumes_projection(
        &self,
        projection: &worth_ui_query_binding::WorthUiQueryViewIdentity,
        node: crate::graph::UiGraphNodeIdentity,
    ) -> bool {
        self.query_by_projection
            .get(projection)
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry.consumer() == super::UiGraphFactConsumerIdentity::GraphNode(node)
                })
            })
    }
}
