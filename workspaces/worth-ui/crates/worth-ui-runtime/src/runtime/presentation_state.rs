use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

mod appearance_generation_succession;
mod appearance_theme;
mod overlay_export;
mod projection;
#[path = "presentation_state/theme_values.rs"]
mod theme_values;

#[cfg(test)]
#[path = "presentation_state_tests.rs"]
mod tests;

pub(crate) use appearance_generation_succession::{
    UiAppearanceGenerationSuccessionDenial, UiPreparedAppearanceGenerationSuccession,
};
#[cfg(test)]
pub(crate) use appearance_theme::UiAppearanceThemeBindingDenial;
pub(crate) use overlay_export::UiApplicationPresentationOwnerExport;

pub(crate) struct UiApplicationPresentationState {
    rows: HashMap<Box<str>, UiApplicationSemanticTextRow>,
    token_values:
        Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
    resolved_targets: BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenId>,
    mutable_token_revisions: BTreeMap<crate::capability::ThemeTokenId, u64>,
    theme_revision: u64,
    pending_appearance_invalidation:
        Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    pending_theme_tokens: std::collections::BTreeSet<crate::capability::ThemeTokenId>,
    next_appearance_batch_revision: u64,
    appearance_theme_values: BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        theme_values::UiApplicationThemeTypedValues,
    >,
    #[allow(
        dead_code,
        reason = "milestone 3.16 Gate 0 places the future binding owner without activating switching"
    )]
    appearance_theme_state: Option<crate::runtime::appearance::UiAppearanceThemeState>,
}

struct UiApplicationSemanticTextRow {
    graph_node: Option<crate::graph::UiGraphNodeIdentity>,
    value: Option<Arc<str>>,
    contract: crate::capability::ComponentSemanticTextContract,
    semantic_revision: u64,
    presentation_revision: u64,
    projected_presentation_revision: Option<u64>,
}

pub(crate) struct UiApplicationPresentationProjection {
    content: crate::mounting::UiMountedSemanticContentInput,
    revisions: Box<[(Box<str>, crate::graph::UiGraphNodeIdentity, u64)]>,
    theme_values: crate::mounting::UiMountedThemeValueSource,
}

pub(crate) struct UiApplicationTextPublication {
    revisions: Box<[(Box<str>, crate::graph::UiGraphNodeIdentity, u64)]>,
}

impl UiApplicationPresentationState {
    pub(crate) fn activate(capabilities: &crate::capability::CapabilitySnapshot) -> Self {
        let mut token_values = BTreeMap::new();
        let mut resolved_targets = BTreeMap::new();
        let mut mutable_token_revisions = BTreeMap::new();
        for entry in capabilities.theme_tokens().entries() {
            let target = entry.resolved_target_id().clone();
            let value = capabilities
                .theme_tokens()
                .get(&target)
                .and_then(crate::capability::ThemeTokenDescriptor::value)
                .expect("frozen theme-token alias target has a value")
                .clone();
            token_values.insert(entry.descriptor().id().clone(), value);
            resolved_targets.insert(entry.descriptor().id().clone(), target);
            if entry.descriptor().source().is_application_owned()
                && entry.descriptor().alias_target().is_none()
            {
                mutable_token_revisions.insert(entry.descriptor().id().clone(), 0);
            }
        }
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
                        },
                    )
                })
            })
            .collect();
        Self {
            rows,
            token_values: Arc::new(token_values),
            resolved_targets,
            mutable_token_revisions,
            theme_revision: 0,
            pending_appearance_invalidation: None,
            pending_theme_tokens: std::collections::BTreeSet::new(),
            next_appearance_batch_revision: 1,
            appearance_theme_values: BTreeMap::new(),
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
        let mut content = crate::mounting::UiMountedSemanticContentInput::empty();
        let mut revisions = Vec::new();
        for (identity, row) in &self.rows {
            if !include(row) {
                continue;
            }
            let (Some(value), Some(graph_node)) = (&row.value, row.graph_node) else {
                continue;
            };
            let token_values = row
                .contract
                .foreground_tokens()
                .map(|token| {
                    self.token_values
                        .get(token)
                        .cloned()
                        .map(|value| (token.clone(), value))
                        .ok_or_else(projection::unknown_graph_node)
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            content
                .insert_scalar_with_formatting(
                    graph_node,
                    crate::mounting::UiMountedSemanticTextValueDirective::Replace(Arc::clone(
                        value,
                    )),
                    Arc::from(" "),
                    Some(
                        crate::mounting::UiMountedSemanticTextFormattingDirective::new(
                            row.contract.clone(),
                            token_values,
                        ),
                    ),
                )
                .map_err(|_| projection::unknown_graph_node())?;
            revisions.push((identity.clone(), graph_node, row.presentation_revision));
        }
        Ok(UiApplicationPresentationProjection {
            content,
            revisions: revisions.into_boxed_slice(),
            theme_values: self.theme_values_source(),
        })
    }

    pub(crate) fn theme_values_source(&self) -> crate::mounting::UiMountedThemeValueSource {
        crate::mounting::UiMountedThemeValueSource::from_current_with_changes(
            Arc::clone(&self.token_values),
            self.pending_theme_tokens.clone(),
            self.theme_revision,
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

    pub(crate) fn settle_published_theme_values(&mut self, expected_revision: u64) {
        if expected_revision == self.theme_revision {
            self.pending_theme_tokens.clear();
        }
    }

    pub(crate) fn preview_theme_observation(
        &self,
    ) -> crate::mounting::UiMountedPreviewThemeObservation {
        crate::mounting::UiMountedPreviewThemeObservation::admit_from_presentation(
            self.theme_revision,
            Arc::clone(&self.token_values),
        )
    }

    #[cfg(test)]
    pub(crate) fn commit(&mut self, projection: &UiApplicationPresentationProjection) {
        self.settle_published_text(&projection.text_publication());
    }

    pub(crate) fn settle_published_text(&mut self, publication: &UiApplicationTextPublication) {
        for (identity, graph_node, revision) in &publication.revisions {
            if let Some(row) = self.rows.get_mut(identity.as_ref()) {
                if row.graph_node == Some(*graph_node) && row.presentation_revision == *revision {
                    row.projected_presentation_revision = Some(*revision);
                }
            }
        }
    }
}
