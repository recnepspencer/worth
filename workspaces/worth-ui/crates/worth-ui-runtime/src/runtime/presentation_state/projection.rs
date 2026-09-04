impl super::UiApplicationPresentationProjection {
    pub(crate) fn content(&self) -> crate::mounting::UiMountedSemanticContentInput {
        self.content.clone()
    }

    pub(crate) fn theme_values(&self) -> crate::mounting::UiMountedThemeValueSource {
        self.theme_values.clone()
    }
}

pub(super) fn unknown_graph_node() -> crate::mounting::UiMountedFramePreparationDenial {
    crate::mounting::UiMountedFramePreparationDenial::Projection(
        crate::mounting::UiMountedProjectionDenial::UnknownGraphNode,
    )
}
