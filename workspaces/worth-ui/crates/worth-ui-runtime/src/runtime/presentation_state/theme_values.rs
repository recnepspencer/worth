use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use super::UiApplicationPresentationState;

#[derive(Clone)]
pub(crate) struct UiApplicationThemeTypedValues {
    capability: crate::runtime::appearance::UiThemeCapabilityReceipt,
    values: Arc<BTreeMap<crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue>>,
}

impl UiApplicationThemeTypedValues {
    fn new(
        capability: &crate::runtime::appearance::UiThemeCapabilityReceipt,
        values: BTreeMap<crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue>,
    ) -> Self {
        Self {
            capability: capability.clone(),
            values: Arc::new(values),
        }
    }

    pub(crate) fn capability(&self) -> &crate::runtime::appearance::UiThemeCapabilityReceipt {
        &self.capability
    }

    pub(crate) fn values(
        &self,
    ) -> Arc<BTreeMap<crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue>> {
        Arc::clone(&self.values)
    }
}

pub(crate) struct UiApplicationThemeValueUpdate {
    predecessor_theme_revision: u64,
    token_values:
        Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
    mutable_token_revisions: BTreeMap<crate::capability::ThemeTokenId, u64>,
    changed_tokens: Box<[crate::capability::ThemeTokenId]>,
    semantic_presentation_revisions: Box<[(Box<str>, u64)]>,
    theme_revision: u64,
    appearance_theme_values:
        BTreeMap<worth_ui_host_contract::UiSemanticSurfaceIdentity, UiApplicationThemeTypedValues>,
}

impl UiApplicationPresentationState {
    pub(crate) fn prepare_theme_values(
        &self,
        changes: &[crate::facade::entry::UiNativeThemeTokenValueChange],
    ) -> Result<UiApplicationThemeValueUpdate, ()> {
        self.prepare_theme_values_inner(changes, None)
    }

