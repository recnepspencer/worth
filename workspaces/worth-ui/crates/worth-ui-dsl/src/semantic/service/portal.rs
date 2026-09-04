#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPortalLayer {
    Transient,
    Modal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiPortalDismissalSet(u8);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPortalDeclaration {
    identity: Box<str>,
    surface: Option<Box<str>>,
    anchor: Box<str>,
    layer: WorthUiPortalLayer,
    dismissal: WorthUiPortalDismissalSet,
    first_enabled_focus: bool,
    restore_focus: bool,
    motion: Box<str>,
}

impl WorthUiPortalDismissalSet {
    const ESCAPE: u8 = 1 << 0;
    const OUTSIDE_PRESS: u8 = 1 << 1;
    const ACCEPTED_SELECTION: u8 = 1 << 2;
    const ANCHOR_GONE: u8 = 1 << 3;

    pub(super) fn parse(
        values: &[&str],
    ) -> Result<Self, super::WorthUiServiceDeclarationParseError> {
        let mut bits = 0;
        for value in values {
            bits |= match *value {
                "escape" => Self::ESCAPE,
                "outside_press" => Self::OUTSIDE_PRESS,
                "accepted_selection" => Self::ACCEPTED_SELECTION,
                "anchor_gone" => Self::ANCHOR_GONE,
                _ => {
                    return Err(super::invalid(
                        "portal dismissal",
                        value,
                        "use escape, outside_press, accepted_selection, or anchor_gone",
                    ))
                }
            };
        }
        if bits == 0 {
            return Err(super::missing(
                "portal dismissal",
                "declare at least one typed dismissal cause",
            ));
        }
        Ok(Self(bits))
    }

    pub const fn escape(self) -> bool {
        self.0 & Self::ESCAPE != 0
    }
    pub const fn outside_press(self) -> bool {
        self.0 & Self::OUTSIDE_PRESS != 0
    }
    pub const fn accepted_selection(self) -> bool {
        self.0 & Self::ACCEPTED_SELECTION != 0
    }
    pub const fn anchor_gone(self) -> bool {
        self.0 & Self::ANCHOR_GONE != 0
    }

    pub const fn from_flags(
        escape: bool,
        outside_press: bool,
        accepted_selection: bool,
        anchor_gone: bool,
    ) -> Option<Self> {
        let mut bits = 0;
        if escape {
            bits |= Self::ESCAPE;
        }
        if outside_press {
            bits |= Self::OUTSIDE_PRESS;
        }
        if accepted_selection {
            bits |= Self::ACCEPTED_SELECTION;
        }
        if anchor_gone {
            bits |= Self::ANCHOR_GONE;
        }
        if bits == 0 {
            None
        } else {
            Some(Self(bits))
        }
    }
    pub(super) const fn bits(self) -> u8 {
        self.0
    }
}

impl WorthUiPortalDeclaration {
    pub fn authoring(
        identity: impl Into<Box<str>>,
        surface: impl Into<Box<str>>,
        anchor: impl Into<Box<str>>,
        layer: WorthUiPortalLayer,
        dismissal: WorthUiPortalDismissalSet,
        first_enabled_focus: bool,
        restore_focus: bool,
        motion: impl Into<Box<str>>,
    ) -> Result<Self, super::WorthUiServiceDeclarationParseError> {
        let surface = surface.into();
        if surface.trim().is_empty() {
            return Err(super::missing(
                "portal surface",
                "add the declared semantic surface identity",
            ));
        }
        let anchor = anchor.into();
        if anchor.trim().is_empty() {
            return Err(super::missing(
                "portal anchor",
                "add the authored anchor identity",
            ));
        }
        if !first_enabled_focus {
            return Err(super::missing("portal focus", "include first_enabled"));
        }
        let motion = motion.into();
        if motion.as_ref() != "system_popover" {
            return Err(super::invalid(
                "portal motion",
                &motion,
                "use system_popover",
            ));
        }
        Ok(Self {
            identity: identity.into(),
            surface: Some(surface),
            anchor,
            layer,
            dismissal,
            first_enabled_focus,
            restore_focus,
            motion,
        })
    }

    pub(super) fn parse(
        identity: &str,
        words: &[super::Word],
    ) -> Result<Self, super::WorthUiServiceDeclarationParseError> {
        super::validate_clauses(
            words,
            &[
                super::ClauseRule::Single("surface"),
                super::ClauseRule::Single("anchor"),
                super::ClauseRule::Single("layer"),
                super::ClauseRule::List("dismiss"),
                super::ClauseRule::List("focus"),
                super::ClauseRule::Single("motion"),
            ],
        )?;
        let surface = super::optional_value(words, "surface").map(Into::into);
        let anchor = super::one_value(words, "anchor")?;
        let layer = match super::one_value(words, "layer")? {
            "transient" => WorthUiPortalLayer::Transient,
            "modal" => WorthUiPortalLayer::Modal,
            value => {
                return Err(super::invalid(
                    "portal layer",
                    value,
                    "use transient or modal",
                ))
            }
        };
        let dismissal = WorthUiPortalDismissalSet::parse(&super::values_until(
            words,
            "dismiss",
            &["focus", "motion"],
        )?)?;
        let focus = super::values_until(words, "focus", &["motion"])?;
        if let Some(unsupported) = focus
            .iter()
            .find(|value| !matches!(**value, "first_enabled" | "restore"))
        {
            return Err(super::invalid(
                "portal focus",
                unsupported,
                "use first_enabled and optional restore",
            ));
        }
        let first_enabled_focus = focus.contains(&"first_enabled");
        let restore_focus = focus.contains(&"restore");
        if !first_enabled_focus {
            return Err(super::missing("portal focus", "include first_enabled"));
        }
        let motion = super::one_value(words, "motion")?;
        if motion != "system_popover" {
            return Err(super::invalid(
                "portal motion",
                motion,
                "use system_popover",
            ));
        }
        Ok(Self {
            identity: identity.into(),
            surface,
            anchor: anchor.into(),
            layer,
            dismissal,
            first_enabled_focus,
            restore_focus,
            motion: motion.into(),
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn surface(&self) -> Option<&str> {
        self.surface.as_deref()
    }
    pub fn anchor(&self) -> &str {
        &self.anchor
    }
    pub const fn layer(&self) -> WorthUiPortalLayer {
        self.layer
    }
    pub const fn dismissal(&self) -> WorthUiPortalDismissalSet {
        self.dismissal
    }
    pub const fn focuses_first_enabled(&self) -> bool {
        self.first_enabled_focus
    }
    pub const fn restores_focus(&self) -> bool {
        self.restore_focus
    }
    pub fn motion(&self) -> &str {
        &self.motion
    }
    pub(super) fn canonical_text(&self) -> String {
        format!(
            "portal:{}:{}:{}:{:?}:{}:{}:{}:{}",
            self.identity,
            self.surface.as_deref().unwrap_or("<undeclared>"),
            self.anchor,
            self.layer,
            self.dismissal.bits(),
            self.first_enabled_focus,
            self.restore_focus,
            self.motion
        )
    }
}
