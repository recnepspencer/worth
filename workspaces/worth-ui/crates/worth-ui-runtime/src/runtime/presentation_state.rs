use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

mod appearance_generation_succession;
mod appearance_theme;
mod overlay_export;
mod projection;
mod semantic_text_tokens;
mod text_publication;
mod text_succession;
pub(crate) use text_publication::{
    UiApplicationTextMountedCoverage, UiApplicationTextPublication,
    UiApplicationTextRevisionSelection,
};
pub(crate) use text_succession::UiPreparedApplicationTextSuccession;
#[cfg(test)]
#[path = "presentation_state_tests.rs"]
mod tests;

pub(crate) use appearance_generation_succession::{
    UiAppearanceGenerationSuccessionDenial, UiPreparedAppearanceGenerationSuccession,
};
pub(crate) use appearance_theme::UiAppearanceThemeBindingDenial;
pub(crate) use overlay_export::UiApplicationPresentationOwnerExport;

pub(crate) struct UiApplicationPresentationState {
    rows: HashMap<Box<str>, UiApplicationSemanticTextRow>,
    token_values:
        Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
    pending_appearance_invalidation:
        Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    next_appearance_batch_revision: u64,
    appearance_theme_state: Option<crate::runtime::appearance::UiAppearanceThemeState>,
}

struct UiApplicationSemanticTextRow {
    graph_node: Option<crate::graph::UiGraphNodeIdentity>,
    value: Option<Arc<str>>,
    contract: crate::capability::ComponentSemanticTextContract,
    semantic_revision: u64,
    presentation_revision: u64,
    projected_presentation_revision: Option<u64>,
    pending_publication_coverage: Option<UiApplicationTextMountedCoverage>,
}

pub(crate) struct UiApplicationPresentationProjection {
    content: crate::mounting::UiMountedSemanticContentInput,
    revisions: Box<[(Box<str>, crate::graph::UiGraphNodeIdentity, u64)]>,
    theme_values: crate::mounting::UiMountedThemeValueSource,
}

impl UiApplicationPresentationState {
    pub(crate) fn activate(capabilities: &crate::capability::CapabilitySnapshot) -> Self {
        let token_values = semantic_text_tokens::admit(capabilities.theme_tokens());
        let rows = capabilities
            .components()
            .descriptors()
            .iter()
            .filter_map(|component| {
                component.semantic_text_contract().map(|contract| {
                    (
                        Box::<str>::from(format!("component:{}", component.id().as_str())),
                        UiApplicationSemanticTextRow {
                            graph_node: None,
                            value: None,
                            contract: contract.clone(),
                            semantic_revision: 0,
                            presentation_revision: 0,
                            projected_presentation_revision: None,
                            pending_publication_coverage: None,
                        },
                    )
                })
            })
            .collect();
        Self {
            rows,
            token_values,
            pending_appearance_invalidation: None,
            next_appearance_batch_revision: 1,
            appearance_theme_state: None,
        }
    }