    pub(crate) fn prepare_theme_values_for_appearance(
        &self,
        changes: &[crate::facade::entry::UiNativeThemeTokenValueChange],
        themes: &crate::capability::FrozenAppearanceThemeCapabilities,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<UiApplicationThemeValueUpdate, ()> {
        self.prepare_theme_values_inner(changes, Some((themes, generation)))
    }

    fn prepare_theme_values_inner(
        &self,
        changes: &[crate::facade::entry::UiNativeThemeTokenValueChange],
        appearance: Option<(
            &crate::capability::FrozenAppearanceThemeCapabilities,
            &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        )>,
    ) -> Result<UiApplicationThemeValueUpdate, ()> {
        let mut seen = HashSet::with_capacity(changes.len());
        for change in changes {
            if !seen.insert(change.token()) {
                return Err(());
            }
            let revision = self.mutable_token_revisions.get(change.token()).ok_or(())?;
            if *revision != change.expected_revision() {
                return Err(());
            }
        }
        let typed_changes = changes
            .iter()
            .map(|change| {
                let value = match change.value() {
                    crate::capability::ThemeTokenValue::Color(color) => {
                        worth_ui_dsl::UiThemeColor::parse(color.as_str())
                            .map(worth_ui_dsl::UiThemeValue::Color)
                            .map_err(|_| ())?
                    }
                };
                Ok((change.token().clone(), value))
            })
            .collect::<Result<BTreeMap<_, _>, ()>>()?;

        let mut token_values = Arc::clone(&self.token_values);
        let mut mutable_token_revisions = self.mutable_token_revisions.clone();
        let mut changed_tokens = Vec::new();
        let mut changed_targets = Vec::new();
        for change in changes {
            let revision = mutable_token_revisions
                .get_mut(change.token())
                .expect("validated mutable theme token remains installed");
            *revision = revision.checked_add(1).ok_or(())?;
            let affected = self
                .resolved_targets
                .iter()
                .filter_map(|(token, resolved)| {
                    (resolved == change.token()).then_some(token.clone())
                })
                .collect::<Vec<_>>();
            if affected
                .iter()
                .any(|token| token_values.get(token) != Some(change.value()))
            {
                let values = Arc::make_mut(&mut token_values);
                for token in affected {
                    values.insert(token.clone(), change.value().clone());
                    changed_tokens.push(token);
                }
                changed_targets.push(change.token().clone());
            }
        }
        changed_tokens.sort();
        changed_tokens.dedup();
        changed_targets.sort();
        changed_targets.dedup();
        let mut appearance_theme_values = self.appearance_theme_values.clone();
        if let Some((themes, generation)) = appearance {
            let Some(theme_state) = self.appearance_theme_state.as_ref() else {
                return Err(());
            };
            for binding in theme_state.active_bindings() {
                if binding.capability().application() != generation {
                    return Err(());
                }
                let view = crate::runtime::appearance::UiThemeResolutionView::from_capability(
                    binding.capability(),
                    themes,
                )
                .map_err(|_| ())?;
                let mut values = appearance_theme_values
                    .get(&binding.surface())
                    .filter(|current| current.capability == *binding.capability())
                    .map_or_else(BTreeMap::new, |current| (*current.values).clone());
                for (token, value) in &typed_changes {
                    let requested =
                        worth_ui_dsl::UiThemeSlotIdentity::new(token.as_str()).ok_or(())?;
                    let resolved = view.resolve(&requested, value.kind()).map_err(|_| ())?;
                    let terminal =
                        crate::capability::ThemeTokenId::new(resolved.terminal().as_str())
                            .map_err(|_| ())?;
                    values.insert(terminal, *value);
                }
                appearance_theme_values.insert(
                    binding.surface(),
                    UiApplicationThemeTypedValues::new(binding.capability(), values),
                );
            }
        }
        let semantic_presentation_revisions = self
            .rows
            .iter()
            .filter_map(|(identity, row)| {
                let increments = changed_targets
                    .iter()
                    .filter(|target| {
                        row.contract
                            .foreground_tokens()
                            .any(|token| self.resolved_targets.get(token) == Some(*target))
                    })
                    .count();
                (increments != 0).then(|| {
                    let increments = u64::try_from(increments).map_err(|_| ())?;
                    let revision = row
                        .presentation_revision
                        .checked_add(increments)
                        .ok_or(())?;
                    Ok((identity.clone(), revision))
                })
            })
            .collect::<Result<Vec<_>, ()>>()?;
        let theme_revision = if changed_tokens.is_empty() {
            self.theme_revision
        } else {
            self.theme_revision.checked_add(1).ok_or(())?
        };
        Ok(UiApplicationThemeValueUpdate {
            predecessor_theme_revision: self.theme_revision,
            token_values,
            mutable_token_revisions,
            changed_tokens: changed_tokens.into_boxed_slice(),
            semantic_presentation_revisions: semantic_presentation_revisions.into_boxed_slice(),
            theme_revision,
            appearance_theme_values,
        })
    }

    pub(crate) fn commit_theme_values(
        &mut self,
        update: UiApplicationThemeValueUpdate,
        invalidation: Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    ) -> Result<(), ()> {
        if update.predecessor_theme_revision != self.theme_revision {
            return Err(());
        }
        self.token_values = update.token_values;
        self.mutable_token_revisions = update.mutable_token_revisions;
        if update.theme_revision != self.theme_revision {
            for (identity, revision) in update.semantic_presentation_revisions {
                self.rows
                    .get_mut(identity.as_ref())
                    .expect("prepared semantic theme consumer remains installed")
                    .presentation_revision = revision;
            }
            self.theme_revision = update.theme_revision;
            self.appearance_theme_values = update.appearance_theme_values;
            self.pending_theme_tokens
                .extend(update.changed_tokens.iter().cloned());
            if let Some(invalidation) = invalidation {
                self.queue_appearance_invalidation(invalidation)?;
            }
        }
        Ok(())
    }
}

impl UiApplicationThemeValueUpdate {
    pub(crate) fn changed_tokens(&self) -> &[crate::capability::ThemeTokenId] {
        &self.changed_tokens
    }
}
