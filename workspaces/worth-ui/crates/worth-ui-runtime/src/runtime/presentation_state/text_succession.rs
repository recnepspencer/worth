use super::{
    UiApplicationPresentationProjection, UiApplicationPresentationState,
    UiApplicationSemanticTextRow,
};
use std::collections::HashMap;

pub(crate) struct UiPreparedApplicationTextSuccession {
    rows: HashMap<Box<str>, UiApplicationSemanticTextRow>,
    token_values: std::sync::Arc<
        std::collections::BTreeMap<
            crate::capability::ThemeTokenId,
            crate::capability::ThemeTokenValue,
        >,
    >,
    predecessor: Box<
        [(
            Box<str>,
            Option<crate::graph::UiGraphNodeIdentity>,
            u64,
            u64,
        )],
    >,
}

impl UiApplicationPresentationState {
    pub(crate) fn prepare_text_succession(
        &self,
        predecessor: &crate::capability::CapabilitySnapshot,
        successor: &crate::capability::CapabilitySnapshot,
        graph: crate::graph::UiGraphAuthority<'_>,
    ) -> Result<UiPreparedApplicationTextSuccession, crate::mounting::UiMountedFramePreparationDenial>
    {
        let mut candidate = Self::activate(successor);
        for node in graph.snapshot().nodes() {
            candidate
                .register_semantic_text(
                    node.declaration_identity().authored_semantic_name().into(),
                    node.graph_node_identity(),
                )
                .map_err(|_| super::projection::unknown_graph_node())?;
        }
        for component in successor.components().descriptors() {
            let identity = format!("component:{}", component.id().as_str());
            let Some(row) = candidate
                .rows
                .get_mut(identity.as_str())
                .filter(|row| row.graph_node.is_some())
            else {
                continue;
            };
            let Some(prior) = self
                .rows
                .get(identity.as_str())
                .filter(|row| row.graph_node.is_some())
            else {
                continue;
            };
            row.value = prior.value.clone();
            row.semantic_revision = prior.semantic_revision;
            row.presentation_revision = prior.presentation_revision;
            if predecessor
                .components()
                .get(component.id())
                .and_then(|component| component.semantic_text_contract())
                == component.semantic_text_contract()
            {
                // Owner-issued formatting survives an unrelated source replacement.
                row.contract = prior.contract.clone();
            } else {
                row.presentation_revision = row
                    .presentation_revision
                    .checked_add(1)
                    .ok_or_else(super::projection::unknown_graph_node)?;
            }
        }
        Ok(UiPreparedApplicationTextSuccession {
            rows: candidate.rows,
            token_values: candidate.token_values,
            predecessor: self
                .rows
                .iter()
                .map(|(identity, row)| {
                    (
                        identity.clone(),
                        row.graph_node,
                        row.semantic_revision,
                        row.presentation_revision,
                    )
                })
                .collect(),
        })
    }
}

impl UiPreparedApplicationTextSuccession {
    pub(crate) fn project(
        &self,
    ) -> Result<UiApplicationPresentationProjection, crate::mounting::UiMountedFramePreparationDenial>
    {
        UiApplicationPresentationProjection::project_rows(
            self.rows.iter(),
            self.token_values.clone(),
            |_| true,
        )
    }

    pub(crate) fn is_current(&self, owner: &UiApplicationPresentationState) -> bool {
        self.predecessor.len() == owner.rows.len()
            && self
                .predecessor
                .iter()
                .all(|(identity, graph, semantic, presentation)| {
                    owner.rows.get(identity.as_ref()).is_some_and(|row| {
                        row.graph_node == *graph
                            && row.semantic_revision == *semantic
                            && row.presentation_revision == *presentation
                    })
                })
    }

    pub(crate) fn commit(self, owner: &mut UiApplicationPresentationState) {
        owner.rows = self.rows;
    }
}
