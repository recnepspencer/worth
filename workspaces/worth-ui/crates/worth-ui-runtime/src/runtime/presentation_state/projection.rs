impl super::UiApplicationPresentationProjection {
    #[cfg(test)]
    pub(crate) fn text_revisions_for_test(
        &self,
    ) -> &[(Box<str>, crate::graph::UiGraphNodeIdentity, u64)] {
        &self.revisions
    }

    pub(super) fn project_rows<'a>(
        rows: impl Iterator<Item = (&'a Box<str>, &'a super::UiApplicationSemanticTextRow)>,
        token_values: std::sync::Arc<
            std::collections::BTreeMap<
                crate::capability::ThemeTokenId,
                crate::capability::ThemeTokenValue,
            >,
        >,
        include: impl Fn(&super::UiApplicationSemanticTextRow) -> bool,
    ) -> Result<
        super::UiApplicationPresentationProjection,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let mut content = crate::mounting::UiMountedSemanticContentInput::empty();
        let mut revisions = Vec::new();
        for (identity, row) in rows {
            let (Some(value), Some(graph_node)) = (&row.value, row.graph_node) else {
                continue;
            };
            let token_values = row
                .contract
                .foreground_tokens()
                .map(|token| {
                    token_values
                        .get(token)
                        .cloned()
                        .map(|value| (token.clone(), value))
                        .ok_or_else(unknown_graph_node)
                })
                .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
            content
                .insert_scalar_with_formatting(
                    graph_node,
                    crate::mounting::UiMountedSemanticTextValueDirective::Replace(
                        std::sync::Arc::clone(value),
                    ),
                    std::sync::Arc::from(" "),
                    Some(
                        crate::mounting::UiMountedSemanticTextFormattingDirective::new(
                            row.contract.clone(),
                            token_values,
                        ),
                    ),
                )
                .map_err(|_| unknown_graph_node())?;
            if include(row) {
                revisions.push((identity.clone(), graph_node, row.presentation_revision));
            } else {
                content.retain_application_text_source(graph_node);
            }
        }
        Ok(Self {
            content,
            revisions: revisions.into_boxed_slice(),
            theme_values: crate::mounting::UiMountedThemeValueSource::from_admitted(
                std::sync::Arc::clone(&token_values),
            ),
        })
    }
    pub(crate) fn text_publication(&self) -> super::UiApplicationTextRevisionSelection {
        super::UiApplicationTextRevisionSelection {
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

pub(super) fn unknown_graph_node() -> crate::mounting::UiMountedFramePreparationDenial {
    crate::mounting::UiMountedFramePreparationDenial::Projection(
        crate::mounting::UiMountedProjectionDenial::UnknownGraphNode,
    )
}
