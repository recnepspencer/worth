use std::collections::BTreeMap;
use std::sync::Arc;

/// An immutable theme observation admitted by the presentation owner.
///
/// It contains only the already-admitted values and revision. The mounted
/// preview binds it to the surface after mounted identity validation; the
/// observation cannot resolve a role, select consumers, or publish state.
#[derive(Clone)]
pub(crate) struct UiMountedPreviewThemeObservation {
    theme_revision: u64,
    values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
}

impl UiMountedPreviewThemeObservation {
    pub(crate) fn admit_from_presentation(
        theme_revision: u64,
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
    ) -> Self {
        Self {
            theme_revision,
            values,
        }
    }

    pub(crate) fn bind_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> UiMountedPreviewThemeBinding {
        UiMountedPreviewThemeBinding {
            surface,
            theme_revision: self.theme_revision,
            values: Arc::clone(&self.values),
        }
    }
}

/// An immutable theme observation bound to one mounted-preview surface.
///
/// The presentation owner supplies the admitted values. This binding only
/// lets the preview read those values; it cannot resolve a role, select
/// consumers, or publish any runtime state.
#[derive(Clone)]
pub(crate) struct UiMountedPreviewThemeBinding {
    #[allow(
        dead_code,
        reason = "The existing preview seam carries surface provenance for the later appearance consumer"
    )]
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    #[allow(
        dead_code,
        reason = "The existing preview seam carries theme revision provenance for the later appearance consumer"
    )]
    theme_revision: u64,
    values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
}

impl UiMountedPreviewThemeBinding {
    #[cfg(test)]
    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }

    #[cfg(test)]
    pub(crate) const fn theme_revision(&self) -> u64 {
        self.theme_revision
    }

    pub(crate) fn current_value(
        &self,
        token: &crate::capability::ThemeTokenId,
    ) -> Option<&crate::capability::ThemeTokenValue> {
        self.values.get(token)
    }
}

#[derive(Clone)]
pub(crate) enum UiMountedThemeValueSource {
    ActiveCurrent {
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
        changed_tokens: Arc<std::collections::BTreeSet<crate::capability::ThemeTokenId>>,
    },
    ReplacementCandidateFrozenPlan,
    PreviewOnly {
        binding: UiMountedPreviewThemeBinding,
    },
}

impl UiMountedThemeValueSource {
    pub(crate) fn from_current(
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
    ) -> Self {
        Self::ActiveCurrent {
            values,
            changed_tokens: Arc::new(std::collections::BTreeSet::new()),
        }
    }

    pub(crate) fn from_current_with_changes(
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
        changed_tokens: std::collections::BTreeSet<crate::capability::ThemeTokenId>,
    ) -> Self {
        Self::ActiveCurrent {
            values,
            changed_tokens: Arc::new(changed_tokens),
        }
    }

    pub(crate) const fn replacement_candidate_frozen_plan() -> Self {
        Self::ReplacementCandidateFrozenPlan
    }

    pub(crate) fn preview_only(binding: UiMountedPreviewThemeBinding) -> Self {
        Self::PreviewOnly { binding }
    }

    pub(crate) fn current_value(
        &self,
        token: &crate::capability::ThemeTokenId,
    ) -> Option<&crate::capability::ThemeTokenValue> {
        match self {
            Self::ActiveCurrent { values, .. } => values.get(token),
            Self::PreviewOnly { binding } => binding.current_value(token),
            Self::ReplacementCandidateFrozenPlan => None,
        }
    }

    pub(crate) const fn uses_frozen_plan(&self) -> bool {
        matches!(self, Self::ReplacementCandidateFrozenPlan)
    }

    pub(crate) fn has_theme_changes(&self) -> bool {
        match self {
            Self::ActiveCurrent { changed_tokens, .. } => !changed_tokens.is_empty(),
            Self::ReplacementCandidateFrozenPlan | Self::PreviewOnly { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_values_require_an_explicit_non_authoritative_binding() {
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let token = crate::capability::ThemeTokenId::new("preview.surface").unwrap();
        let value = crate::capability::ThemeTokenValue::color(
            crate::capability::ThemeColorValue::hex("#112233").unwrap(),
        );
        let binding = UiMountedPreviewThemeObservation::admit_from_presentation(
            4,
            Arc::new(BTreeMap::from([(token.clone(), value.clone())])),
        )
        .bind_surface(surface);
        let source = UiMountedThemeValueSource::preview_only(binding.clone());

        assert_eq!(binding.surface(), surface);
        assert_eq!(binding.theme_revision(), 4);
        assert_eq!(source.current_value(&token), Some(&value));
        assert!(!source.uses_frozen_plan());
    }

    #[test]
    fn active_values_carry_values_without_consumer_selection() {
        let token = crate::capability::ThemeTokenId::new("theme.current").unwrap();
        let value = crate::capability::ThemeTokenValue::color(
            crate::capability::ThemeColorValue::hex("#112233").unwrap(),
        );
        let source = UiMountedThemeValueSource::from_current(Arc::new(BTreeMap::from([(
            token.clone(),
            value.clone(),
        )])));

        assert_eq!(source.current_value(&token), Some(&value));
        assert!(!source.has_theme_changes());
    }

    #[test]
    fn preview_binding_observes_values_without_selecting_consumers() {
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let token = crate::capability::ThemeTokenId::new("preview.observed").unwrap();
        let value = crate::capability::ThemeTokenValue::color(
            crate::capability::ThemeColorValue::hex("#445566").unwrap(),
        );
        let source = UiMountedThemeValueSource::preview_only(
            UiMountedPreviewThemeObservation::admit_from_presentation(
                7,
                Arc::new(BTreeMap::from([(token.clone(), value)])),
            )
            .bind_surface(surface),
        );
        assert!(matches!(
            &source,
            UiMountedThemeValueSource::PreviewOnly { .. }
        ));
        assert!(!source.has_theme_changes());
        assert!(!source.uses_frozen_plan());
        assert!(source.current_value(&token).is_some());
    }
}
