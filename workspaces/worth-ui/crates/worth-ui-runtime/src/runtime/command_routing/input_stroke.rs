#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiCommandInputStroke {
    logical: crate::capability::UiCommandShortcutStroke,
    physical: Option<crate::capability::UiCommandShortcutStroke>,
}

impl UiCommandInputStroke {
    pub(super) const fn new(
        logical: crate::capability::UiCommandShortcutStroke,
        physical: Option<crate::capability::UiCommandShortcutStroke>,
    ) -> Self {
        Self { logical, physical }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(super) const fn single(stroke: crate::capability::UiCommandShortcutStroke) -> Self {
        Self::new(stroke, None)
    }

    pub(super) const fn logical(self) -> crate::capability::UiCommandShortcutStroke {
        self.logical
    }

    pub(super) const fn physical(self) -> Option<crate::capability::UiCommandShortcutStroke> {
        self.physical
    }

    pub(super) fn matches(
        self,
        declared: crate::capability::UiCommandShortcutStroke,
        platform: crate::capability::UiCommandShortcutPlatform,
    ) -> bool {
        let declared = declared.resolved_for(platform);
        declared == self.logical.resolved_for(platform)
            || self
                .physical
                .is_some_and(|physical| declared == physical.resolved_for(platform))
    }
}
