//! Runtime-admitted scroll chrome: the axis support, appearance role
//! identities and metrics the Scroll runtime is allowed to derive chrome from.
//!
//! The *declaration* of chrome is public surface owned by the region-kind
//! descriptor (`UiScrollChromeContract`). This file holds the strictly
//! runtime-side admitted form. It deliberately does not re-declare the public
//! type: [`UiScrollAdmittedChrome::admit_declared_chrome`] is the single
//! adapter point where a declared contract's axis support and role identities
//! become runtime-admitted chrome.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeAxisSupport {
    Inline,
    Block,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeAdmissionDenial {
    /// One registered role cannot style both the track and the thumb: their
    /// normal, hover and drag appearances are distinct product decisions.
    RoleIdentitiesCollide,
}

/// Chrome the Scroll runtime has admitted for one region kind. Carries no
/// authority over offsets; it only says which axes may present chrome, which
/// registered roles style it, and with which admitted metrics.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiScrollAdmittedChrome {
    axes: UiScrollChromeAxisSupport,
    track_role: worth_ui_dsl::UiAppearanceRoleIdentity,
    thumb_role: worth_ui_dsl::UiAppearanceRoleIdentity,
    metrics: super::UiScrollChromeMetrics,
}

impl UiScrollChromeAxisSupport {
    pub(crate) const fn supports_inline(self) -> bool {
        matches!(self, Self::Inline | Self::Both)
    }

    pub(crate) const fn supports_block(self) -> bool {
        matches!(self, Self::Block | Self::Both)
    }
}

impl UiScrollAdmittedChrome {
    /// The single adapter point from a declared region chrome contract to the
    /// runtime-admitted form. Axis support and role identities arrive as plain
    /// parameters so this runtime type never duplicates the public declaration
    /// type, and so the declaration keeps sole authorship of what was authored.
    pub(crate) fn admit_declared_chrome(
        axes: UiScrollChromeAxisSupport,
        track_role: worth_ui_dsl::UiAppearanceRoleIdentity,
        thumb_role: worth_ui_dsl::UiAppearanceRoleIdentity,
        metrics: super::UiScrollChromeMetrics,
    ) -> Result<Self, UiScrollChromeAdmissionDenial> {
        if track_role == thumb_role {
            return Err(UiScrollChromeAdmissionDenial::RoleIdentitiesCollide);
        }
        Ok(Self {
            axes,
            track_role,
            thumb_role,
            metrics,
        })
    }

    pub(crate) const fn axes(&self) -> UiScrollChromeAxisSupport {
        self.axes
    }

    pub(crate) const fn track_role(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.track_role
    }

    pub(crate) const fn thumb_role(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.thumb_role
    }

    pub(crate) const fn metrics(&self) -> super::UiScrollChromeMetrics {
        self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(name: &str) -> worth_ui_dsl::UiAppearanceRoleIdentity {
        worth_ui_dsl::UiAppearanceRoleIdentity::new(name).expect("role identity")
    }

    #[test]
    fn admission_refuses_one_role_standing_in_for_both_track_and_thumb() {
        assert_eq!(
            UiScrollAdmittedChrome::admit_declared_chrome(
                UiScrollChromeAxisSupport::Both,
                role("test.scroll_chrome"),
                role("test.scroll_chrome"),
                super::super::UiScrollChromeMetrics::declared(),
            ),
            Err(UiScrollChromeAdmissionDenial::RoleIdentitiesCollide)
        );
    }

    #[test]
    fn admitted_chrome_reports_the_roles_and_metrics_it_was_admitted_with() {
        let admitted = UiScrollAdmittedChrome::admit_declared_chrome(
            UiScrollChromeAxisSupport::Block,
            role("test.scroll_track"),
            role("test.scroll_thumb"),
            super::super::UiScrollChromeMetrics::declared(),
        )
        .expect("distinct roles");

        assert_eq!(*admitted.track_role(), role("test.scroll_track"));
        assert_eq!(*admitted.thumb_role(), role("test.scroll_thumb"));
        assert_eq!(admitted.axes(), UiScrollChromeAxisSupport::Block);
        assert_eq!(
            admitted.metrics(),
            super::super::UiScrollChromeMetrics::declared()
        );
    }

    #[test]
    fn axis_support_answers_only_for_the_axes_it_names() {
        assert!(UiScrollChromeAxisSupport::Inline.supports_inline());
        assert!(!UiScrollChromeAxisSupport::Inline.supports_block());
        assert!(!UiScrollChromeAxisSupport::Block.supports_inline());
        assert!(UiScrollChromeAxisSupport::Block.supports_block());
        assert!(UiScrollChromeAxisSupport::Both.supports_inline());
        assert!(UiScrollChromeAxisSupport::Both.supports_block());
    }
}
