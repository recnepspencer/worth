use worth_ui::facade::declaration::{
    ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource,
    ThemeTokenValue, UiThemeColor,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseRgba([u8; 4]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulsePaletteRole {
    Canvas,
    RaisedSurface,
    ElevatedSurface,
    StructuralRule,
    PrimaryText,
    SecondaryText,
    PrincipalAccent,
    ActionFill,
    ActionText,
    Positive,
    Caution,
    NavigationSurface,
    HoverSurface,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulseSourceSignalRole {
    SourceSignalBlue,
    SourceSignalGreen,
}

impl PlatformPulsePaletteRole {
    pub const ALL: [Self; 13] = [
        Self::Canvas,
        Self::RaisedSurface,
        Self::ElevatedSurface,
        Self::StructuralRule,
        Self::PrimaryText,
        Self::SecondaryText,
        Self::PrincipalAccent,
        Self::ActionFill,
        Self::ActionText,
        Self::Positive,
        Self::Caution,
        Self::NavigationSurface,
        Self::HoverSurface,
    ];

    pub const fn authored_rgba(self) -> PlatformPulseRgba {
        PlatformPulseRgba(match self {
            Self::Canvas => [0xF6, 0xF4, 0xEF, 0xFF],
            Self::RaisedSurface => [0xFF, 0xFF, 0xFF, 0xFF],
            Self::ElevatedSurface => [0xF0, 0xEE, 0xF8, 0xFF],
            Self::StructuralRule => [0xD8, 0xDC, 0xE7, 0xFF],
            Self::PrimaryText => [0x17, 0x21, 0x3A, 0xFF],
            Self::SecondaryText => [0x66, 0x70, 0x85, 0xFF],
            Self::PrincipalAccent => [0x74, 0x67, 0xE8, 0xFF],
            Self::ActionFill => [0x17, 0x23, 0x3C, 0xFF],
            Self::ActionText => [0xFF, 0xFF, 0xFF, 0xFF],
            Self::Positive => [0x2E, 0x9D, 0x72, 0xFF],
            Self::Caution => [0xC9, 0x87, 0x2D, 0xFF],
            Self::NavigationSurface => [0x17, 0x23, 0x3C, 0xFF],
            Self::HoverSurface => [0xE9, 0xE6, 0xFF, 0xFF],
        })
    }

    pub const fn authored_identity(self) -> &'static str {
        match self {
            Self::Canvas => "canvas",
            Self::RaisedSurface => "raised-surface",
            Self::ElevatedSurface => "elevated-surface",
            Self::StructuralRule => "structural-rule",
            Self::PrimaryText => "primary-text",
            Self::SecondaryText => "secondary-text",
            Self::PrincipalAccent => "principal-accent",
            Self::ActionFill => "action-fill",
            Self::ActionText => "action-text",
            Self::Positive => "positive",
            Self::Caution => "caution",
            Self::NavigationSurface => "navigation-surface",
            Self::HoverSurface => "hover-surface",
        }
    }

    pub const fn authored_hex(self) -> &'static str {
        match self {
            Self::Canvas => "#F6F4EF",
            Self::RaisedSurface => "#FFFFFF",
            Self::ElevatedSurface => "#F0EEF8",
            Self::StructuralRule => "#D8DCE7",
            Self::PrimaryText => "#17213A",
            Self::SecondaryText => "#667085",
            Self::PrincipalAccent => "#7467E8",
            Self::ActionFill => "#17233C",
            Self::ActionText => "#FFFFFF",
            Self::Positive => "#2E9D72",
            Self::Caution => "#C9872D",
            Self::NavigationSurface => "#17233C",
            Self::HoverSurface => "#E9E6FF",
        }
    }

    const fn token_segment(self) -> &'static str {
        match self {
            Self::Canvas => "canvas",
            Self::RaisedSurface => "raised_surface",
            Self::ElevatedSurface => "elevated_surface",
            Self::StructuralRule => "structural_rule",
            Self::PrimaryText => "primary_text",
            Self::SecondaryText => "secondary_text",
            Self::PrincipalAccent => "principal_accent",
            Self::ActionFill => "action_fill",
            Self::ActionText => "action_text",
            Self::Positive => "positive",
            Self::Caution => "caution",
            Self::NavigationSurface => "navigation_surface",
            Self::HoverSurface => "hover_surface",
        }
    }

    pub fn token_id(self) -> ThemeTokenId {
        ThemeTokenId::new(format!("theme.platform_pulse.{}", self.token_segment()))
            .expect("Pulse palette roles are valid token identities")
    }

    fn palette_token_id(self) -> ThemeTokenId {
        ThemeTokenId::new(format!(
            "theme.platform_pulse.palette.{}",
            self.token_segment()
        ))
        .expect("Pulse base palette roles are valid token identities")
    }

    pub fn token_descriptor(self) -> ThemeTokenDescriptor {
        ThemeTokenDescriptor::define(
            self.palette_token_id(),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(
                UiThemeColor::parse(self.authored_hex())
                    .expect("Pulse palette values are valid authored colors"),
            ),
        )
    }

    pub fn source_alias_descriptor(self) -> ThemeTokenDescriptor {
        ThemeTokenDescriptor::alias(
            self.token_id(),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(self.palette_token_id()),
        )
    }
}

impl PlatformPulseSourceSignalRole {
    pub const ALL: [Self; 2] = [Self::SourceSignalBlue, Self::SourceSignalGreen];

    pub const fn authored_rgba(self) -> PlatformPulseRgba {
        PlatformPulseRgba(match self {
            Self::SourceSignalBlue => [0x2F, 0x81, 0xF7, 0xFF],
            Self::SourceSignalGreen => [0x3F, 0xB9, 0x50, 0xFF],
        })
    }

    pub const fn authored_identity(self) -> &'static str {
        match self {
            Self::SourceSignalBlue => "source-signal-blue",
            Self::SourceSignalGreen => "source-signal-green",
        }
    }

    pub fn token_id(self) -> ThemeTokenId {
        let segment = match self {
            Self::SourceSignalBlue => "blue",
            Self::SourceSignalGreen => "green",
        };
        ThemeTokenId::new(format!("theme.platform_pulse.{segment}"))
            .expect("Pulse source-signal roles are valid token identities")
    }

    pub fn token_descriptor(self) -> ThemeTokenDescriptor {
        ThemeTokenDescriptor::define(
            self.token_id(),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(
                UiThemeColor::parse(match self {
                    Self::SourceSignalBlue => "#2F81F7",
                    Self::SourceSignalGreen => "#3FB950",
                })
                .expect("Pulse source-signal values are valid authored colors"),
            ),
        )
    }
}

impl PlatformPulseRgba {
    pub const fn channels(self) -> [u8; 4] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn authored_roles_build_distinct_real_application_tokens() {
        let descriptors = PlatformPulsePaletteRole::ALL
            .into_iter()
            .map(PlatformPulsePaletteRole::token_descriptor)
            .collect::<Vec<_>>();
        assert_eq!(
            descriptors
                .iter()
                .map(|descriptor| descriptor.id().as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            PlatformPulsePaletteRole::ALL.len()
        );
        assert!(descriptors
            .iter()
            .all(|descriptor| descriptor.value().is_some()));
    }
}
