#![allow(
    dead_code,
    reason = "Gate 1 retains the mounted preview theme seam for later appearance publication"
)]

use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct UiMountedPreviewThemeBinding {
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    theme_revision: u64,
    values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
}

impl UiMountedPreviewThemeBinding {
    pub(crate) fn from_presentation(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        theme_revision: u64,
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
    ) -> Self {
        Self {
            surface,
            theme_revision,
            values,
        }
    }

    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }

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
    CanonicalSelection {
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
        selection: crate::runtime::appearance::UiAppearanceConsumerSelection,
    },
    ReplacementCandidateFrozenPlan,
    PreviewOnly {
        binding: UiMountedPreviewThemeBinding,
    },
}

impl UiMountedThemeValueSource {
    pub(crate) fn from_canonical_selection(
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>>,
        selection: crate::runtime::appearance::UiAppearanceConsumerSelection,
    ) -> Self {
        Self::CanonicalSelection { values, selection }
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
            Self::CanonicalSelection { values, .. } => values.get(token),
            Self::PreviewOnly { binding } => binding.current_value(token),
            Self::ReplacementCandidateFrozenPlan => None,
        }
    }

    pub(crate) const fn uses_frozen_plan(&self) -> bool {
        matches!(self, Self::ReplacementCandidateFrozenPlan)
    }

    pub(crate) fn canonical_consumers(&self) -> &[crate::graph::UiGraphNodeIdentity] {
        match self {
            Self::CanonicalSelection { selection, .. } => selection.consumers(),
            Self::ReplacementCandidateFrozenPlan | Self::PreviewOnly { .. } => &[],
        }
    }

    pub(crate) fn is_canonically_selected(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> bool {
        self.canonical_consumers()
            .binary_search(&graph_node)
            .is_ok()
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
        let binding = UiMountedPreviewThemeBinding::from_presentation(
            surface,
            4,
            Arc::new(BTreeMap::from([(token.clone(), value.clone())])),
        );
        let source = UiMountedThemeValueSource::preview_only(binding.clone());

        assert_eq!(binding.surface(), surface);
        assert_eq!(binding.theme_revision(), 4);
        assert_eq!(source.current_value(&token), Some(&value));
        assert!(!source.uses_frozen_plan());
    }

    #[test]
    fn active_values_expose_only_the_canonical_consumer_selection() {
        let token = crate::capability::ThemeTokenId::new("theme.current").unwrap();
        let value = crate::capability::ThemeTokenValue::color(
            crate::capability::ThemeColorValue::hex("#112233").unwrap(),
        );
        let selected = crate::graph::UiGraphNodeIdentity::new(92_001);
        let unselected = crate::graph::UiGraphNodeIdentity::new(92_002);
        let selection =
            crate::runtime::appearance::UiAppearanceConsumerSelection::for_test([selected]);
        let source = UiMountedThemeValueSource::from_canonical_selection(
            Arc::new(BTreeMap::from([(token.clone(), value.clone())])),
            selection,
        );

        assert_eq!(source.current_value(&token), Some(&value));
        assert_eq!(source.canonical_consumers(), &[selected]);
        assert!(source.is_canonically_selected(selected));
        assert!(!source.is_canonically_selected(unselected));
    }
}
