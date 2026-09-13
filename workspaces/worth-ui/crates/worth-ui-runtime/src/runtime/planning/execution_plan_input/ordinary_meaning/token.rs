use crate::source::WorthUiBoundThemeTokenSemantics;

use super::digest::fold_text;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiTokenPlanMeaning {
    entry: crate::capability::FrozenThemeTokenEntry,
    semantics: WorthUiBoundThemeTokenSemantics,
}

impl WorthUiTokenPlanMeaning {
    pub(crate) fn new(
        entry: crate::capability::FrozenThemeTokenEntry,
        semantics: WorthUiBoundThemeTokenSemantics,
    ) -> Self {
        Self { entry, semantics }
    }

    pub(crate) fn token_id(&self) -> &str {
        self.entry.descriptor().id().as_str()
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let digest = fold_text(0x746f_6b65_6e00_0001, self.token_id());
        let digest = self
            .entry
            .descriptor()
            .value()
            .map_or(digest, |value| fold_text(digest, &value.digest_basis()));
        fold_text(
            digest,
            self.semantics.resolved_target_theme_token().id().as_str(),
        )
    }

    pub(crate) fn entry(&self) -> &crate::capability::FrozenThemeTokenEntry {
        &self.entry
    }

    pub(crate) fn semantics(&self) -> &WorthUiBoundThemeTokenSemantics {
        &self.semantics
    }
}