    pub(crate) fn register_semantic_text(
        &mut self,
        authored_identity: Box<str>,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Result<(), ()> {
        let Some(row) = self.rows.get_mut(authored_identity.as_ref()) else {
            return Ok(());
        };
        if row.graph_node.replace(graph_node).is_some() {
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn admit_semantic_text(
        &mut self,
        changes: &[crate::facade::entry::UiNativeComponentSemanticTextChange],
    ) -> Result<(), ()> {
        let mut seen = HashSet::with_capacity(changes.len());
        let mut contracts = Vec::with_capacity(changes.len());
        for change in changes {
            if !seen.insert(change.authored_semantic_identity()) {
                return Err(());
            }
            let row = self
                .rows
                .get(change.authored_semantic_identity())
                .ok_or(())?;
            if row.semantic_revision != change.expected_revision() || row.graph_node.is_none() {
                return Err(());
            }
            let contract = match change.spans() {
                Some(spans) => self.validate_span_successor(row, change.text(), spans)?,
                None => row.contract.clone(),
            };
            contracts.push(contract);
        }
        for (change, contract) in changes.iter().zip(contracts) {
            let row = self
                .rows
                .get_mut(change.authored_semantic_identity())
                .expect("validated semantic-text row remains installed");
            row.semantic_revision = row.semantic_revision.checked_add(1).ok_or(())?;
            let next: Arc<str> = Arc::from(change.text());
            let changed = row.value.as_deref() != Some(next.as_ref()) || row.contract != contract;
            row.value = Some(next);
            row.contract = contract;
            if changed {
                row.presentation_revision = row.presentation_revision.checked_add(1).ok_or(())?;
                row.pending_publication_coverage = None;
            }
        }
        Ok(())
    }

    fn validate_span_successor(
        &self,
        row: &UiApplicationSemanticTextRow,
        text: &str,
        spans: &[crate::capability::ComponentSemanticTextSpanContract],
    ) -> Result<crate::capability::ComponentSemanticTextContract, ()> {
        let exact_end = u32::try_from(text.len()).map_err(|_| ())?;
        if spans.last().map(|span| span.original_range().end()) != Some(exact_end)
            || spans.iter().any(|span| {
                !text.is_char_boundary(span.original_range().start() as usize)
                    || !text.is_char_boundary(span.original_range().end() as usize)
                    || !self.token_values.contains_key(span.foreground_token())
            })
        {
            return Err(());
        }
        crate::capability::ComponentSemanticTextContract::spanned(
            row.contract.theme_token().clone(),
            row.contract.layer_semantic_order(),
            spans.iter().cloned(),
        )
        .map_err(|_| ())
    }

    pub(crate) fn project(
        &self,
    ) -> Result<UiApplicationPresentationProjection, crate::mounting::UiMountedFramePreparationDenial>
    {
        self.project_rows(|row| {
            row.projected_presentation_revision != Some(row.presentation_revision)
        })
    }

    pub(crate) fn requires_mounted_projection(&self) -> bool {
        self.pending_appearance_invalidation.is_some()
            || self.rows.values().any(|row| {
                row.value.is_some()
                    && row.graph_node.is_some()
                    && row.projected_presentation_revision != Some(row.presentation_revision)
            })
    }

    pub(crate) fn project_complete(
        &self,
    ) -> Result<UiApplicationPresentationProjection, crate::mounting::UiMountedFramePreparationDenial>
    {
        self.project_rows(|_| true)
    }

    fn project_rows(
        &self,
        include: impl Fn(&UiApplicationSemanticTextRow) -> bool,
    ) -> Result<UiApplicationPresentationProjection, crate::mounting::UiMountedFramePreparationDenial>
    {
        UiApplicationPresentationProjection::project_rows(
            self.rows.iter(),
            Arc::clone(&self.token_values),
            include,
        )
    }

    pub(crate) fn appearance_invalidation_batch(
        &self,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        self.pending_appearance_invalidation.clone()
    }

    pub(crate) fn queue_appearance_invalidation(
        &mut self,
        batch: crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) -> Result<(), ()> {
        let (pending, next_revision) = self.prepare_appearance_invalidation(Some(batch))?;
        self.pending_appearance_invalidation = pending;
        self.next_appearance_batch_revision = next_revision;
        Ok(())
    }

    pub(crate) fn prepare_appearance_invalidation(
        &self,
        batch: Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    ) -> Result<
        (
            Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
            u64,
        ),
        (),
    > {
        let Some(batch) = batch else {
            return Ok((
                self.pending_appearance_invalidation.clone(),
                self.next_appearance_batch_revision,
            ));
        };
        let revision = self.next_appearance_batch_revision;
        let next_revision = revision.checked_add(1).ok_or(())?;
        let batch = batch.with_revision(revision);
        let mut pending = self.pending_appearance_invalidation.clone();
        if let Some(current) = pending.as_mut() {
            if current.basis() == batch.basis() {
                current.merge(batch);
            } else {
                *current = batch;
            }
        } else {
            pending = Some(batch);
        }
        Ok((pending, next_revision))
    }

    pub(crate) fn settle_appearance_invalidation(
        &mut self,
        batch: &crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) {
        if self
            .pending_appearance_invalidation
            .as_ref()
            .is_some_and(|pending| pending.revision() == batch.revision())
        {
            self.pending_appearance_invalidation = None;
        }
    }

    pub(crate) fn preview_theme_observation(
        &self,
    ) -> crate::mounting::UiMountedPreviewThemeObservation {
        crate::mounting::UiMountedPreviewThemeObservation::admit_from_presentation(
            0,
            Arc::clone(&self.token_values),
        )
    }
}
