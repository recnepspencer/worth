impl super::UiApplicationPresentationProjection {
    pub(crate) fn text_publication(&self) -> super::UiApplicationTextPublication {
        super::UiApplicationTextPublication {
            revisions: self.revisions.clone(),
        }
    }

    pub(crate) fn content(&self) -> crate::mounting::UiMountedSemanticContentInput {
        self.content.clone()
    }

    pub(crate) fn theme_values(&self) -> crate::mounting::UiMountedThemeValueSource {
        self.theme_values.clone()
    }
}

impl super::UiApplicationTextPublication {
    pub(crate) fn retain_complete_graphs(
        self,
        mut complete: impl FnMut(crate::graph::UiGraphNodeIdentity) -> bool,
    ) -> Self {
        Self {
            revisions: self
                .revisions
                .into_vec()
                .into_iter()
                .filter(|(_, graph, _)| complete(*graph))
                .collect(),
        }
    }
}

pub(super) fn unknown_graph_node() -> crate::mounting::UiMountedFramePreparationDenial {
    crate::mounting::UiMountedFramePreparationDenial::Projection(
        crate::mounting::UiMountedProjectionDenial::UnknownGraphNode,
    )
}
