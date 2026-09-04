use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use super::UiApplicationPresentationState;

#[path = "theme_values/appearance_effect.rs"]
mod appearance_effect;

#[cfg(test)]
#[path = "theme_values/tests.rs"]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub(super) fn for_successor_application(
        &self,
        capability: &crate::runtime::appearance::UiThemeCapabilityReceipt,
    ) -> Self {
        Self {
            capability: capability.clone(),
            values: Arc::clone(&self.values),
        }
    }
}

#[cfg(test)]
impl UiApplicationPresentationState {
    pub(crate) fn replace_appearance_theme_values_for_test(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        values: BTreeMap<crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue>,
    ) {
        let capability = self
            .active_appearance_theme_binding(surface)
            .expect("the test fault must retain an active owner binding")
            .capability()
            .clone();
        self.appearance_theme_values.insert(
            surface,
            UiApplicationThemeTypedValues::new(&capability, values),
        );
    }

    pub(crate) fn remove_appearance_theme_values_for_test(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) {
        self.appearance_theme_values.remove(&surface);
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
            if *revision != change.expected_revision() || revision.checked_add(1).is_none() {
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
                    crate::capability::ThemeTokenValue::Typed(value) => *value,
                };
                Ok((change.token().clone(), value))
            })
            .collect::<Result<Vec<_>, ()>>()?;

        let appearance_effect = match appearance {
            Some((themes, generation)) => {
                let state = self.appearance_theme_state.as_ref().ok_or(())?;
                Some(appearance_effect::prepare_successor(
                    state,
                    &self.appearance_theme_values,
                    themes,
                    generation,
                    &typed_changes,
                )?)
            }
            None => None,
        };

        let mut token_values = Arc::clone(&self.token_values);
        let mut mutable_token_revisions = self.mutable_token_revisions.clone();
        let mut changed_tokens = Vec::new();
        let mut changed_targets = Vec::new();
        for change in changes {
            let revision = mutable_token_revisions
                .get_mut(change.token())
                .expect("validated mutable theme token remains installed");
            *revision = revision.checked_add(1).ok_or(())?;
            let affected = self.aliases_for_terminal(change.token());
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
        changed_targets.sort();
        changed_targets.dedup();
        let (appearance_theme_values, appearance_changed_terminals) = match appearance_effect {
            Some(effect) => (effect.values, effect.changed_terminals),
            None => (self.appearance_theme_values.clone(), Vec::new()),
        };
        for terminal in appearance_changed_terminals {
            changed_tokens.extend(self.aliases_for_terminal(&terminal));
        }
        changed_tokens.sort();
        changed_tokens.dedup();
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

    fn aliases_for_terminal(
        &self,
        terminal: &crate::capability::ThemeTokenId,
    ) -> Vec<crate::capability::ThemeTokenId> {
        self.resolved_targets
            .iter()
            .filter_map(|(token, resolved)| (resolved == terminal).then_some(token.clone()))
            .collect()
    }

    pub(crate) fn commit_theme_values(
        &mut self,
        update: UiApplicationThemeValueUpdate,
        invalidation: Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    ) -> Result<(), ()> {
        if update.predecessor_theme_revision != self.theme_revision {
            return Err(());
        }
        let prepared_invalidation = if update.theme_revision != self.theme_revision {
            match invalidation.filter(|batch| batch.selected_count() != 0) {
                Some(batch) => Some(self.prepare_appearance_invalidation(Some(batch))?),
                None => None,
            }
        } else {
            None
        };
        self.token_values = update.token_values;
        self.mutable_token_revisions = update.mutable_token_revisions;
        for (identity, revision) in update.semantic_presentation_revisions {
            self.rows
                .get_mut(identity.as_ref())
                .expect("prepared semantic theme consumer remains installed")
                .presentation_revision = revision;
        }
        self.theme_revision = update.theme_revision;
        self.appearance_theme_values = update.appearance_theme_values;
        if let Some((pending_invalidation, next_batch_revision)) = prepared_invalidation {
            self.pending_theme_tokens
                .extend(update.changed_tokens.iter().cloned());
            self.pending_appearance_invalidation = pending_invalidation;
            self.next_appearance_batch_revision = next_batch_revision;
        }
        Ok(())
    }
}

impl UiApplicationThemeValueUpdate {
    pub(crate) fn changed_tokens(&self) -> &[crate::capability::ThemeTokenId] {
        &self.changed_tokens
    }
}
