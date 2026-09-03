use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use super::UiApplicationPresentationState;

pub(crate) struct UiApplicationThemeValueUpdate {
    predecessor_theme_revision: u64,
    token_values:
        Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
    mutable_token_revisions: BTreeMap<crate::capability::ThemeTokenId, u64>,
    changed_tokens: Box<[crate::capability::ThemeTokenId]>,
    semantic_presentation_revisions: Box<[(Box<str>, u64)]>,
    theme_revision: u64,
}

impl UiApplicationPresentationState {
    pub(crate) fn prepare_theme_values(
        &self,
        changes: &[crate::facade::entry::UiNativeThemeTokenValueChange],
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
        })
    }

    pub(crate) fn commit_theme_values(
        &mut self,
        update: UiApplicationThemeValueUpdate,
        canonical_selection: crate::runtime::appearance::UiAppearanceConsumerSelection,
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
            self.pending_theme_consumers.merge(canonical_selection);
        }
        Ok(())
    }
}

impl UiApplicationThemeValueUpdate {
    pub(crate) fn changed_tokens(&self) -> &[crate::capability::ThemeTokenId] {
        &self.changed_tokens
    }
}
